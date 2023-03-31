use crate::solver::flow_network::{Edge, FlowNetwork, Id};
use std::collections::{HashMap, HashSet, LinkedList};

pub fn solve(network: &mut FlowNetwork) {
    let source = network.source();
    let sink = network.sink();

    // 0. Clear the flow values
    network
        .flows_mut()
        .iter_mut()
        .for_each(|(edge, flow)| *flow = 0);

    let mut residual_graph = FlowNetwork::empty(source, sink);
    let mut level_graph = FlowNetwork::empty(source, sink);
    let mut vertex_levels = HashMap::new();
    let mut worklist = LinkedList::new();
    let mut visited = HashSet::new();
    let mut path = HashSet::new();

    loop {
        construct_residual_graph(&network, &mut residual_graph);

        construct_level_graph(
            &residual_graph,
            &mut level_graph,
            &mut vertex_levels,
            &mut worklist,
            &mut visited,
        );

        {
            loop {
                worklist.clear();
                visited.clear();
                path.clear();
                level_graph
                    .outgoing_edges(source)
                    .iter()
                    .for_each(|&e| worklist.push_front(e));

                while let Some(edge) = worklist.pop_front() {
                    if level_graph.flow(edge) >= level_graph.capacity(edge) {
                        continue;
                    }

                    path.insert(edge);

                    if edge.end == level_graph.sink() {
                        break;
                    }

                    let mut should_retreat = true;
                    let mut outgoing_edges = level_graph.outgoing_edges(edge.end);
                    if !outgoing_edges.is_empty() {
                        outgoing_edges.iter().for_each(|&e| {
                            if visited.insert(e) {
                                worklist.push_front(e);
                                should_retreat = false;
                            }
                        });
                    }

                    if should_retreat {
                        path.remove(&edge);
                        level_graph.remove_edge(edge.start, edge.end);
                    }
                }

                dbg!(&path);

                if path.is_empty() {
                    break;
                }

                let path_flow = path
                    .iter()
                    .map(|&e| level_graph.capacity(e) - level_graph.flow(e))
                    .min()
                    .unwrap();
                for &edge in &path {
                    let flow = level_graph.flow(edge);
                    let capacity = level_graph.capacity(edge);
                    level_graph.flows_mut().insert(edge, flow + path_flow);
                    network.flows_mut().insert(edge, flow + path_flow);
                    if flow + path_flow >= capacity {
                        level_graph.remove_edge(edge.start, edge.end);
                    }
                }
            }
        }

        dbg!(network.flows());
    }
}

fn construct_residual_graph(network: &FlowNetwork, residual_graph: &mut FlowNetwork) {
    residual_graph.clear();

    for &edge in network.edges() {
        let capacity = network.capacity(edge);
        let flow = network.flow(edge);

        if capacity - flow > 0 {
            residual_graph.add_edge(edge.start, edge.end, capacity - flow, 0);
        }

        if flow > 0 {
            residual_graph.add_edge(edge.end, edge.start, flow, 0);
        }
    }
}

fn construct_level_graph(
    residual_graph: &FlowNetwork,
    level_graph: &mut FlowNetwork,
    vertex_levels: &mut HashMap<Id, u16>,
    worklist: &mut LinkedList<Edge>,
    visited: &mut HashSet<Edge>,
) {
    let source = residual_graph.source();

    level_graph.clear();
    vertex_levels.clear();
    worklist.clear();
    visited.clear();

    residual_graph
        .outgoing_edges(source)
        .iter()
        .for_each(|&e| worklist.push_back(e));
    vertex_levels.insert(source, 0u16);

    while let Some(edge) = worklist.pop_front() {
        let start = edge.start;
        let end = edge.end;
        let level = *vertex_levels.get(&start).unwrap() + 1;

        let add_edge = match vertex_levels.get(&end) {
            Some(&previous_level) if level <= previous_level => true,
            None => true,
            _ => false,
        };

        if add_edge {
            vertex_levels.insert(end, level);
            level_graph.add_edge(
                start,
                end,
                residual_graph.capacity(edge),
                residual_graph.flow(edge),
            );
        }

        residual_graph.outgoing_edges(end).iter().for_each(|&e| {
            if visited.insert(e) {
                worklist.push_back(e);
            }
        });
    }
}

mod tests {
    use std::collections::{HashMap, HashSet, LinkedList};

    use map_macro::map;

