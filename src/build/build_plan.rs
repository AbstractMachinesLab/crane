use super::{BuildNode, BuildRule};
use crate::label::Label;
use crate::toolchains::ToolchainName;
use anyhow::Context;
use log::debug;
use petgraph::dot;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Direction;
use std::collections::HashMap;
use std::collections::HashSet;

/// The BuildPlan contains the graph of all the nodes that have been scoped for
/// building, and the list of toolchains that are required by these nodes.
///
/// It is used to perform a dependency-first traversal (aka, Topological) that
/// will ensure that every node is executed _after_ its dependencies have been
/// successfully executed.
///
/// Initially, the BuildPlan will populate its internal build graph with all
/// of the BuildRules given to it. However, it is most often the case that we
/// want to build a subset of the entirety of a project, so scoping the path to
/// our target node throughout this build graph is incredibly useful.
///
/// To do this, we can call `build_plan.scoped(label)` where Label is our target
/// node's label.
///
/// This will trim down the build graph to the smallest graph that would satisfy
/// our target's dependencies transitively.
///
/// In other words, this struct takes care of figuring out what the smallest
/// amount of work that needs to be done is.
///
#[derive(Debug, Clone)]
pub struct BuildPlan {
    /// The build graph from all of the BuildNodes, linked by their Labels
    pub build_graph: Graph<BuildNode, Label>,

    /// A lookup map used to find a node in the graph by its label alone
    nodes: HashMap<Label, NodeIndex>,

    /// A set of toolchains collected while scoping down the build plan.
    toolchains: HashSet<ToolchainName>,
}

impl BuildPlan {
    pub fn from_rules(rules: Vec<BuildRule>) -> Result<BuildPlan, anyhow::Error> {
        let mut toolchains: HashSet<ToolchainName> = HashSet::new();
        let mut build_graph: Graph<BuildNode, Label> = Graph::new();
        let mut nodes: HashMap<Label, NodeIndex> = HashMap::new();

        debug!("Building table of labels to node indices...");
        for rule in rules {
            let label = rule.name().clone();
            if let Some(toolchain) = rule.toolchain() {
                toolchains.insert(toolchain);
            }
            let node = build_graph.add_node(BuildNode::from_rule(rule));
            debug!("{:?} -> {:?}", &label, &node);
            nodes.insert(label, node);
        }
        debug!("Added {} nodes to table.", nodes.len());

        let mut edges = vec![];
        debug!("Building dependency adjacency matrix...");
        for node_idx in nodes.values() {
            let node = build_graph.node_weight(*node_idx).unwrap();
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
        debug!("Adding {} edges to graph...", edges.len());
        build_graph.extend_with_edges(&edges);
        debug!(
            "Build graph completed with {} nodes and {} edges",
            &nodes.len(),
            &edges.len()
        );

        Ok(BuildPlan {
            build_graph,
            toolchains,
            nodes,
        })
    }

    pub fn scoped(self, target: Label) -> Result<BuildPlan, anyhow::Error> {
        let mut build_plan = self.clone();

        if target.is_all() {
            Ok(build_plan)
        } else {
            let node_index = build_plan
                .nodes
                .get(&target)
                .context(format!("Could not find node: {:?}", &target))?;
            let nodes_to_keep = subgraph(self.build_graph, *node_index);
            build_plan
                .build_graph
                .retain_nodes(|_g, node| nodes_to_keep.contains_key(&node));
            Ok(build_plan)
        }
    }

    pub fn to_graphviz(&self) -> String {
        format!(
            "{:?}",
            dot::Dot::with_attr_getters(
                &self.build_graph,
                &[dot::Config::EdgeNoLabel, dot::Config::NodeNoLabel],
                &|_graph, _edge| "".to_string(),
                &|_graph, (_idx, rule)| format!("label = \"{}\"", rule.name().to_string())
            )
        )
    }

    pub fn toolchains_in_use(&self) -> Vec<ToolchainName> {
        self.toolchains.clone().into_iter().collect()
    }

    pub fn find_nodes(&self, labels: &[Label]) -> Vec<BuildNode> {
        labels.iter().flat_map(|l| self.find_node(l)).collect()
    }

    pub fn find_node(&self, label: &Label) -> Option<BuildNode> {
        let node_index = *self.nodes.get(label)?;
        Some(self.build_graph[node_index].clone())
    }
}

fn subgraph(graph: Graph<BuildNode, Label>, node: NodeIndex) -> HashMap<NodeIndex, ()> {
    let mut nodes = HashMap::new();
    nodes.insert(node, ());
    nodes.insert(node, ());

    let mut walker = graph.neighbors_directed(node, Direction::Outgoing).detach();
    while let Some(neighbor) = walker.next_node(&graph) {
        nodes.insert(neighbor, ());
        nodes.extend(subgraph(graph.clone(), neighbor));
    }

    nodes
}
