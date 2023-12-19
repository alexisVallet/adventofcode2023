use std::collections::{HashMap, HashSet};

use chumsky::prelude::*;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::astar;

type Scalar = i32;

type Map = Vec<Vec<Scalar>>;

fn parse_map() -> impl Parser<char, Map, Error = Simple<char>> {
    let tile = one_of(String::from_iter('0'..='9')).map(|c: char| String::from_iter([c]).parse().unwrap());
    let row = tile.repeated().at_least(1);
    row.separated_by(text::newline())
}

type Vector = [Scalar; 2];

const ROT_LEFT: Rotation = 1;
const ROT_RIGHT: Rotation = -1;

// We only care about 90/270 degree rotation so
// we just store the sine.
type Rotation = Scalar;

// Directed graph where each node represent a state as:
// - A location in the lava grid
// - Current direction of the crucible
// - How many steps the crucible can move forward still
// Edges between nodes correspond to the heat loss to go from
// a state to another, if it's possible at all.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct State {
    pos: Vector,
    dir: Vector,
    forward_steps: Scalar
}
struct StateGraph {
    graph: DiGraph<State, f64>,
    shape: Vector,
    init_node: NodeIndex
}

const NORTH: Vector = [-1, 0];
const SOUTH: Vector = [1, 0];
const WEST: Vector = [0, -1];
const EAST: Vector = [0, 1];

fn rotate([x, y]: Vector, rot: Rotation) -> Vector {
    [-rot * y, rot * x]
}

fn build_state_graph(map: &Map) -> StateGraph {
    // Rough upper bounds for number of nodes, edges
    let num_rows = map.len();
    let num_cols = map[0].len();
    // 0, 1 or 2 steps left, and 4 directions
    let max_num_nodes = num_cols * num_rows * 3 * 4;
    // No matter the state you can only turn left, right or keep going straight
    let max_num_edges = max_num_nodes * 3;
    let mut state_graph = DiGraph::with_capacity(max_num_nodes, max_num_edges);
    let mut state_to_node: HashMap<State, NodeIndex> = HashMap::new();
    let num_rows = num_rows as Scalar;
    let num_cols = num_cols as Scalar;

    // Adding all the nodes first
    for i in 0..num_rows {
        for j in 0..num_cols {
            for dir in [NORTH, SOUTH, WEST, EAST] {
                for forward_steps in 0..3 {
                    let state = State {
                        pos: [i, j],
                        dir,
                        forward_steps,
                    };
                    let node = state_graph.add_node(state);
                    state_to_node.insert(state, node);
                }
            }
        }
    }

    // Then creating edges from nodes to other nodes they can reach
    for src_node in state_graph.node_indices() {
        let src_state = *(state_graph.node_weight(src_node).unwrap());
        let directions = vec![rotate(src_state.dir, ROT_LEFT), rotate(src_state.dir, ROT_RIGHT), src_state.dir];

        for tgt_dir@[tgt_dir_i, tgt_dir_j] in directions {
            let mut tgt_state = src_state;
            tgt_state.pos[0] += tgt_dir_i;
            tgt_state.pos[1] += tgt_dir_j;
            tgt_state.dir = tgt_dir;
            if tgt_dir == src_state.dir {
                tgt_state.forward_steps -= 1;
            } else {
                tgt_state.forward_steps = 2;
            }

            // If the state is not found, we cannot reach it
            // either because it's out of bounds, or would require
            // moving forward too many times.
            if let Some(tgt_node) = state_to_node.get(&tgt_state) {
                let [tgt_i, tgt_j] = tgt_state.pos;
                state_graph.add_edge(src_node, *tgt_node, map[tgt_i as usize][tgt_j as usize] as f64);
            }
        }
    }

    // The initial state is special. We choose the initial direction, so
    // we always have forward_steps=2 on the next step, and there is no initial direction.
    let init_state = State {
        pos: [0,0],
        dir: NORTH,  // not used
        forward_steps: 2
    };
    let init_node = state_graph.add_node(init_state);
    state_to_node.insert(init_state, init_node);

    for tgt_dir@[tgt_dir_i, tgt_dir_j] in [SOUTH, EAST] {
        let mut tgt_state = init_state;
        tgt_state.pos[0] += tgt_dir_i;
        tgt_state.pos[1] += tgt_dir_j;
        tgt_state.dir = tgt_dir;
        let tgt_node = state_to_node.get(&tgt_state).unwrap();
        let [tgt_i, tgt_j] = tgt_state.pos;
        state_graph.add_edge(init_node, *tgt_node, map[tgt_i as usize][tgt_j as usize] as f64);
    }
    
    StateGraph {
        graph: state_graph,
        init_node,
        shape: [num_rows, num_cols]
    }
}


fn visualize_path(state_graph: &StateGraph, path: Vec<NodeIndex>) {
    let path_positions: HashSet<Vector> = HashSet::from_iter(
        path.into_iter().map(|n| state_graph.graph.node_weight(n).unwrap().pos)
    );
    
    for i in 0..state_graph.shape[0] {
        for j in 0..state_graph.shape[1] {
            let s = if path_positions.contains(&[i, j]) {
                "#"
            } else {
                "."
            };
            print!("{}", s);
        }
        println!("");
    }
    println!("");
}


fn find_shortest_path_length(state_graph: &StateGraph) -> Scalar {
    let [num_rows, num_cols] = state_graph.shape;
    let tgt_pos = [num_rows-1, num_cols-1];
    let tgt_nodes = state_graph.graph.node_indices().filter(|node|
        state_graph.graph.node_weight(*node).unwrap().pos == tgt_pos
    );
    tgt_nodes.filter_map(|node| {
        let shortest_path = astar(
            &(state_graph.graph), 
            state_graph.init_node,
            |n| n == node,
            |e| *(e.weight()), 
            |src| {
                let [src_i, src_j] = state_graph.graph.node_weight(src).unwrap().pos;
                ((num_rows - 1 - src_i).abs() + (num_cols - 1 - src_j).abs()) as f64
            });
        shortest_path.map(|(cost, _)|  cost as i32)
    }).min().unwrap()
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let map = parse_map().parse(src).unwrap();
    let state_graph = build_state_graph(&map);

    println!("Question 1 answer is: {}", find_shortest_path_length(&state_graph));
}