    use crate::solver::flow_network::{edge, FlowNetwork};

    use super::{construct_level_graph, construct_residual_graph, solve};

    // Wikipedia tests are taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example

    #[test]
    fn wikipedia_residual_1() {
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge(0, 1, 10, 0);
        network.add_edge(0, 2, 10, 0);
        network.add_edge(1, 2, 2, 0);
        network.add_edge(1, 3, 4, 0);
        network.add_edge(1, 4, 8, 0);
        network.add_edge(2, 4, 9, 0);
        network.add_edge(3, 5, 10, 0);
        network.add_edge(4, 3, 6, 0);
        network.add_edge(4, 5, 10, 0);

        let mut residual = FlowNetwork::empty(0, 5);
        construct_residual_graph(&network, &mut residual);

        assert_eq!(
            residual.capacities(),
            &map! {
                edge(0, 1) => 10,
                edge(0, 2) => 10,
                edge(1, 2) =>  2,
                edge(1, 3) =>  4,
                edge(1, 4) =>  8,
                edge(2, 4) =>  9,
                edge(3, 5) => 10,
                edge(4, 3) =>  6,
                edge(4, 5) => 10,
            }
        );
    }

    #[test]
    fn wikipedia_residual_2() {
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge(0, 1, 10, 10);
        network.add_edge(0, 2, 10, 4);
        network.add_edge(1, 2, 2, 0);
        network.add_edge(1, 3, 4, 4);
        network.add_edge(1, 4, 8, 6);
        network.add_edge(2, 4, 9, 4);
        network.add_edge(3, 5, 10, 4);
        network.add_edge(4, 3, 6, 0);
        network.add_edge(4, 5, 10, 10);

        let mut residual = FlowNetwork::empty(0, 5);
        construct_residual_graph(&network, &mut residual);

        assert_eq!(
            residual.capacities(),
            &map! {
                edge(0, 2) => 6,
                edge(1, 0) => 10,
                edge(1, 2) => 2,
                edge(1, 4) => 2,
                edge(2, 0) => 4,
                edge(2, 4) => 5,
                edge(3, 1) => 4,
                edge(3, 5) => 6,
                edge(4, 1) => 6,
                edge(4, 2) => 4,
                edge(4, 3) => 6,
                edge(5, 3) => 4,
                edge(5, 4) => 10,
            }
        );
    }

    #[test]
    fn wikipedia_residual_3() {
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge(0, 1, 10, 10);
        network.add_edge(0, 2, 10, 9);
        network.add_edge(1, 2, 2, 0);
        network.add_edge(1, 3, 4, 4);
        network.add_edge(1, 4, 8, 6);
        network.add_edge(2, 4, 9, 9);
        network.add_edge(3, 5, 10, 9);
        network.add_edge(4, 3, 6, 5);
        network.add_edge(4, 5, 10, 10);

        let mut residual = FlowNetwork::empty(0, 5);
        construct_residual_graph(&network, &mut residual);

        assert_eq!(
            residual.capacities(),
            &map! {
                edge(0, 2) => 1,
                edge(1, 0) => 10,
                edge(1, 2) => 2,
                edge(1, 4) => 2,
                edge(2, 0) => 9,
                edge(3, 1) => 4,
                edge(3, 4) => 5,
                edge(3, 5) => 1,
                edge(4, 1) => 6,
                edge(4, 2) => 9,
                edge(4, 3) => 1,
                edge(5, 3) => 9,
                edge(5, 4) => 10,
            }
        );
    }

    #[test]
    fn wikipedia_levels_1() {
        let mut residual = FlowNetwork::empty(0, 5);
        residual.add_edge(0, 1, 10, 0);
        residual.add_edge(0, 2, 10, 0);
        residual.add_edge(1, 2, 2, 0);
        residual.add_edge(1, 3, 4, 0);
        residual.add_edge(1, 4, 8, 0);
        residual.add_edge(2, 4, 9, 0);
        residual.add_edge(3, 5, 10, 0);
        residual.add_edge(4, 3, 6, 0);
        residual.add_edge(4, 5, 10, 0);

        let mut level_graph = FlowNetwork::empty(0, 5);
        let mut levels = HashMap::new();
        let mut worklist = LinkedList::new();
        let mut visited = HashSet::new();

        construct_level_graph(
            &residual,
            &mut level_graph,
            &mut levels,
            &mut worklist,
            &mut visited,
        );

        assert_eq!(
            level_graph.capacities(),
            &map! {
                edge(0, 1) => 10,
                edge(0, 2) => 10,
                edge(1, 3) =>  4,
                edge(1, 4) =>  8,
                edge(2, 4) =>  9,
                edge(3, 5) => 10,
                edge(4, 5) => 10,
            }
        );

        assert_eq!(
            &levels,
            &map! {
                0 => 0,
                1 => 1,
                2 => 1,
                3 => 2,
                4 => 2,
                5 => 3,
            }
        );
    }

