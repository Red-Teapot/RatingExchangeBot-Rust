use tracing::{trace, warn};

use crate::solver::flow_network::{Edge, FlowNetwork, Id};
use std::collections::{HashMap, HashSet, LinkedList};

pub fn solve(network: &mut FlowNetwork) {
    let source = network.source();
    let sink = network.sink();

    let mut residual_graph = FlowNetwork::empty(source, sink);
    let mut level_graph = FlowNetwork::empty(source, sink);
    let mut vertex_levels = HashMap::new();
    let mut worklist = LinkedList::new();
    let mut visited = HashSet::new();
    let mut path = LinkedList::new();

    loop {
        construct_residual_graph(network, &mut residual_graph);

        trace!("Residual graph:\n{residual_graph:?}");

        construct_level_graph(
            &residual_graph,
            &mut level_graph,
            &mut vertex_levels,
            &mut worklist,
            &mut visited,
        );

        trace!("Level graph:\n{level_graph:?}");
        trace!("Vertex levels:\n{vertex_levels:?}");

        let has_blocking_flow =
            find_blocking_flow(&mut level_graph, &mut worklist, &mut visited, &mut path);

        trace!("Has blocking flow: {}", has_blocking_flow);
        trace!("Level graph with blocking flow:\n{level_graph:?}");

        if !has_blocking_flow {
            break;
        }

        for (&edge, &flow) in level_graph.flows() {
            if flow == 0 {
                continue;
            }

            if network.edges().contains(&edge) {
                let orig_flow = network.flow(edge);
                network.set_flow(edge, orig_flow + flow);
            } else if network.edges().contains(&edge.opposite()) {
                let opposite = edge.opposite();
                let orig_flow = network.flow(opposite);
                network.set_flow(opposite, orig_flow - flow);
            } else {
                warn!("Edge {edge:?} from level graph does not exist in network.\nNetwork:\n{network:?}Level graph:\n{level_graph:?}");
            }
        }

        trace!("Network after flow adjustment:\n{network:?}");
    }
}

