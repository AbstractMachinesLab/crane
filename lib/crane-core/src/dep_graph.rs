use super::{Label, Target};
use anyhow::Context;
use log::debug;
use petgraph::dot;
use petgraph::graph::{Graph, NodeIndex};
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
    dep_graph: Graph<Box<dyn Target>, Label>,

    /// A lookup map used to find a node in the graph by its label alone
    nodes: HashMap<Label, NodeIndex>,
}

impl DepGraph {
    pub fn from_targets(targets: Vec<Box<dyn Target>>) -> Result<DepGraph, anyhow::Error> {
        let mut dep_graph: Graph<Box<dyn Target>, Label> = Graph::new();
        let mut nodes: HashMap<Label, NodeIndex> = HashMap::new();

        debug!("Building table of labels to node indices...");
        for target in targets {
            let label = target.name().clone();
            let node = dep_graph.add_node(target);
            debug!("{:?} -> {:?}", &label, &node);
            nodes.insert(label, node);
        }
        debug!("Added {} nodes to table.", nodes.len());

        let mut edges = vec![];
        debug!("Building dependency adjacency matrix...");
        for node_idx in nodes.values() {
            let node = dep_graph.node_weight(*node_idx).unwrap();
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
        dep_graph.extend_with_edges(&edges);
        debug!(
            "Build graph completed with {} nodes and {} edges",
            &nodes.len(),
            &edges.len()
        );

        Ok(DepGraph { dep_graph, nodes })
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
            let nodes_to_keep = DepGraph::subgraph(&mut build_plan.dep_graph, *node_index);
            build_plan
                .dep_graph
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
                &self.dep_graph,
                &[dot::Config::EdgeNoLabel, dot::Config::NodeNoLabel],
                &|_graph, _edge| "".to_string(),
                &|_graph, (_idx, target)| format!("label = \"{}\"", target.name().to_string())
            )
        )
    }

    pub fn find_nodes(&self, labels: &[Label]) -> Vec<&Box<dyn Target>> {
        labels.iter().flat_map(|l| self.find_node(l)).collect()
    }

    pub fn find_node(&self, label: &Label) -> Option<&Box<dyn Target>> {
        let node_index = *self.nodes.get(label)?;
        Some(&self.dep_graph[node_index])
    }
}
