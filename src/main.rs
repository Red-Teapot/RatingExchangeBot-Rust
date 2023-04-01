use log::*;
use map_macro::map;

use crate::solver::dinic;
use crate::solver::flow_network::{edge, FlowNetwork};

pub mod solver;

fn main() {
    pretty_env_logger::init();

    let mut network = FlowNetwork::empty(0, 5);
    network.add_edge(edge(0, 1), 10, 0);
    network.add_edge(edge(0, 2), 10, 0);
    network.add_edge(edge(1, 2), 2, 0);
    network.add_edge(edge(1, 4), 8, 0);
    network.add_edge(edge(1, 3), 4, 0);
    network.add_edge(edge(2, 4), 9, 0);
    network.add_edge(edge(3, 5), 10, 0);
    network.add_edge(edge(4, 3), 6, 0);
    network.add_edge(edge(4, 5), 10, 0);

    dinic::solve(&mut network);

    info!("Network:\n{network:?}");
}
