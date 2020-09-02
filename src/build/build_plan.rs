use crate::build::{BuildContext, BuildRule};
use crate::label::Label;
use crate::workspace::Workspace;
use log::debug;
use petgraph::dot;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Topo;
use petgraph::Direction;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BuildPlan {
    workspace: Workspace,
    build_graph: Graph<BuildRule, Label>,
    nodes: HashMap<Label, NodeIndex>,
}

impl BuildPlan {
    pub fn for_workspace(workspace: Workspace) -> BuildPlan {
        BuildPlan {
            workspace,
            build_graph: Graph::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn plan(self) -> Result<BuildPlan, anyhow::Error> {
        let rules: Vec<BuildRule> = self
            .workspace
            .cranefiles()
            .iter()
            .cloned()
            .flat_map(|cranefile| cranefile.rules())
            .collect();

        let (build_graph, nodes) = BuildPlan::populate_build_graph(rules);

        Ok(BuildPlan {
            build_graph,
            nodes,
            ..self.clone()
        })
    }

    fn populate_build_graph(
        rules: Vec<BuildRule>,
    ) -> (Graph<BuildRule, Label>, HashMap<Label, NodeIndex>) {
        let mut build_graph = Graph::new();
        let mut nodes: HashMap<Label, NodeIndex> = HashMap::new();

        debug!("Building temporary table of labels to node indices...");
        for rule in rules {
            let label = rule.name().clone();
            let node = build_graph.add_node(rule);
            debug!("{:?} -> {:?}", &label, &node);
            nodes.insert(label, node);
        }
        debug!("Added {} nodes to table.", nodes.len());

        let mut edges = vec![];
        debug!("Building dependency adjacency matrix...");
        for node in nodes.values() {
            let rule = build_graph.node_weight(*node).unwrap();
            debug!("{:?} depends on:", &rule.name());
            for label in rule.dependencies().iter().cloned() {
                let dep = nodes.get(&label);
                debug!("\t-> {:?} ({:?})", &label, &dep);
                if let Some(dep) = dep {
                    let edge = (*node, *dep);
                    edges.push(edge);
                } else {
                    panic!(
                        "Could not resolve dependency {:?} for target {:?}",
                        &label.to_string(),
                        rule.name()
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
        (build_graph, nodes)
    }

    pub fn scoped(self, target: Label) -> Result<BuildPlan, anyhow::Error> {
        let mut build_plan = self.clone();

        if target.is_all() {
            Ok(build_plan)
        } else {
            let node_index = build_plan.nodes.get(&target).expect("Could not find node!");
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

    pub fn find_node(&self, label: &Label) -> Option<BuildRule> {
        let node_index = self.nodes.get(label)?.clone();
        let node: BuildRule = self.build_graph[node_index].clone();
        Some(node)
    }

    pub fn run(&mut self) -> Result<u32, anyhow::Error> {
        let mut ctx = BuildContext::new(self.workspace.clone(), self.clone());
        self.build_graph.reverse();
        let mut walker = Topo::new(&self.build_graph);
        let mut artifacts = 0;
        while let Some(node) = walker.next(&self.build_graph) {
            let node = &mut self.build_graph[node];
            node.run(&mut ctx)?;
            artifacts = artifacts + 1;
        }
        Ok(artifacts)
    }

    pub fn build(&mut self) -> Result<u32, anyhow::Error> {
        let mut ctx = BuildContext::new(self.workspace.clone(), self.clone());
        self.build_graph.reverse();
        let mut walker = Topo::new(&self.build_graph);
        let mut artifacts = 0;
        while let Some(node) = walker.next(&self.build_graph) {
            let node = &mut self.build_graph[node];
            let inputs = &node.inputs(&mut ctx);
            let outputs = &node.outputs(&mut ctx);
            let name = &node.name();
            if let Some(changes) = &ctx.changed_inputs(inputs) {
                debug!("Changed inputs: {:?}...", &changes);
                debug!("Building {}...", &name.to_string());
                &node.build(&mut ctx)?;
                artifacts = artifacts + 1;
                &ctx.update_cache(inputs, outputs)?;
            } else {
                debug!("Skipping {}. Nothing to do.", &name.to_string());
            }
        }
        Ok(artifacts)
    }
}

fn subgraph(graph: Graph<BuildRule, Label>, node: NodeIndex) -> HashMap<NodeIndex, ()> {
    let mut nodes = HashMap::new();
    nodes.insert(node, ());

    let mut walker = graph.neighbors_directed(node, Direction::Outgoing).detach();
    while let Some(neighbor) = walker.next_node(&graph) {
        nodes.insert(neighbor.clone(), ());
        nodes.extend(subgraph(graph.clone(), neighbor));
    }

    nodes
}
