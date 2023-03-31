use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

pub type Id = u16;
pub type Flow = i32;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Edge {
    pub start: Id,
    pub end: Id,
}

impl Debug for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.start, self.end)
    }
}

pub fn edge(start: Id, end: Id) -> Edge {
    Edge { start, end }
}

#[derive(Debug)]
pub struct FlowNetwork {
    edges: HashSet<Edge>,
    capacities: HashMap<Edge, Flow>,
    flows: HashMap<Edge, Flow>,
    outgoing_edges: HashMap<Id, HashSet<Edge>>,
    incoming_edges: HashMap<Id, HashSet<Edge>>,
    source: Id,
    sink: Id,
    empty: HashSet<Edge>, // FIXME: Replace this with something better.
}

impl FlowNetwork {
    pub fn empty(source: Id, sink: Id) -> FlowNetwork {
        FlowNetwork {
            edges: HashSet::new(),
            capacities: HashMap::new(),
            flows: HashMap::new(),
            outgoing_edges: HashMap::new(),
            incoming_edges: HashMap::new(),
            source,
            sink,
            empty: HashSet::new(),
        }
    }

    pub fn add_edge(&mut self, start: Id, end: Id, capacity: Flow, flow: Flow) {
        let edge = edge(start, end);
        self.edges.insert(edge);
        self.capacities.insert(edge, capacity);
        self.flows.insert(edge, flow);

        if !self.outgoing_edges.contains_key(&start) {
            self.outgoing_edges.insert(start, HashSet::new());
        }
        self.outgoing_edges.get_mut(&start).unwrap().insert(edge);

        if !self.incoming_edges.contains_key(&end) {
            self.incoming_edges.insert(end, HashSet::new());
        }
        self.incoming_edges.get_mut(&end).unwrap().insert(edge);
    }

    pub fn remove_edge(&mut self, start: Id, end: Id) {
        let edge = edge(start, end);
        self.edges.remove(&edge);
        self.capacities.remove(&edge);
        self.flows.remove(&edge);

        self.outgoing_edges.get_mut(&start).unwrap().remove(&edge);
        if self.outgoing_edges.get(&start).unwrap().is_empty() {
            self.outgoing_edges.remove(&start);
        }

        self.incoming_edges.get_mut(&end).unwrap().remove(&edge);
        if self.incoming_edges.get(&end).unwrap().is_empty() {
            self.incoming_edges.remove(&end);
        }
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.capacities.clear();
        self.flows.clear();
        self.outgoing_edges.clear();
        self.incoming_edges.clear();
    }

    pub fn source(&self) -> Id {
        self.source
    }

    pub fn sink(&self) -> Id {
        self.sink
    }

    pub fn edges(&self) -> &HashSet<Edge> {
        &self.edges
    }

    pub fn flows(&self) -> &HashMap<Edge, Flow> {
        &self.flows
    }

    pub fn flows_mut(&mut self) -> &mut HashMap<Edge, Flow> {
        &mut self.flows
    }

    pub fn capacities(&self) -> &HashMap<Edge, Flow> {
        &self.capacities
    }

    pub fn capacity(&self, edge: Edge) -> Flow {
        *self.capacities.get(&edge).unwrap_or(&0)
    }

    pub fn flow(&self, edge: Edge) -> Flow {
        *self.flows.get(&edge).unwrap_or(&0)
    }

    pub fn outgoing_edges(&self, vertex: Id) -> &HashSet<Edge> {
        self.outgoing_edges.get(&vertex).unwrap_or(&self.empty)
    }
}

mod tests {
    use std::collections::{HashMap, HashSet};

    use map_macro::{map, set};

    use super::{edge, FlowNetwork};

    #[test]
    fn empty() {
        let network = FlowNetwork::empty(0, 1);
        assert_eq!(network.source, 0);
        assert_eq!(network.sink, 1);
        assert_eq!(network.edges, HashSet::new());
        assert_eq!(network.capacities, HashMap::new());
        assert_eq!(network.flows, HashMap::new());
        assert_eq!(network.outgoing_edges, HashMap::new());
        assert_eq!(network.incoming_edges, HashMap::new());
    }

    #[test]
    fn adding_edges() {
        let mut network = FlowNetwork::empty(0, 3);
        network.add_edge(0, 1, 5, 0);
        network.add_edge(1, 3, 19, 10);
        network.add_edge(0, 2, 3, 3);
        network.add_edge(2, 3, 0, 0);
        network.add_edge(1, 2, 3, 0);

        assert_eq!(network.source, 0);
        assert_eq!(network.sink, 3);

        assert_eq!(
            network.edges,
            set! {
                edge(0, 1),
                edge(1, 3),
                edge(0, 2),
                edge(2, 3),
                edge(1, 2),
            }
        );

        assert_eq!(
            network.capacities,
            map! {
                edge(0, 1) => 5,
                edge(1, 3) => 19,
                edge(0, 2) => 3,
                edge(2, 3) => 0,
                edge(1, 2) => 3,
            }
        );

        assert_eq!(
            network.flows,
            map! {
                edge(0, 1) => 0,
                edge(1, 3) => 10,
                edge(0, 2) => 3,
                edge(2, 3) => 0,
                edge(1, 2) => 0,
            }
        );

        assert_eq!(
            network.outgoing_edges,
            map! {
                0 => set!(edge(0, 1), edge(0, 2)),
                1 => set!(edge(1, 3), edge(1, 2)),
                2 => set!(edge(2, 3)),
            }
        );

        assert_eq!(
            network.incoming_edges,
            map! {
                1 => set!(edge(0, 1)),
                2 => set!(edge(0, 2), edge(1, 2)),
                3 => set!(edge(1, 3), edge(2, 3)),
            }
        );
    }

    #[test]
    fn removing_edges() {
        let mut network = FlowNetwork::empty(0, 3);
        network.add_edge(0, 1, 5, 0);
        network.add_edge(1, 3, 19, 10);
        network.add_edge(0, 2, 3, 3);
        network.add_edge(2, 3, 0, 0);
        network.add_edge(1, 2, 3, 0);

        network.remove_edge(0, 2);
        network.remove_edge(1, 3);
        network.remove_edge(2, 3);

        assert_eq!(network.source, 0);
        assert_eq!(network.sink, 3);

        assert_eq!(
            network.edges,
            set! {
                edge(0, 1),
                edge(1, 2),
            }
        );

        assert_eq!(
            network.capacities,
            map! {
                edge(0, 1) => 5,
                edge(1, 2) => 3,
            }
        );

        assert_eq!(
            network.flows,
            map! {
                edge(0, 1) => 0,
                edge(1, 2) => 0,
            }
        );

        assert_eq!(
            network.outgoing_edges,
            map! {
                0 => set!(edge(0, 1)),
                1 => set!(edge(1, 2)),
            }
        );

        assert_eq!(
            network.incoming_edges,
            map! {
                1 => set!(edge(0, 1)),
                2 => set!(edge(1, 2)),
            }
        );
    }
}