    #[test]
    fn wikipedia_levels_2() {
        let mut residual = FlowNetwork::empty(0, 5);
        residual.add_edge(0, 2, 6, 0);
        residual.add_edge(1, 0, 10, 0);
        residual.add_edge(1, 2, 2, 0);
        residual.add_edge(1, 4, 2, 0);
        residual.add_edge(2, 0, 4, 0);
        residual.add_edge(2, 4, 5, 0);
        residual.add_edge(3, 1, 4, 0);
        residual.add_edge(3, 5, 6, 0);
        residual.add_edge(4, 1, 6, 0);
        residual.add_edge(4, 2, 4, 0);
        residual.add_edge(4, 3, 6, 0);
        residual.add_edge(5, 3, 4, 0);
        residual.add_edge(5, 4, 10, 0);

        let mut level_graph = FlowNetwork::empty(0, 5);
        let mut levels = HashMap::new();
        let mut worklist = LinkedList::new();
        let mut visited = HashSet::new();

        construct_level_graph(
            &residual,
            &mut level_graph,
            &mut levels,
            &mut worklist,
            &mut visited,
        );

        assert_eq!(
            level_graph.capacities(),
            &map! {
                edge(0, 2) => 6,
                edge(2, 4) => 5,
                edge(4, 1) => 6,
                edge(4, 3) => 6,
                edge(3, 5) => 6,
            }
        );

        assert_eq!(
            &levels,
            &map! {
                0 => 0,
                1 => 3,
                2 => 1,
                3 => 3,
                4 => 2,
                5 => 4,
            }
        );
    }

    #[test]
    fn wikipedia_levels_3() {
        let mut residual = FlowNetwork::empty(0, 5);
        residual.add_edge(0, 2, 1, 0);
        residual.add_edge(1, 0, 10, 0);
        residual.add_edge(1, 2, 2, 0);
        residual.add_edge(1, 4, 2, 0);
        residual.add_edge(2, 0, 9, 0);
        residual.add_edge(3, 1, 4, 0);
        residual.add_edge(3, 4, 5, 0);
        residual.add_edge(3, 5, 1, 0);
        residual.add_edge(4, 1, 6, 0);
        residual.add_edge(4, 2, 9, 0);
        residual.add_edge(4, 3, 1, 0);
        residual.add_edge(5, 3, 9, 0);
        residual.add_edge(5, 4, 10, 0);

        let mut level_graph = FlowNetwork::empty(0, 5);
        let mut levels = HashMap::new();
        let mut worklist = LinkedList::new();
        let mut visited = HashSet::new();

        construct_level_graph(
            &residual,
            &mut level_graph,
            &mut levels,
            &mut worklist,
            &mut visited,
        );

        assert_eq!(
            level_graph.capacities(),
            &map! {
                edge(0, 2) => 1,
            }
        );

        assert_eq!(
            &levels,
            &map! {
                0 => 0,
                2 => 1,
            }
        );
    }

    //#[test]
    fn wikipedia_solve() {
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge(0, 1, 10, 0);
        network.add_edge(0, 2, 10, 0);
        network.add_edge(1, 2, 2, 0);
        network.add_edge(1, 4, 8, 0);
        network.add_edge(1, 3, 4, 0);
        network.add_edge(2, 4, 9, 0);
        network.add_edge(3, 5, 10, 0);
        network.add_edge(4, 3, 6, 0);
        network.add_edge(4, 5, 10, 0);

        solve(&mut network);

        assert_eq!(
            network.flows(),
            &map! {
                edge(0, 1) => 10,
                edge(0, 2) => 9,
                edge(1, 2) => 0,
                edge(1, 4) => 6,
                edge(2, 4) => 9,
                edge(3, 5) => 9,
                edge(4, 3) => 5,
                edge(4, 5) => 10,
            }
        );
    }
}
