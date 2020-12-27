use super::BuildNode;
use crane_core::{Artifact, DepGraph, Label};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Topo;
use std::collections::HashMap;

pub struct BuildGraph {
    /// The build graph from all of the BuildNodes, linked by their Labels
    pub dep_graph: Graph<BuildNode, Label>,

    /// A lookup map used to find a node in the graph by its label alone
    nodes: HashMap<Label, NodeIndex>,
}

impl BuildGraph {
    pub fn from_dep_graph(dep_graph: DepGraph) -> Result<BuildGraph, anyhow::Error> {
        let nodes = dep_graph.nodes;

        // NOTE(@ostera): if the graph was topologically sorted by now, during this
        // map we could also compute the hash of each node since their dependencies
        // would already have been hashed.
        let dep_graph = dep_graph._inner_graph.map(
            |_idx, target| BuildNode::from_target(target.clone_boxed()),
            |_idx, edge| edge.clone(),
        );

        Ok(BuildGraph { dep_graph, nodes })
    }

    pub fn find_dependency_hashes_and_outputs(
        &self,
        node: &BuildNode,
    ) -> Vec<(String, Vec<Artifact>)> {
        node.deps()
            .iter()
            .flat_map(|l| self.find_hash_and_outs(l))
            .collect()
    }

    fn find_hash_and_outs(&self, label: &Label) -> Option<(String, Vec<Artifact>)> {
        let node_index = *self.nodes.get(label)?;
        let node = &self.dep_graph[node_index];
        Some((node.hash(), node.outs()))
    }

    pub fn find_nodes(&self, labels: &[Label]) -> Vec<&BuildNode> {
        labels.iter().flat_map(|l| self.find_node(l)).collect()
    }

    pub fn find_node(&self, label: &Label) -> Option<&BuildNode> {
        let node_index = *self.nodes.get(label)?;
        Some(&self.dep_graph[node_index])
    }

    pub fn find_node_mut(&mut self, label: &Label) -> Option<&mut BuildNode> {
        let node_index = *self.nodes.get(label)?;
        Some(&mut self.dep_graph[node_index])
    }

    pub fn target_names(&mut self) -> Vec<String> {
        self.dep_graph.reverse();
        let mut walker = Topo::new(&self.dep_graph);

        let mut nodes: Vec<String> = vec![];
        while let Some(idx) = walker.next(&self.dep_graph) {
            nodes.push(self.dep_graph[idx].name().to_string());
        }

        self.dep_graph.reverse();

        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crane_core::{Artifact, Rule, Target};
    use std::path::PathBuf;

    #[derive(Clone, Debug)]
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

    #[derive(Clone, Debug)]
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

    fn fixture_graph() -> Vec<Box<dyn Target>> {
        vec![
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
        ]
    }

    #[test]
    fn respects_structure() {
        let targets = fixture_graph();
        let mut depgraph = DepGraph::from_targets(targets).unwrap();
        let dep_names = depgraph.target_names();
        let mut buildgraph = BuildGraph::from_dep_graph(depgraph).unwrap();
        let build_names = buildgraph.target_names();
        assert_eq!(build_names, dep_names);
    }

    #[test]
    fn only_finds_existing_nodes() {
        let targets = fixture_graph();
        let depgraph = DepGraph::from_targets(targets).unwrap();
        let buildgraph = BuildGraph::from_dep_graph(depgraph).unwrap();
        let nodes = buildgraph.find_nodes(&vec![Label::new(":c"), Label::new("//z")]);
        assert_eq!(1, nodes.len());
        assert_eq!(
            r#"[
    BuildNode {
        target: test_rule(name = ":c"),
        hash: None,
        outs: None,
        srcs: None,
        status: Pending,
    },
]"#,
            format!("{:#?}", nodes)
        );
    }

    #[test]
    fn node_without_deps_returns_empty_hash_and_output_vector() {
        let targets = fixture_graph();
        let depgraph = DepGraph::from_targets(targets).unwrap();
        let buildgraph = BuildGraph::from_dep_graph(depgraph).unwrap();

        // Node without dependencies just returns an empty vector
        let node = buildgraph.find_node(&Label::new(":a")).unwrap();
        let deps = buildgraph.find_dependency_hashes_and_outputs(&node);
        assert_eq!(r#"[]"#, format!("{:#?}", deps));
    }

    #[test]
    #[should_panic]
    fn node_with_unhashed_deps_panics() {
        let targets = fixture_graph();
        let depgraph = DepGraph::from_targets(targets).unwrap();
        let buildgraph = BuildGraph::from_dep_graph(depgraph).unwrap();

        let node = buildgraph.find_node(&Label::new(":c")).unwrap();
        let deps = buildgraph.find_dependency_hashes_and_outputs(&node);
        assert_eq!(r#"[]"#, format!("{:#?}", deps));
    }

    #[test]
    fn node_with_hashed_deps_returns_their_hash_and_output() {
        let targets = fixture_graph();
        let depgraph = DepGraph::from_targets(targets).unwrap();
        let mut buildgraph = BuildGraph::from_dep_graph(depgraph).unwrap();

        let node = buildgraph.find_node_mut(&Label::new(":a")).unwrap();
        node.update_hash(vec![]);
        let a_hash = node.hash();

        let node = buildgraph.find_node(&Label::new(":c")).unwrap();
        let deps = buildgraph.find_dependency_hashes_and_outputs(&node);
        assert_eq!(1, deps.len());
        assert_eq!(a_hash, deps[0].0);
        assert_eq!(
            r#"[("a4839edbf020b8c1ac398fa119979fc5384f52d4", [])]"#,
            format!("{:?}", deps)
        );
    }
}
