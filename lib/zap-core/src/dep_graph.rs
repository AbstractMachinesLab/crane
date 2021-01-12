use super::{Action, ComputedTarget, Dependency, Label, Target};
use anyhow::{anyhow, Context};
use daggy::{Dag, NodeIndex};
use dashmap::DashMap;
use log::debug;
use petgraph::dot;
use petgraph::{stable_graph::StableDiGraph, Direction};
use std::collections::HashMap;
use std::path::PathBuf;
use zap_buildscript::*;

/// The DepGraph contains the graph of all the targets in this project.
///
/// It is used to perform a dependency-first traversal (aka, Topological) that
/// will ensure that every target is executed _after_ its dependencies have been
/// successfully executed.
///
/// Initially, the DepGraph will populate its internal graph with all
/// of the Targets given to it. However, it is most often the case that we
/// want to build a subset of the entirety of a project, so scoping the path to
/// our target node throughout this build graph is incredibly useful.
///
/// To do this, we can call `build_plan.scoped(label)` where Label is our target
/// node's label.
///
/// This will trim down the build graph to the smallest graph that would satisfy
/// our target's dependencies transitively.
///
/// In other words, this struct takes care of figuring out what is the smallest
/// amount of work that needs to be done.
///
#[derive(Debug, Default)]
pub struct DepGraph {
    /// The build graph from all of the Targets, linked by their Labels
    pub _inner_graph: StableDiGraph<ComputedTarget, ()>,

    /// A lookup map used to find a node in the graph by its label alone
    pub nodes: HashMap<Label, NodeIndex>,
}

impl DepGraph {
    pub fn from_targets(targets: &[Target]) -> Result<DepGraph, anyhow::Error> {
        let mut dag: Dag<ComputedTarget, (), u32> = Dag::new();
        let mut nodes: HashMap<Label, NodeIndex> = HashMap::new();

        debug!("Building table of labels to node indices...");
        for target in targets {
            let label = target.label().clone();
            let node = dag.add_node(ComputedTarget::from_target(target.clone()));
            debug!("{:?} -> {:?}", &label, &node);
            nodes.insert(label, node);
        }
        debug!("Added {} nodes to table.", nodes.len());

        let mut edges = vec![];
        debug!("Building dependency adjacency matrix...");
        for node_idx in nodes.values() {
            let node = dag.node_weight(*node_idx).unwrap();
            debug!("{:?} depends on:", node.label());
            for label in node.target.deps().iter().cloned() {
                let dep = if label.is_relative() {
                    nodes.get(&label.canonicalize(&node.target.label().path()))
                } else {
                    nodes.get(&label)
                };
                debug!("-> {:?} ({:?})", &label, &dep);
                if let Some(dep) = dep {
                    let edge = (*dep, *node_idx);
                    edges.push(edge);
                } else {
                    return Err(anyhow!(format!(
                        "Could not resolve dependency {:?} for target {:?}",
                        &label.to_string(),
                        node.label().to_string()
                    )));
                }
            }
        }
        edges.sort();
        debug!("Adding {} edges to graph...", edges.len());

        dag.extend_with_edges(&edges)
            .context("Found a cycle in the graph!")?;
        debug!(
            "Build graph completed with {} nodes and {} edges",
            &nodes.len(),
            &edges.len()
        );

        let mut _inner_graph: StableDiGraph<ComputedTarget, ()> = dag.into_graph().into();
        let mut nodes: HashMap<Label, NodeIndex> = HashMap::new();

        let mut walker = petgraph::visit::Topo::new(&_inner_graph);
        while let Some(idx) = walker.next(&_inner_graph) {
            let label = _inner_graph[idx].target.label().clone();
            nodes.insert(label, idx);
        }

        Ok(DepGraph {
            _inner_graph,
            nodes,
        })
    }

    pub fn scoped(&mut self, target: &Label) -> Result<&mut DepGraph, anyhow::Error> {
        if target.is_all() {
            Ok(self)
        } else {
            let node_index = self
                .nodes
                .get(&target)
                .context(format!("Could not find node: {:?}", &target))?;
            let nodes_to_keep = DepGraph::subgraph(&mut self._inner_graph, *node_index);
            self._inner_graph
                .retain_nodes(|_g, node| nodes_to_keep.contains_key(&node));

            Ok(self)
        }
    }

    fn subgraph(
        mut graph: &mut StableDiGraph<ComputedTarget, ()>,
        node: NodeIndex,
    ) -> HashMap<NodeIndex, Label> {
        let mut nodes = HashMap::new();
        nodes.insert(node, graph[node].target.label().clone());

        let mut walker = graph.neighbors_directed(node, Direction::Incoming).detach();
        while let Some(neighbor) = walker.next_node(&graph) {
            nodes.insert(neighbor, graph[neighbor].target.label().clone());
            nodes.extend(DepGraph::subgraph(&mut graph, neighbor));
        }

        nodes
    }

