use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub transition: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NodeGraph {
    pub states: Vec<String>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EventGraph {
    pub nodes: HashMap<u32, NodeGraph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EventRecord {
    node_id: u32,
    transition: String,
    state: String,
}

pub fn build_event_graph(log: &str) -> EventGraph {
    let mut records: Vec<EventRecord> = Vec::new();
    let mut states_by_node: HashMap<u32, HashSet<String>> = HashMap::new();

    for line in log.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        let node_id: u32 = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let transition = parts[1].to_string();
        let state = parts[2].to_string();

        states_by_node
            .entry(node_id)
            .or_default()
            .insert(state.clone());
        records.push(EventRecord {
            node_id,
            transition,
            state,
        });
    }

    let mut edges_by_node: HashMap<u32, HashSet<Edge>> = HashMap::new();
    let mut last_for_node: HashMap<u32, (String, String)> = HashMap::new();
    for rec in &records {
        if let Some((prev_state, prev_transition)) = last_for_node.get(&rec.node_id) {
            edges_by_node.entry(rec.node_id).or_default().insert(Edge {
                from: prev_state.clone(),
                transition: prev_transition.clone(),
                to: rec.state.clone(),
            });
        }
        last_for_node.insert(rec.node_id, (rec.state.clone(), rec.transition.clone()));
    }

    let mut nodes: HashMap<u32, NodeGraph> = HashMap::new();
    let all_node_ids: HashSet<u32> = states_by_node
        .keys()
        .chain(edges_by_node.keys())
        .copied()
        .collect();

    for node_id in all_node_ids {
        let mut states: Vec<String> = states_by_node
            .get(&node_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_else(Vec::new);
        let mut edges: Vec<Edge> = edges_by_node
            .get(&node_id)
            .map(|e| e.iter().cloned().collect())
            .unwrap_or_else(Vec::new);

        states.sort_unstable();
        edges.sort_unstable_by(|l, r| match l.from.cmp(&r.from) {
            std::cmp::Ordering::Equal => match l.to.cmp(&r.to) {
                std::cmp::Ordering::Equal => l.transition.cmp(&r.transition),
                o => o,
            },
            o => o,
        });

        nodes.insert(node_id, NodeGraph { states, edges });
    }

    EventGraph { nodes }
}

pub fn dot_for_node_graph(node: &NodeGraph) -> String {
    let mut grouped: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for e in &node.edges {
        grouped
            .entry((e.from.clone(), e.to.clone()))
            .or_default()
            .push(e.transition.clone());
    }

    for labels in grouped.values_mut() {
        labels.sort_unstable();
    }

    let mut out = String::new();
    out.push_str("digraph G{\n");
    out.push_str("    rankdir=LR;\n");
    out.push_str("    node [shape=circle, fontsize=10, width=0.5];\n");
    out.push_str("    edge [fontsize=9];\n");

    for s in &node.states {
        out.push_str("    \"");
        out.push_str(s);
        out.push_str("\";\n");
    }

    for ((from, to), labels) in grouped {
        let label_str = labels.join("<BR/>");
        out.push_str("    \"");
        out.push_str(&from);
        out.push_str("\" -> \"");
        out.push_str(&to);
        out.push_str("\" [label=<");
        out.push_str(&label_str);
        out.push_str(">];\n");
    }

    out.push_str("}\n");
    out
}

pub fn dot_for_node(graph: &EventGraph, node_id: u32) -> Option<String> {
    graph
        .nodes
        .get(&node_id)
        .map(|node| dot_for_node_graph(node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_cyclic_graph_with_four_states() {
        let log = "1 init idle
1 start active
1 process busy
1 complete ready
1 reset idle";
        let g = build_event_graph(log);
        let node = g.nodes.get(&1).unwrap();

        assert_eq!(node.states.len(), 4);
        assert!(node.states.contains(&"idle".to_string()));
        assert!(node.states.contains(&"active".to_string()));
        assert!(node.states.contains(&"busy".to_string()));
        assert!(node.states.contains(&"ready".to_string()));

        assert_eq!(node.edges.len(), 4);
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "idle" && e.transition == "init" && e.to == "active")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "active" && e.transition == "start" && e.to == "busy")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "busy" && e.transition == "process" && e.to == "ready")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "ready" && e.transition == "complete" && e.to == "idle")
        );

        assert!(
            !node
                .edges
                .iter()
                .any(|e| e.from == "active" && e.to == "ready")
        );
        assert!(
            !node
                .edges
                .iter()
                .any(|e| e.from == "idle" && e.to == "busy")
        );
    }

    #[test]
    fn builds_complex_non_cyclic_graph_with_five_states() {
        let log = "1 boot startup
1 load config
1 validate ready
1 connect network
1 auth authenticated
1 error network
1 retry authenticated
1 shutdown config
1 cleanup startup";
        let g = build_event_graph(log);
        let node = g.nodes.get(&1).unwrap();

        assert_eq!(node.states.len(), 5);
        assert!(node.states.contains(&"startup".to_string()));
        assert!(node.states.contains(&"config".to_string()));
        assert!(node.states.contains(&"ready".to_string()));
        assert!(node.states.contains(&"network".to_string()));
        assert!(node.states.contains(&"authenticated".to_string()));

        assert_eq!(node.edges.len(), 8);
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "startup" && e.transition == "boot" && e.to == "config")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "config" && e.transition == "load" && e.to == "ready")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "ready" && e.transition == "validate" && e.to == "network")
        );
        assert!(
            node.edges.iter().any(|e| e.from == "network"
                && e.transition == "connect"
                && e.to == "authenticated")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "authenticated" && e.transition == "auth" && e.to == "network")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "network" && e.transition == "error" && e.to == "authenticated")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "authenticated" && e.transition == "retry" && e.to == "config")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "config" && e.transition == "shutdown" && e.to == "startup")
        );
    }

    #[test]
    fn builds_separate_graphs_for_different_node_ids() {
        let log = "1 start idle
2 init stopped
1 work active
2 boot running
1 finish idle
2 halt stopped";
        let g = build_event_graph(log);

        let n1 = g.nodes.get(&1).unwrap();
        assert_eq!(n1.states.len(), 2);
        assert!(n1.states.contains(&"idle".to_string()));
        assert!(n1.states.contains(&"active".to_string()));
        assert_eq!(n1.edges.len(), 2);
        assert!(
            n1.edges
                .iter()
                .any(|e| e.from == "idle" && e.transition == "start" && e.to == "active")
        );
        assert!(
            n1.edges
                .iter()
                .any(|e| e.from == "active" && e.transition == "work" && e.to == "idle")
        );

        let n2 = g.nodes.get(&2).unwrap();
        assert_eq!(n2.states.len(), 2);
        assert!(n2.states.contains(&"stopped".to_string()));
        assert!(n2.states.contains(&"running".to_string()));
        assert_eq!(n2.edges.len(), 2);
        assert!(
            n2.edges
                .iter()
                .any(|e| e.from == "stopped" && e.transition == "init" && e.to == "running")
        );
        assert!(
            n2.edges
                .iter()
                .any(|e| e.from == "running" && e.transition == "boot" && e.to == "stopped")
        );

        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn skips_invalid_lines_and_continues_processing() {
        let log = "1 start idle
invalid line with no numbers
1 work active
not enough parts
too many parts here now
X invalid_id active
1 finish idle
   
1 restart active";
        let g = build_event_graph(log);
        let node = g.nodes.get(&1).unwrap();

        assert_eq!(node.states.len(), 2);
        assert!(node.states.contains(&"idle".to_string()));
        assert!(node.states.contains(&"active".to_string()));

        assert_eq!(node.edges.len(), 3);
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "idle" && e.transition == "start" && e.to == "active")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "active" && e.transition == "work" && e.to == "idle")
        );
        assert!(
            node.edges
                .iter()
                .any(|e| e.from == "idle" && e.transition == "finish" && e.to == "active")
        );
    }

    #[test]
    fn renders_dot_for_node_graph() {
        let log = "1 A s0\n1 B s1\n1 C s0\n";
        let g = build_event_graph(log);
        let dot = dot_for_node_graph(g.nodes.get(&1).unwrap());
        assert!(dot.contains("digraph G{"));
        assert!(dot.contains("rankdir=LR;"));
        assert!(dot.contains("\"s0\";"));
        assert!(dot.contains("\"s1\";"));
        assert!(dot.contains("\"s0\" -> \"s1\" [label=<A>];"));
        assert!(dot.contains("\"s1\" -> \"s0\" [label=<B>];"));
    }

    #[test]
    fn renders_dot_labels_grouped_and_sorted() {
        let log = "1 A s0\n1 B s1\n1 C s0\n1 D s1\n1 E s0\n";
        let g = build_event_graph(log);
        let dot = dot_for_node_graph(g.nodes.get(&1).unwrap());
        assert!(dot.contains("\"s0\" -> \"s1\" [label=<A<BR/>C>];"));
        assert!(dot.contains("\"s1\" -> \"s0\" [label=<B<BR/>D>];"));
    }
}
