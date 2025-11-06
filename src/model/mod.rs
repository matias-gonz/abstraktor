pub mod event_graph;
pub mod instrumentor;

pub use event_graph::{
    Edge, EventGraph, NodeGraph, build_event_graph, dot_for_node, dot_for_node_graph,
};
