use crate::build_rules::build_rule::BuildRule;
use crate::model::target::Label;
use crate::model::workspace::Workspace;
use crate::toolchains::erlang::Toolchain;
use log::debug;
use petgraph::dot;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Topo;
use petgraph::Direction;
use std::collections::HashMap;
use std::fs;
use std::ops::Index;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BuildContext {
    artifact_root: PathBuf,
    workspace: Workspace,
    toolchain: Toolchain,
    declared_outputs: Vec<PathBuf>,
    build_plan: BuildPlan,
}

impl BuildContext {
    pub fn toolchain(&self) -> Toolchain {
        self.toolchain.clone()
    }

    pub fn find_node(&self, label: &Label) -> Option<BuildRule> {
        self.build_plan.find_node(&label)
    }

    pub fn transitive_dependencies(&self, rule: &BuildRule) -> Vec<BuildRule> {
        rule.dependencies()
            .iter()
            .cloned()
            .flat_map(|dep_label| {
                let node = self.find_node(&dep_label).unwrap();
                let mut tran_deps = self.transitive_dependencies(&node);
                let label = node.label();
                let mut transitive = vec![node];
                transitive.append(&mut tran_deps);
                transitive
            })
            .collect()
    }

    pub fn path_in_context(&self, path: PathBuf) -> PathBuf {
        self.artifact_root.clone().join(path).to_path_buf()
    }

    pub fn declare_output(&mut self, path: PathBuf) -> PathBuf {
        self.declared_outputs.push(path.clone());
        debug!("Declared output {:?}", &path);
        let actual_path = self.path_in_context(path);
        let parent = actual_path.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        actual_path
    }
}

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
        let mut build_graph = Graph::new();
        let mut nodes: HashMap<Label, NodeIndex> = HashMap::new();

        let rules: Vec<BuildRule> = self
            .workspace
            .cranefiles()
            .iter()
            .cloned()
            .flat_map(|cranefile| cranefile.rules())
            .collect();

        debug!("Building temporary table of labels to node indices...");
        for rule in rules {
            let label = rule.label().clone();
            let node = build_graph.add_node(rule);
            debug!("{:?} -> {:?}", &label, &node);
            nodes.insert(label, node);
        }
        debug!("Added {} nodes to table.", nodes.len());

        let mut edges = vec![];
        debug!("Building dependency adjacency matrix...");
        for node in nodes.values() {
            let rule = build_graph.node_weight(*node).unwrap();
            debug!("{:?} depends on:", &rule.label());
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
                        rule.label()
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
            nodes,
            ..self.clone()
        })
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
                &[dot::Config::EdgeNoLabel],
                &|_graph, _edge| "".to_string(),
                &|_graph, (_idx, rule)| format!("label = \"{}\"", rule.label().to_string())
            )
        )
    }

    pub fn find_node(&self, label: &Label) -> Option<BuildRule> {
        let node_index = self.nodes.get(label)?.clone();
        println!("{:?} {:?}", node_index, label);
        let node: BuildRule = self.build_graph[node_index].clone();
        Some(node)
    }

    pub fn execute(&mut self) -> Result<u32, anyhow::Error> {
        let mut ctx = BuildContext {
            artifact_root: self.workspace.root().join("__crane_workspace"),
            workspace: self.workspace.clone(),
            toolchain: Toolchain::default(),
            declared_outputs: vec![],
            build_plan: self.clone(),
        };
        self.build_graph.reverse();
        let mut walker = Topo::new(&self.build_graph);
        let mut artifacts = 0;
        while let Some(node) = walker.next(&self.build_graph) {
            let node = &self.build_graph[node];
            &node.execute(&mut ctx)?;
            artifacts = artifacts + 1;
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
