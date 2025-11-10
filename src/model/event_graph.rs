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

fn parse_mediator_log_line(line: &str) -> Option<EventRecord> {
    let relevant_part = if line.contains("[Node ") {
        let node_pos = line.find("[Node ")?;
        &line[node_pos..]
    } else {
        return None;
    };

    let node_start = 6;
    let node_end = relevant_part[node_start..].find(' ')?;
    let node_id: u32 = relevant_part[node_start..node_start + node_end]
        .parse()
        .ok()?;

    let function_marker = "@ functionName ";
    let function_start = relevant_part.find(function_marker)? + function_marker.len();
    let function_end = relevant_part[function_start..].find(" @ ")?;
    let transition = relevant_part[function_start..function_start + function_end].to_string();

    let state = if let Some(state_marker_pos) = relevant_part.find("@ state ") {
        let state_start = state_marker_pos + 8;
        relevant_part[state_start..].trim().to_string()
    } else if let Some(const_marker_pos) = relevant_part.find("@ constant ") {
        let const_start = const_marker_pos + 11;
        relevant_part[const_start..].trim().to_string()
    } else {
        return None;
    };

    Some(EventRecord {
        node_id,
        transition,
        state,
    })
}

pub fn build_event_graph(log: &str) -> EventGraph {
    let mut records: Vec<EventRecord> = Vec::new();
    let mut states_by_node: HashMap<u32, HashSet<String>> = HashMap::new();

    for line in log.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(record) = parse_mediator_log_line(trimmed) {
            states_by_node
                .entry(record.node_id)
                .or_default()
                .insert(record.state.clone());
            records.push(record);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_cyclic_graph_with_four_states() {
        let log = "[2025-11-10 19:56:55.000001][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 1 / 5] FunctionExecute 1 @ functionName init @ state idle
[2025-11-10 19:56:55.000002][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 2 / 5] FunctionExecute 2 @ functionName start @ state active
[2025-11-10 19:56:55.000003][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 3 / 5] FunctionExecute 3 @ functionName process @ state busy
[2025-11-10 19:56:55.000004][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 4 / 5] FunctionExecute 4 @ functionName complete @ state ready
[2025-11-10 19:56:55.000005][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 5 / 5] FunctionExecute 5 @ functionName reset @ state idle";
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
        let log = "[2025-11-10 19:56:55.000001][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 1 / 9] BlockExecute 1 @ functionName boot @ state startup
[2025-11-10 19:56:55.000002][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 2 / 9] BlockExecute 2 @ functionName load @ state config
[2025-11-10 19:56:55.000003][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 3 / 9] BlockExecute 3 @ functionName validate @ state ready
[2025-11-10 19:56:55.000004][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 4 / 9] BlockExecute 4 @ functionName connect @ state network
[2025-11-10 19:56:55.000005][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 5 / 9] BlockExecute 5 @ functionName auth @ state authenticated
[2025-11-10 19:56:55.000006][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 6 / 9] BlockExecute 6 @ functionName error @ state network
[2025-11-10 19:56:55.000007][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 7 / 9] BlockExecute 7 @ functionName retry @ state authenticated
[2025-11-10 19:56:55.000008][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 8 / 9] BlockExecute 8 @ functionName shutdown @ state config
[2025-11-10 19:56:55.000009][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 1 Entry 9 / 9] BlockExecute 9 @ functionName cleanup @ state startup";
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
        let log = "[2025-11-10 19:56:55.000001][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 1 / 2] FunctionExecute 1 @ functionName start @ state idle
[2025-11-10 19:56:55.000002][INFO] [FUNC_EVENT_TYPE][Node 2 Batch 1 Entry 1 / 2] FunctionExecute 2 @ functionName init @ state stopped
[2025-11-10 19:56:55.000003][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 2 / 2] FunctionExecute 3 @ functionName work @ state active
[2025-11-10 19:56:55.000004][INFO] [FUNC_EVENT_TYPE][Node 2 Batch 1 Entry 2 / 2] FunctionExecute 4 @ functionName boot @ state running
[2025-11-10 19:56:55.000005][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 2 Entry 1 / 1] FunctionExecute 5 @ functionName finish @ state idle
[2025-11-10 19:56:55.000006][INFO] [FUNC_EVENT_TYPE][Node 2 Batch 2 Entry 1 / 1] FunctionExecute 6 @ functionName halt @ state stopped";
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
        let log = "[2025-11-10 19:56:55.000001][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 1 / 4] FunctionExecute 1 @ functionName start @ state idle
invalid line with no numbers
[2025-11-10 19:56:55.000002][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 2 / 4] FunctionExecute 2 @ functionName work @ state active
not enough parts
too many parts here now
[2025-11-10 19:56:55.000003][INFO] [FUNC_EVENT_TYPE][Node X Batch 1 Entry 3 / 4] FunctionExecute 3 @ functionName invalid_id @ state active
[2025-11-10 19:56:55.000004][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 3 / 4] FunctionExecute 4 @ functionName finish @ state idle
   
[2025-11-10 19:56:55.000005][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 4 / 4] FunctionExecute 5 @ functionName restart @ state active";
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
        let log = "[2025-11-10 19:56:55.000001][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 1 / 3] FunctionExecute 1 @ functionName A @ state s0
[2025-11-10 19:56:55.000002][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 2 / 3] FunctionExecute 2 @ functionName B @ state s1
[2025-11-10 19:56:55.000003][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 3 / 3] FunctionExecute 3 @ functionName C @ state s0";
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
        let log = "[2025-11-10 19:56:55.000001][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 1 / 5] FunctionExecute 1 @ functionName A @ state s0
[2025-11-10 19:56:55.000002][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 2 / 5] FunctionExecute 2 @ functionName B @ state s1
[2025-11-10 19:56:55.000003][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 3 / 5] FunctionExecute 3 @ functionName C @ state s0
[2025-11-10 19:56:55.000004][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 4 / 5] FunctionExecute 4 @ functionName D @ state s1
[2025-11-10 19:56:55.000005][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 1 Entry 5 / 5] FunctionExecute 5 @ functionName E @ state s0";
        let g = build_event_graph(log);
        let dot = dot_for_node_graph(g.nodes.get(&1).unwrap());
        assert!(dot.contains("\"s0\" -> \"s1\" [label=<A<BR/>C>];"));
        assert!(dot.contains("\"s1\" -> \"s0\" [label=<B<BR/>D>];"));
    }

    #[test]
    fn parses_mediator_log_format() {
        let log = "[2025-11-10 19:56:55.000001][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 2 Entry 1 / 3] BlockExecute 42 @ functionName init @ state idle
[2025-11-10 19:56:55.000002][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 2 Entry 2 / 3] FunctionExecute 43 @ functionName process @ state active
[2025-11-10 19:56:55.000003][INFO] [CONST_EVENT_TYPE][Node 1 Batch 2 Entry 3 / 3] ConstantExecute 44 @ functionName reset @ constant idle
[2025-11-10 19:56:55.000004][INFO] [BLOCK_EVENT_TYPE][Node 2 Batch 3 Entry 1 / 1] BlockExecute 50 @ functionName start @ state running";
        let g = build_event_graph(log);

        let n1 = g.nodes.get(&1).unwrap();
        assert_eq!(n1.states.len(), 2);
        assert!(n1.states.contains(&"idle".to_string()));
        assert!(n1.states.contains(&"active".to_string()));
        assert_eq!(n1.edges.len(), 2);
        assert!(
            n1.edges
                .iter()
                .any(|e| e.from == "idle" && e.transition == "init" && e.to == "active")
        );
        assert!(
            n1.edges
                .iter()
                .any(|e| e.from == "active" && e.transition == "process" && e.to == "idle")
        );

        let n2 = g.nodes.get(&2).unwrap();
        assert_eq!(n2.states.len(), 1);
        assert!(n2.states.contains(&"running".to_string()));
        assert_eq!(n2.edges.len(), 0);

        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn skips_non_event_log_lines_in_mediator_format() {
        let log = "[2025-11-10 19:56:55.000001][INFO] Some informational message
[2025-11-10 19:56:55.000002][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 2 Entry 1 / 2] BlockExecute 42 @ functionName init @ state idle
[2025-11-10 19:56:55.000003][DEBUG] Debug message without relevant data
[2025-11-10 19:56:55.000004][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 2 Entry 2 / 2] FunctionExecute 43 @ functionName process @ state active
Random log line without brackets
[2025-11-10 19:56:55.000005][ERROR] Error message";
        let g = build_event_graph(log);

        let n1 = g.nodes.get(&1).unwrap();
        assert_eq!(n1.states.len(), 2);
        assert!(n1.states.contains(&"idle".to_string()));
        assert!(n1.states.contains(&"active".to_string()));
        assert_eq!(n1.edges.len(), 1);
        assert!(
            n1.edges
                .iter()
                .any(|e| e.from == "idle" && e.transition == "init" && e.to == "active")
        );

        assert_eq!(g.nodes.len(), 1);
    }

    #[test]
    fn parses_mediator_log_with_timestamps_and_log_levels() {
        let log = "[2025-11-10 19:56:55.662761][INFO] [BLOCK_EVENT_TYPE][Node 1 Batch 68 Entry 51 / 56] BlockExecute 94 @ functionName sendAppendEntries @ state Leader
[2025-11-10 19:56:55.662800][INFO] [FUNC_EVENT_TYPE][Node 1 Batch 68 Entry 52 / 56] FunctionExecute 95 @ functionName processResponse @ state Follower
[2025-11-10 19:56:55.663000][DEBUG] [CONST_EVENT_TYPE][Node 2 Batch 10 Entry 1 / 1] ConstantExecute 10 @ functionName timeout @ constant Candidate";
        let g = build_event_graph(log);

        let n1 = g.nodes.get(&1).unwrap();
        assert_eq!(n1.states.len(), 2);
        assert!(n1.states.contains(&"Leader".to_string()));
        assert!(n1.states.contains(&"Follower".to_string()));
        assert_eq!(n1.edges.len(), 1);
        assert!(n1.edges.iter().any(|e| e.from == "Leader"
            && e.transition == "sendAppendEntries"
            && e.to == "Follower"));

        let n2 = g.nodes.get(&2).unwrap();
        assert_eq!(n2.states.len(), 1);
        assert!(n2.states.contains(&"Candidate".to_string()));
        assert_eq!(n2.edges.len(), 0);

        assert_eq!(g.nodes.len(), 2);
    }
}