    pub fn to_graphviz(&self) -> String {
        format!(
            "{:?}",
            dot::Dot::with_attr_getters(
                &self._inner_graph,
                &[dot::Config::EdgeNoLabel, dot::Config::NodeNoLabel],
                &|_graph, _edge| "".to_string(),
                &|_graph, (_idx, target)| format!("label = \"{}\"", target.label().to_string())
            )
        )
    }

    pub fn find_nodes(&self, labels: &[Label]) -> Vec<&ComputedTarget> {
        labels.iter().flat_map(|l| self.find_node(l)).collect()
    }

    pub fn find_node(&self, label: &Label) -> Option<&ComputedTarget> {
        let node_index = *self.nodes.get(label)?;
        Some(&self._inner_graph[node_index])
    }

    pub fn seal_target_by_label(
        &mut self,
        label: &Label,
        action_map: &DashMap<Label, Vec<Action>>,
        output_map: &DashMap<Label, Vec<PathBuf>>,
        bs_ctx: &mut BuildScript,
        archive_root: &PathBuf,
    ) -> Result<&ComputedTarget, anyhow::Error> {
        let node_index = *self
            .nodes
            .get(label)
            .context(format!("Could not find node with label: {:?}", label))?;
        self.seal_target(node_index, action_map, output_map, bs_ctx, archive_root)
    }

    pub fn seal_target(
        &mut self,
        node_index: NodeIndex,
        action_map: &DashMap<Label, Vec<Action>>,
        output_map: &DashMap<Label, Vec<PathBuf>>,
        mut bs_ctx: &mut BuildScript,
        archive_root: &PathBuf,
    ) -> Result<&ComputedTarget, anyhow::Error> {
        let labels = self._inner_graph[node_index].target.deps();
        let deps: Vec<Dependency> = self
            .find_nodes(&labels)
            .iter()
            .map(|computed_target| computed_target.as_dep())
            .collect();

        self._inner_graph[node_index].seal(
            &deps,
            &action_map,
            &output_map,
            &mut bs_ctx,
            &archive_root,
        )?;

        Ok(&self._inner_graph[node_index])
    }

    pub fn seal(
        &mut self,
        action_map: &DashMap<Label, Vec<Action>>,
        output_map: &DashMap<Label, Vec<PathBuf>>,
        mut bs_ctx: &mut BuildScript,
        archive_root: &PathBuf,
    ) -> Result<&mut DepGraph, anyhow::Error> {
        let mut walker = petgraph::visit::Topo::new(&self._inner_graph);
        while let Some(idx) = walker.next(&self._inner_graph) {
            self.seal_target(idx, action_map, output_map, &mut bs_ctx, &archive_root)?;
        }

        Ok(self)
    }

    pub fn targets(&mut self) -> Vec<ComputedTarget> {
        let mut walker = petgraph::visit::Topo::new(&self._inner_graph);

        let mut nodes = vec![];
        while let Some(idx) = walker.next(&self._inner_graph) {
            nodes.push(self._inner_graph[idx].clone());
        }

        nodes
    }

    pub fn target_names(&mut self) -> Vec<String> {
        let mut walker = petgraph::visit::Topo::new(&self._inner_graph);

        let mut nodes: Vec<String> = vec![];
        while let Some(idx) = walker.next(&self._inner_graph) {
            nodes.push(self._inner_graph[idx].label().to_string());
        }

        nodes
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Action, Artifact, DepGraph, Rule};
    use std::path::PathBuf;

    #[derive(Clone, Debug, Default)]
    struct TestRule {
        toolchains: Vec<Label>,
    }
    impl Rule for TestRule {
        fn mnemonic(&self) -> &str {
            "test_rule"
        }
        fn toolchains(&self) -> &[Label] {
            &self.toolchains
        }
        fn execute(
            &self,
            _ct: &ComputedTarget,
            _tm: &ToolchainManager,
        ) -> Result<Vec<Action>, anyhow::Error> {
            Ok(vec![])
        }
    }

    #[derive(Debug, Clone)]
    struct TestTarget {
        label: Label,
        deps: Vec<Label>,
        srcs: Vec<PathBuf>,
        outs: Vec<Artifact>,
        rule: TestRule,
    }
    impl Target for TestTarget {
        fn label(&self) -> &Label {
            &self.label
        }
        fn rule(&self) -> &dyn Rule {
            &self.rule
        }
        fn deps(&self) -> &[Label] {
            &self.deps
        }
        fn set_deps(&mut self, _deps: &[Label]) {}
        fn srcs(&self, _deps: &[Artifact]) -> &[PathBuf] {
            &self.srcs
        }
        fn set_srcs(&mut self, _srcs: &[PathBuf]) {}
        fn outputs(&self, _deps: &[Artifact]) -> Vec<Artifact> {
            self.outs.clone()
        }
    }