fn construct_residual_graph(network: &FlowNetwork, residual_graph: &mut FlowNetwork) {
    residual_graph.clear();

    for &edge in network.edges() {
        let capacity = network.capacity(edge);
        let flow = network.flow(edge);

        if capacity > flow {
            residual_graph.add_edge(edge, capacity - flow, 0);
        }

        if flow > 0 {
            residual_graph.add_edge(edge.opposite(), flow, 0);
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
                edge,
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

fn find_blocking_flow(
    level_graph: &mut FlowNetwork,
    worklist: &mut LinkedList<Edge>,
    visited: &mut HashSet<Edge>,
    path: &mut LinkedList<Edge>,
) -> bool {
    let source = level_graph.source();

    let mut reached_sink_once = false;

    loop {
        let mut reached_sink = false;

        worklist.clear();
        visited.clear();
        path.clear();

        level_graph
            .outgoing_edges(source)
            .iter()
            .for_each(|&e| worklist.push_front(e));

        while let Some(edge) = worklist.pop_front() {
            while let Some(tail) = path.back() {
                if tail.end == edge.start {
                    break;
                } else {
                    path.pop_back();
                }
            }

            path.push_back(edge);

            if edge.end == level_graph.sink() {
                reached_sink = true;
                reached_sink_once = true;
                break;
            }

            let mut should_retreat = true;
            level_graph.outgoing_edges(edge.end).iter().for_each(|&e| {
                if level_graph.available_capacity(e) > 0 && visited.insert(e) {
                    worklist.push_front(e);
                    should_retreat = false;
                }
            });

            if should_retreat {
                assert_eq!(path.pop_back().unwrap(), edge);
            }
        }

        if !reached_sink {
            break;
        }

        let path_flow = path
            .iter()
            .map(|&e| level_graph.available_capacity(e))
            .min()
            .unwrap();

        if path_flow == 0 {
            break;
        }

        for &edge in path.iter() {
            let flow = level_graph.flow(edge);
            level_graph.set_flow(edge, flow + path_flow);
        }
    }

    reached_sink_once
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet, LinkedList};
    use test_log::test;

    use map_macro::hash_map as map;

    use crate::solver::flow_network::{edge, Flow, FlowNetwork};

    use super::{construct_level_graph, construct_residual_graph, solve};

    fn validate_network(network: &FlowNetwork, total_flow: Flow) {
        if let Err(err) = network.validate(Some(total_flow)) {
            eprintln!("{network:?}");
            panic!("{}", err);
        }
    }

    #[test]
    fn wikipedia_residual_1() {
        // Taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge((0, 1), 10, 0);
        network.add_edge((0, 2), 10, 0);
        network.add_edge((1, 2), 2, 0);
        network.add_edge((1, 3), 4, 0);
        network.add_edge((1, 4), 8, 0);
        network.add_edge((2, 4), 9, 0);
        network.add_edge((3, 5), 10, 0);
        network.add_edge((4, 3), 6, 0);
        network.add_edge((4, 5), 10, 0);

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
        // Taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge((0, 1), 10, 10);
        network.add_edge((0, 2), 10, 4);
        network.add_edge((1, 2), 2, 0);
        network.add_edge((1, 3), 4, 4);
        network.add_edge((1, 4), 8, 6);
        network.add_edge((2, 4), 9, 4);
        network.add_edge((3, 5), 10, 4);
        network.add_edge((4, 3), 6, 0);
        network.add_edge((4, 5), 10, 10);

        let mut residual = FlowNetwork::empty(0, 5);
        construct_residual_graph(&network, &mut residual);

        assert_eq!(
            residual.capacities(),
            &map! {
                edge(0, 2) =>  6,
                edge(1, 0) => 10,
                edge(1, 2) =>  2,
                edge(1, 4) =>  2,
                edge(2, 0) =>  4,
                edge(2, 4) =>  5,
                edge(3, 1) =>  4,
                edge(3, 5) =>  6,
                edge(4, 1) =>  6,
                edge(4, 2) =>  4,
                edge(4, 3) =>  6,
                edge(5, 3) =>  4,
                edge(5, 4) => 10,
            }
        );
    }

    #[test]
    fn wikipedia_residual_3() {
        // Taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge((0, 1), 10, 10);
        network.add_edge((0, 2), 10, 9);
        network.add_edge((1, 2), 2, 0);
        network.add_edge((1, 3), 4, 4);
        network.add_edge((1, 4), 8, 6);
        network.add_edge((2, 4), 9, 9);
        network.add_edge((3, 5), 10, 9);
        network.add_edge((4, 3), 6, 5);
        network.add_edge((4, 5), 10, 10);

        let mut residual = FlowNetwork::empty(0, 5);
        construct_residual_graph(&network, &mut residual);

        assert_eq!(
            residual.capacities(),
            &map! {
                edge(0, 2) =>  1,
                edge(1, 0) => 10,
                edge(1, 2) =>  2,
                edge(1, 4) =>  2,
                edge(2, 0) =>  9,
                edge(3, 1) =>  4,
                edge(3, 4) =>  5,
                edge(3, 5) =>  1,
                edge(4, 1) =>  6,
                edge(4, 2) =>  9,
                edge(4, 3) =>  1,
                edge(5, 3) =>  9,
                edge(5, 4) => 10,
            }
        );
    }

    #[test]
    fn wikipedia_levels_1() {
        // Taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example
        let mut residual = FlowNetwork::empty(0, 5);
        residual.add_edge((0, 1), 10, 0);
        residual.add_edge((0, 2), 10, 0);
        residual.add_edge((1, 2), 2, 0);
        residual.add_edge((1, 3), 4, 0);
        residual.add_edge((1, 4), 8, 0);
        residual.add_edge((2, 4), 9, 0);
        residual.add_edge((3, 5), 10, 0);
        residual.add_edge((4, 3), 6, 0);
        residual.add_edge((4, 5), 10, 0);

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
        // Taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example
        let mut residual = FlowNetwork::empty(0, 5);
        residual.add_edge((0, 2), 6, 0);
        residual.add_edge((1, 0), 10, 0);
        residual.add_edge((1, 2), 2, 0);
        residual.add_edge((1, 4), 2, 0);
        residual.add_edge((2, 0), 4, 0);
        residual.add_edge((2, 4), 5, 0);
        residual.add_edge((3, 1), 4, 0);
        residual.add_edge((3, 5), 6, 0);
        residual.add_edge((4, 1), 6, 0);
        residual.add_edge((4, 2), 4, 0);
        residual.add_edge((4, 3), 6, 0);
        residual.add_edge((5, 3), 4, 0);
        residual.add_edge((5, 4), 10, 0);

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
        // Taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example
        let mut residual = FlowNetwork::empty(0, 5);
        residual.add_edge((0, 2), 1, 0);
        residual.add_edge((1, 0), 10, 0);
        residual.add_edge((1, 2), 2, 0);
        residual.add_edge((1, 4), 2, 0);
        residual.add_edge((2, 0), 9, 0);
        residual.add_edge((3, 1), 4, 0);
        residual.add_edge((3, 4), 5, 0);
        residual.add_edge((3, 5), 1, 0);
        residual.add_edge((4, 1), 6, 0);
        residual.add_edge((4, 2), 9, 0);
        residual.add_edge((4, 3), 1, 0);
        residual.add_edge((5, 3), 9, 0);
        residual.add_edge((5, 4), 10, 0);

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

    #[test]
    fn wikipedia_solve() {
        // Taken from https://en.wikipedia.org/wiki/Dinic's_algorithm#Example
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge((0, 1), 10, 0);
        network.add_edge((0, 2), 10, 0);
        network.add_edge((1, 2), 2, 0);
        network.add_edge((1, 4), 8, 0);
        network.add_edge((1, 3), 4, 0);
        network.add_edge((2, 4), 9, 0);
        network.add_edge((3, 5), 10, 0);
        network.add_edge((4, 3), 6, 0);
        network.add_edge((4, 5), 10, 0);

        solve(&mut network);

        validate_network(&network, 19);
    }

    #[test]
    fn assignment_too_big_flow_from_source_solve_small() {
        // 5 edges from each vertex but incoming flow is 4
        let mut network = FlowNetwork::empty(0, 5);
        network.add_edge((0, 1), 1, 0);
        network.add_edge((0, 2), 1, 0);

        network.add_edge((1, 3), 1, 0);
        network.add_edge((1, 4), 1, 0);

        network.add_edge((2, 3), 1, 0);
        network.add_edge((2, 4), 1, 0);

        network.add_edge((3, 5), 1, 0);
        network.add_edge((4, 5), 1, 0);

        solve(&mut network);

        validate_network(&network, 2);
    }

    #[test]
    fn assignment_too_big_flow_from_source_solve() {
        // 5 edges from each vertex but incoming flow is 4
        let mut network = FlowNetwork::empty(0, 11);
        network.add_edge((0, 1), 4, 0);
        network.add_edge((0, 2), 4, 0);
        network.add_edge((0, 3), 4, 0);
        network.add_edge((0, 4), 4, 0);
        network.add_edge((0, 5), 4, 0);

        network.add_edge((1, 6), 1, 0);
        network.add_edge((1, 7), 1, 0);
        network.add_edge((1, 8), 1, 0);
        network.add_edge((1, 9), 1, 0);
        network.add_edge((1, 10), 1, 0);

        network.add_edge((2, 6), 1, 0);
        network.add_edge((2, 7), 1, 0);
        network.add_edge((2, 8), 1, 0);
        network.add_edge((2, 9), 1, 0);
        network.add_edge((2, 10), 1, 0);

        network.add_edge((3, 6), 1, 0);
        network.add_edge((3, 7), 1, 0);
        network.add_edge((3, 8), 1, 0);
        network.add_edge((3, 9), 1, 0);
        network.add_edge((3, 10), 1, 0);

        network.add_edge((4, 6), 1, 0);
        network.add_edge((4, 7), 1, 0);
        network.add_edge((4, 8), 1, 0);
        network.add_edge((4, 9), 1, 0);
        network.add_edge((4, 10), 1, 0);

        network.add_edge((5, 6), 1, 0);
        network.add_edge((5, 7), 1, 0);
        network.add_edge((5, 8), 1, 0);
        network.add_edge((5, 9), 1, 0);
        network.add_edge((5, 10), 1, 0);

        network.add_edge((6, 11), 4, 0);
        network.add_edge((7, 11), 4, 0);
        network.add_edge((8, 11), 4, 0);
        network.add_edge((9, 11), 4, 0);
        network.add_edge((10, 11), 4, 0);

        solve(&mut network);

        validate_network(&network, 5 * 4);
    }

    #[test]
    fn assignment_full_solve() {
        // Assign everyone to everyone
        let mut network = FlowNetwork::empty(0, 11);
        network.add_edge((0, 1), 5, 0);
        network.add_edge((0, 2), 5, 0);
        network.add_edge((0, 3), 5, 0);
        network.add_edge((0, 4), 5, 0);
        network.add_edge((0, 5), 5, 0);

        network.add_edge((1, 6), 1, 0);
        network.add_edge((1, 7), 1, 0);
        network.add_edge((1, 8), 1, 0);
        network.add_edge((1, 9), 1, 0);
        network.add_edge((1, 10), 1, 0);

        network.add_edge((2, 6), 1, 0);
        network.add_edge((2, 7), 1, 0);
        network.add_edge((2, 8), 1, 0);
        network.add_edge((2, 9), 1, 0);
        network.add_edge((2, 10), 1, 0);

        network.add_edge((3, 6), 1, 0);
        network.add_edge((3, 7), 1, 0);
        network.add_edge((3, 8), 1, 0);
        network.add_edge((3, 9), 1, 0);
        network.add_edge((3, 10), 1, 0);

        network.add_edge((4, 6), 1, 0);
        network.add_edge((4, 7), 1, 0);
        network.add_edge((4, 8), 1, 0);
        network.add_edge((4, 9), 1, 0);
        network.add_edge((4, 10), 1, 0);

        network.add_edge((5, 6), 1, 0);
        network.add_edge((5, 7), 1, 0);
        network.add_edge((5, 8), 1, 0);
        network.add_edge((5, 9), 1, 0);
        network.add_edge((5, 10), 1, 0);

        network.add_edge((6, 11), 5, 0);
        network.add_edge((7, 11), 5, 0);
        network.add_edge((8, 11), 5, 0);
        network.add_edge((9, 11), 5, 0);
        network.add_edge((10, 11), 5, 0);

        solve(&mut network);

        validate_network(&network, 5 * 5);
    }

    #[test]
    fn assignment_overflowing_solve() {
        // Assign everyone to everyone except their own game
        let mut network = FlowNetwork::empty(0, 11);
        network.add_edge((0, 1), 5, 0);
        network.add_edge((0, 2), 5, 0);
        network.add_edge((0, 3), 5, 0);
        network.add_edge((0, 4), 5, 0);
        network.add_edge((0, 5), 5, 0);

        network.add_edge((1, 7), 1, 0);
        network.add_edge((1, 8), 1, 0);
        network.add_edge((1, 9), 1, 0);
        network.add_edge((1, 10), 1, 0);

        network.add_edge((2, 6), 1, 0);
        network.add_edge((2, 8), 1, 0);
        network.add_edge((2, 9), 1, 0);
        network.add_edge((2, 10), 1, 0);

        network.add_edge((3, 6), 1, 0);
        network.add_edge((3, 7), 1, 0);
        network.add_edge((3, 9), 1, 0);
        network.add_edge((3, 10), 1, 0);

        network.add_edge((4, 6), 1, 0);
        network.add_edge((4, 7), 1, 0);
        network.add_edge((4, 8), 1, 0);
        network.add_edge((4, 10), 1, 0);

        network.add_edge((5, 6), 1, 0);
        network.add_edge((5, 7), 1, 0);
        network.add_edge((5, 8), 1, 0);
        network.add_edge((5, 9), 1, 0);

        network.add_edge((6, 11), 5, 0);
        network.add_edge((7, 11), 5, 0);
        network.add_edge((8, 11), 5, 0);
        network.add_edge((9, 11), 5, 0);
        network.add_edge((10, 11), 5, 0);

        solve(&mut network);

        validate_network(&network, 5 * 4);
    }

    #[test]
    fn assignment_with_forbidden_solve() {
        let mut network = FlowNetwork::empty(0, 11);
        network.add_edge((0, 1), 5, 0);
        network.add_edge((0, 2), 5, 0);
        network.add_edge((0, 3), 5, 0);
        network.add_edge((0, 4), 5, 0);
        network.add_edge((0, 5), 5, 0);

        network.add_edge((1, 7), 1, 0);
        network.add_edge((1, 8), 1, 0);
        network.add_edge((1, 9), 1, 0);
        network.add_edge((1, 10), 1, 0);

        network.add_edge((2, 6), 1, 0);
        network.add_edge((2, 8), 1, 0);
        network.add_edge((2, 9), 1, 0);
        network.add_edge((2, 10), 1, 0);

        network.add_edge((3, 6), 1, 0);
        network.add_edge((3, 7), 1, 0);
        network.add_edge((3, 9), 1, 0);
        network.add_edge((3, 10), 1, 0);

        network.add_edge((4, 6), 1, 0);
        network.add_edge((4, 7), 1, 0);
        network.add_edge((4, 8), 1, 0);
        network.add_edge((4, 10), 1, 0);

        network.add_edge((5, 6), 1, 0);
        network.add_edge((5, 7), 1, 0);
        network.add_edge((5, 8), 1, 0);
        network.add_edge((5, 9), 1, 0);

        network.add_edge((6, 11), 5, 0);
        network.add_edge((7, 11), 5, 0);
        network.add_edge((8, 11), 5, 0);
        network.add_edge((9, 11), 5, 0);
        network.add_edge((10, 11), 5, 0);

        solve(&mut network);

        validate_network(&network, 5 * 4);
    }
}
