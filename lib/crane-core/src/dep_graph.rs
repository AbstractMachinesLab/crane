use super::{Label, Target};
use anyhow::Context;
use log::debug;
use petgraph::dot;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Topo;
use petgraph::Direction;
use std::collections::HashMap;

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
#[derive(Debug)]
pub struct DepGraph {
    /// The build graph from all of the Targets, linked by their Labels
    pub _inner_graph: Graph<Box<dyn Target>, Label>,

    /// A lookup map used to find a node in the graph by its label alone
    pub nodes: HashMap<Label, NodeIndex>,
}

impl DepGraph {
    pub fn from_targets(targets: Vec<Box<dyn Target>>) -> Result<DepGraph, anyhow::Error> {
        let mut _inner_graph: Graph<Box<dyn Target>, Label> = Graph::new();
        let mut nodes: HashMap<Label, NodeIndex> = HashMap::new();

        debug!("Building table of labels to node indices...");
        for target in targets {
            let label = target.name().clone();
            let node = _inner_graph.add_node(target);
            debug!("{:?} -> {:?}", &label, &node);
            nodes.insert(label, node);
        }
        debug!("Added {} nodes to table.", nodes.len());

        let mut edges = vec![];
        debug!("Building dependency adjacency matrix...");
        for node_idx in nodes.values() {
            let node = _inner_graph.node_weight(*node_idx).unwrap();
            debug!("{:?} depends on:", node.name());
            for label in node.deps().iter().cloned() {
                let dep = nodes.get(&label);
                debug!("\t-> {:?} ({:?})", &label, &dep);
                if let Some(dep) = dep {
                    let edge = (*node_idx, *dep);
                    edges.push(edge);
                } else {
                    panic!(
                        "Could not resolve dependency {:?} for target {:?}",
                        &label.to_string(),
                        node.name()
                    );
                }
            }
        }
        edges.sort();
        debug!("Adding {} edges to graph...", edges.len());
        _inner_graph.extend_with_edges(&edges);
        debug!(
            "Build graph completed with {} nodes and {} edges",
            &nodes.len(),
            &edges.len()
        );

        Ok(DepGraph {
            _inner_graph,
            nodes,
        })
    }

    pub fn scoped(self, target: Label) -> Result<DepGraph, anyhow::Error> {
        let mut build_plan = self;

        if target.is_all() {
            Ok(build_plan)
        } else {
            let node_index = build_plan
                .nodes
                .get(&target)
                .context(format!("Could not find node: {:?}", &target))?;
            let nodes_to_keep = DepGraph::subgraph(&mut build_plan._inner_graph, *node_index);
            build_plan
                ._inner_graph
                .retain_nodes(|_g, node| nodes_to_keep.contains_key(&node));
            Ok(build_plan)
        }
    }

    fn subgraph(
        mut graph: &mut Graph<Box<dyn Target>, Label>,
        node: NodeIndex,
    ) -> HashMap<NodeIndex, ()> {
        let mut nodes = HashMap::new();
        nodes.insert(node, ());
        nodes.insert(node, ());

        let mut walker = graph.neighbors_directed(node, Direction::Outgoing).detach();
        while let Some(neighbor) = walker.next_node(&graph) {
            nodes.insert(neighbor, ());
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
                &|_graph, (_idx, target)| format!("label = \"{}\"", target.name().to_string())
            )
        )
    }

    pub fn find_nodes(&self, labels: &[Label]) -> Vec<&dyn Target> {
        labels.iter().flat_map(|l| self.find_node(l)).collect()
    }

    pub fn find_node(&self, label: &Label) -> Option<&dyn Target> {
        let node_index = *self.nodes.get(label)?;
        Some(&*self._inner_graph[node_index])
    }

    pub fn target_names(&mut self) -> Vec<String> {
        self._inner_graph.reverse();
        let mut walker = Topo::new(&self._inner_graph);

        let mut nodes: Vec<String> = vec![];
        while let Some(idx) = walker.next(&self._inner_graph) {
            nodes.push(self._inner_graph[idx].name().to_string());
        }

        self._inner_graph.reverse();

        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Artifact, Rule};
    use std::path::PathBuf;

    #[derive(Debug, Clone)]
    struct TestRule {}
    impl Rule for TestRule {
        fn name(&self) -> &str {
            "test_rule"
        }
        fn toolchains(&self) -> Vec<Label> {
            vec![]
        }
        fn execute(&self) -> Result<(), anyhow::Error> {
            Ok(())
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
        fn name(&self) -> &Label {
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
        fn outputs(&self, _deps: &[Artifact]) -> &[Artifact] {
            &self.outs
        }
    }

    #[test]
    fn does_not_create_graphs_with_cycles() {
        let targets: Vec<Box<dyn Target>> = vec![
            Box::new(TestTarget {
                label: "z".into(),
                deps: vec![Label::new(":z")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![Label::new(":b")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
        ];

        assert_eq!(false, DepGraph::from_targets(targets).is_ok());
    }

    #[test]
    fn respects_dependencies() {
        let targets: Vec<Box<dyn Target>> = vec![
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":c")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "c".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
        ];

        let mut depgraph = DepGraph::from_targets(targets).unwrap();

        let work = depgraph.target_names();

        assert_eq!("[\":a\", \":c\", \":b\"]", format!("{:?}", work));
    }

    #[test]
    fn respects_scoping() {
        let targets: Vec<Box<dyn Target>> = vec![
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":c")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "c".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
        ];

        let mut depgraph = DepGraph::from_targets(targets)
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
        let targets: Vec<Box<dyn Target>> = vec![
            Box::new(TestTarget {
                label: "a".into(),
                deps: vec![],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "b".into(),
                deps: vec![Label::new(":c")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
            Box::new(TestTarget {
                label: "c".into(),
                deps: vec![Label::new(":a")],
                srcs: vec![],
                outs: vec![],
                rule: TestRule {},
            }),
        ];

        let depgraph = DepGraph::from_targets(targets).unwrap();

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
}