    #[test]
    fn does_not_create_graphs_with_missing_nodes() {
        let targets: Vec<Target> = vec![Box::new(TestTarget {
            label: "a".into(),
            deps: vec![Label::new(":b")],
            srcs: vec![],
            outs: vec![],
            rule: TestRule::default(),
        })];

        assert_eq!(true, DepGraph::from_targets(&targets).is_err());
        assert_eq!(
            r#"Could not resolve dependency ":b" for target ":a""#,
            format!("{:?}", DepGraph::from_targets(&targets).unwrap_err())
        );
    }

    #[test]
    fn does_not_create_graphs_with_cycles() {
        let targets: Vec<Target> = vec![
            Box::new(TestTarget {
                label: "z".into(),
                deps: vec![Label::new(":z")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![Label::new(":b")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
        ];

        assert_eq!(false, DepGraph::from_targets(&targets).is_ok());
        assert_eq!(
            r#"Found a cycle in the graph!

Caused by:
    WouldCycle

    "#,
            format!("{:?}", DepGraph::from_targets(&targets).unwrap_err())
        );
    }

    #[test]
    fn respects_dependencies() {
        let targets: Vec<Target> = vec![
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":c")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "c".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
        ];

        let mut depgraph = DepGraph::from_targets(&targets).unwrap();

        let work = depgraph.target_names();

        assert_eq!("[\":a\", \":c\", \":b\"]", format!("{:?}", work));
    }

    #[test]
    fn respects_scoping() {
        let targets: Vec<Target> = vec![
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":c")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "c".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
        ];

        let mut depgraph = DepGraph::from_targets(&targets)
            .unwrap()
            .scoped(Label::new(":c"))
            .unwrap();

        let work = depgraph.target_names();

        // We do not expect to see B since our scope is C, and C only depends
        // on A
        assert_eq!("[\":a\", \":c\"]", format!("{:?}", work));
    }

    #[test]
    fn can_be_printed_as_graphviz() {
        let targets: Vec<Target> = vec![
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":c")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "c".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
        ];

        let depgraph = DepGraph::from_targets(&targets).unwrap();

        assert_eq!(
            r#"digraph {
    0 [ label = ":a"]
    1 [ label = ":b"]
    2 [ label = ":c"]
    1 -> 2 [ ]
    2 -> 0 [ ]
}
"#,
            depgraph.to_graphviz()
        );
    }

    fn fixture_graph() -> Vec<Target> {
        vec![
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":c")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
            Box::new(TestTarget {
                label: "c".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule::default(),
            }),
        ]
    }

    #[test]
    fn only_finds_existing_nodes() {
        let targets = fixture_graph();
        let depgraph = DepGraph::from_targets(&targets).unwrap();
        let nodes = depgraph.find_nodes(&vec![Label::new(":c"), Label::new("//z")]);
        assert_eq!(1, nodes.len());
        assert_eq!(
            r#"[
    ComputedTarget {
        target: test_rule(name = ":c"),
        hash: None,
        deps: None,
        outs: None,
        srcs: None,
        actions: None,
        status: Pending,
    },
]"#,
            format!("{:#?}", nodes)
        );
    }

    #[test]
    fn node_without_deps_returns_empty_hash_and_output_vector() {
        let targets = fixture_graph();
        let mut depgraph = DepGraph::from_targets(&targets).unwrap();

        // Node without dependencies just returns an empty vector
        let target = depgraph
            .seal_target_by_label(&Label::new(":a"), &ToolchainManager::new())
            .unwrap();
        assert_eq!(r#"[]"#, format!("{:#?}", target.deps()));
    }

    #[test]
    #[should_panic]
    fn node_with_unhashed_deps_panics() {
        let targets = fixture_graph();
        let depgraph = DepGraph::from_targets(&targets).unwrap();

        // Node without dependencies just returns an empty vector
        let target = depgraph.find_node(&Label::new(":c")).unwrap();
        assert_eq!(r#"[]"#, format!("{:#?}", target.deps()));
    }

    #[test]
    fn node_with_hashed_deps_returns_their_hash_and_output() {
        let targets = fixture_graph();
        let mut depgraph = DepGraph::from_targets(&targets).unwrap();

        let a_hash = depgraph
            .seal_target_by_label(&Label::new(":a"), &ToolchainManager::new())
            .unwrap()
            .hash();

        let node = depgraph
            .seal_target_by_label(&Label::new(":c"), &ToolchainManager::new())
            .unwrap();
        let deps = node.deps();
        assert_eq!(1, deps.len());
        assert_eq!(a_hash, deps[0].hash);
        assert_eq!(":a", deps[0].label.to_string());
        assert_eq!("[]", format!("{:?}", deps[0].outs));
    }
*/
