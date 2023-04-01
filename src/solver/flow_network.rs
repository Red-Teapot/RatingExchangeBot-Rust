use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

pub type Id = u16;
pub type Flow = u16;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Edge {
    pub start: Id,
    pub end: Id,
}

impl Edge {
    pub fn opposite(&self) -> Edge {
        edge(self.end, self.start)
    }
}

impl Debug for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.start, self.end)
    }
}

impl From<(Id, Id)> for Edge {
    fn from((start, end): (Id, Id)) -> Self {
        Edge { start, end }
    }
}

pub fn edge(start: Id, end: Id) -> Edge {
    Edge { start, end }
}

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

    pub fn add_edge(&mut self, edge: impl Into<Edge>, capacity: Flow, flow: Flow) {
        let edge = edge.into();
        let start = edge.start;
        let end = edge.end;

        self.edges.insert(edge);
        self.capacities.insert(edge, capacity);
        self.flows.insert(edge, flow);

        self.outgoing_edges
            .entry(start)
            .or_insert_with(HashSet::new);
        self.outgoing_edges.get_mut(&start).unwrap().insert(edge);

        self.incoming_edges.entry(end).or_insert_with(HashSet::new);
        self.incoming_edges.get_mut(&end).unwrap().insert(edge);
    }

    pub fn remove_edge(&mut self, edge: impl Into<Edge>) {
        let edge = edge.into();
        let start = edge.start;
        let end = edge.end;

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

    pub fn outgoing_edges(&self, vertex: Id) -> &HashSet<Edge> {
        self.outgoing_edges.get(&vertex).unwrap_or(&self.empty)
    }

    pub fn incoming_edges(&self, vertex: Id) -> &HashSet<Edge> {
        self.incoming_edges.get(&vertex).unwrap_or(&self.empty)
    }

    pub fn flows(&self) -> &HashMap<Edge, Flow> {
        &self.flows
    }

    pub fn flow(&self, edge: impl Into<Edge>) -> Flow {
        let edge = edge.into();
        *self.flows.get(&edge).unwrap_or(&0)
    }

    pub fn set_flow(&mut self, edge: impl Into<Edge>, flow: Flow) {
        let edge = edge.into();
        assert!(self.edges.contains(&edge));

        self.flows.insert(edge, flow);
    }

    pub fn capacities(&self) -> &HashMap<Edge, Flow> {
        &self.capacities
    }

    pub fn capacity(&self, edge: impl Into<Edge>) -> Flow {
        let edge = edge.into();
        *self.capacities.get(&edge).unwrap_or(&0)
    }

    pub fn available_capacity(&self, edge: impl Into<Edge>) -> Flow {
        let edge = edge.into();
        let capacity = self.capacity(edge);
        let flow = self.flow(edge);

        if flow >= capacity {
            0
        } else {
            capacity - flow
        }
    }

    pub fn validate(&self, expected_total_flow: Option<Flow>) -> Result<(), String> {
        for &edge in self.edges() {
            let capacity = self.capacity(edge);
            let flow = self.flow(edge);

            if flow > capacity {
                return Err(format!(
                    "Flow on {edge:?} exceeds capacity: {flow} > {capacity}"
                ));
            }
        }

        let mut vertices = HashSet::new();
        vertices.extend(self.edges().iter().flat_map(|e| [e.start, e.end]));
        for vertex in vertices {
            if vertex == self.source() || vertex == self.sink() {
                continue;
            }

            let incoming_flow = self
                .incoming_edges(vertex)
                .iter()
                .map(|&e| self.flow(e))
                .sum::<Flow>();
            let outgoing_flow = self
                .outgoing_edges(vertex)
                .iter()
                .map(|&e| self.flow(e))
                .sum::<Flow>();

            if incoming_flow != outgoing_flow {
                return Err(format!("Incoming flow {incoming_flow} on vertex {vertex} does not match outgoing flow {outgoing_flow}"));
            }
        }

        if let Some(total_flow) = expected_total_flow {
            let source_flow = self
                .outgoing_edges(self.source())
                .iter()
                .map(|&e| self.flow(e))
                .sum::<Flow>();
            if source_flow != total_flow {
                return Err(format!(
                    "Source flow {source_flow} does not match total flow {total_flow}"
                ));
            }

            let sink_flow = self
                .incoming_edges(self.sink())
                .iter()
                .map(|&e| self.flow(e))
                .sum::<Flow>();
            if sink_flow != total_flow {
                return Err(format!(
                    "Sink flow {sink_flow} does not match total flow {total_flow}"
                ));
            }
        }

        Ok(())
    }
}

impl Debug for FlowNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut edges = self.edges().iter().copied().collect::<Vec<Edge>>();
        edges.sort_by(|&a, &b| {
            use std::cmp::Ordering::*;

            match a.start.cmp(&b.start) {
                Equal => a.end.cmp(&b.end),
                ord => ord,
            }
        });

        for edge in edges {
            let capacity = self.capacity(edge);
            let flow = self.flow(edge);

            writeln!(f, "{edge:?}\t{flow}/{capacity}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

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
        network.add_edge((0, 1), 5, 0);
        network.add_edge((1, 3), 19, 10);
        network.add_edge((0, 2), 3, 3);
        network.add_edge((2, 3), 0, 0);
        network.add_edge((1, 2), 3, 0);

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
        network.add_edge((0, 1), 5, 0);
        network.add_edge((1, 3), 19, 10);
        network.add_edge((0, 2), 3, 3);
        network.add_edge((2, 3), 0, 0);
        network.add_edge((1, 2), 3, 0);

        network.remove_edge((0, 2));
        network.remove_edge((1, 3));
        network.remove_edge((2, 3));

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
