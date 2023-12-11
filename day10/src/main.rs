use core::num;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, Range};

use chumsky::prelude::*;
use chumsky::primitive::Container;
use multiset::HashMultiSet;
use petgraph::algo::dijkstra;
use petgraph::data::Build;
use petgraph::graph::NodeIndex;
use petgraph::graph::{Graph, UnGraph};
use petgraph::visit::Dfs;

type Coord = (i32, i32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Pipe {
    S,
    H,
    V,
    UL,
    DL,
    UR,
    DR,
}

#[derive(Debug)]
struct Map {
    pipe_loop: UnGraph<Coord, ()>,
    start: (Coord, NodeIndex),
}

fn parse_pipe_coords(
    line_length: usize,
) -> impl Parser<char, HashMap<Coord, Pipe>, Error = Simple<char>> {
    let nodes = one_of("-|S7LJF")
        .map(|c: char| match c {
            'S' => Pipe::S,
            '-' => Pipe::H,
            '|' => Pipe::V,
            'F' => Pipe::UL,
            '7' => Pipe::UR,
            'L' => Pipe::DL,
            'J' => Pipe::DR,
            _ => panic!("This should never happen"),
        })
        .map_with_span(move |pipe: Pipe, span: Range<usize>| {
            (
                (
                    (span.start / (line_length + 1)) as i32,
                    (span.start % (line_length + 1)) as i32,
                ),
                pipe,
            )
        })
        .padded_by(one_of(".\n").repeated());
    nodes
        .clone()
        .repeated()
        .map(|nodes| HashMap::from_iter(nodes))
}

fn map_from_pipes(node_coords: &HashMap<Coord, Pipe>) -> Map {
    let all_edges = node_coords.iter().flat_map(move |(src_id @ (i, j), pipe)| {
        match pipe {
            Pipe::S => vec![], // Since we build an undirected graph, it will be picked up by the other nodes.
            Pipe::V => vec![(-1, 0), (1, 0)],
            Pipe::H => vec![(0, -1), (0, 1)],
            Pipe::UL => vec![(1, 0), (0, 1)],
            Pipe::DL => vec![(-1, 0), (0, 1)],
            Pipe::UR => vec![(0, -1), (1, 0)],
            Pipe::DR => vec![(0, -1), (-1, 0)],
        }
        .into_iter()
        .filter_map(move |(oi, oj)| {
            let tgt_id = (i + oi, j + oj);
            if node_coords.contains_key(&tgt_id) {
                Some((*src_id, tgt_id))
            } else {
                None
            }
        })
    });
    let edges_with_counts =
        HashMultiSet::from_iter(
            all_edges.map(|(src, tgt)| if src < tgt { (src, tgt) } else { (tgt, src) }),
        );
    // We keep edges with multiplicity 2, except when they connect to the start, in which case
    // we keep them always.
    let start = node_coords
        .into_iter()
        .find_map(|(i, p)| if *p == Pipe::S { Some(i) } else { None })
        .unwrap();
    let edges_to_keep = edges_with_counts
        .distinct_elements()
        .filter_map(|e @ (src, tgt)| {
            if src == start || tgt == start || edges_with_counts.count_of(e) == 2 {
                Some(e)
            } else {
                None
            }
        });
    let pipe_loop = RefCell::new(Graph::new_undirected());
    let mut added_map: HashMap<(i32, i32), NodeIndex> = HashMap::new();

    let mut get_coord_idx = |c| {
        added_map.get(c).map(|v| *v).unwrap_or_else(|| {
            let i1;
            {
                let mut pipe_loop = pipe_loop.borrow_mut();
                i1 = pipe_loop.add_node(*c);
            }
            added_map.insert(*c, i1);
            i1
        })
    };

    for (c1, c2) in edges_to_keep {
        let i1 = get_coord_idx(c1);
        let i2 = get_coord_idx(c2);
        {
            let mut pipe_loop = pipe_loop.borrow_mut();
            pipe_loop.add_edge(i1, i2, ());
        }
    }

    Map {
        pipe_loop: pipe_loop.take(),
        start: (*start, *(added_map.get(&start).unwrap())),
    }
}

fn farthest_steps(map: &Map) -> i32 {
    let (_, start) = map.start;
    *(dijkstra(&map.pipe_loop, start.into(), None, |_| 1)
        .values()
        .into_iter()
        .max()
        .unwrap())
}

fn num_contained(map: &Map, line_length: usize, num_lines: usize) -> i32 {
    // Keep only the connected components of the start node by Dfs.
    let mut cc_node_indices = HashSet::new();
    let (start_coords @ (s_i, s_j), start_idx) = map.start;
    let mut dfs = Dfs::new(&map.pipe_loop, start_idx);

    while let Some(node) = dfs.next(&map.pipe_loop) {
        cc_node_indices.insert(node);
    }
    // Filter the nodes while doubling the coordinates to leave some gaps for a flood fill.
    let bigger_loop = map.pipe_loop.filter_map(
        |node, (i, j)| {
            if cc_node_indices.contains(&node) {
                Some((i * 2, j * 2))
            } else {
                None
            }
        },
        |_, e| Some(e),
    );

    let mut cc_node_coords = HashSet::new();

    // Build up coordinates by following edges.
    for edge in bigger_loop.edge_indices() {
        let (n1, n2) = bigger_loop.edge_endpoints(edge).unwrap();
        let (i1, j1) = *(bigger_loop.node_weight(n1).unwrap());
        let (i2, j2) = *(bigger_loop.node_weight(n2).unwrap());
        let coords = if i1 == i2 {
            Vec::from_iter((j1..=j2).map(|j| (i1, j)))
        } else {
            Vec::from_iter((i1..=i2).map(|i| (i, j1)))
        };
        cc_node_coords.extend(coords.into_iter());
    }

    let num_lines = num_lines as i32;
    let line_length = line_length as i32;

    // Flood fill the outside
    let mut empty_tile_graph: Graph<(i32, i32), (), petgraph::prelude::Undirected> =
        Graph::new_undirected();
    let mut coord_to_node = HashMap::new();
    let mut num_total = 0;

    for i in -1..(2 * num_lines + 1) {
        for j in -1..(2 * line_length + 1) {
            let coord = (i, j);
            if i % 2 == 0 && j % 2 == 0 {
                num_total += 1;
            }
            if cc_node_coords.contains(&coord) {
                continue;
            }
            let node = empty_tile_graph.add_node(coord);
            coord_to_node.insert(coord, node);
        }
    }

    for ((i, j), src) in coord_to_node.iter() {
        for (oi, oj) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let tgt_coord = (i + oi, j + oj);
            match coord_to_node.get(&tgt_coord) {
                Some(tgt) => {
                    empty_tile_graph.add_edge(*src, *tgt, ());
                }
                None => (),
            }
        }
    }
    let mut dfs = Dfs::new(&empty_tile_graph, *(coord_to_node.get(&(-1, -1)).unwrap()));
    let mut num_outside_nodes: i32 = 0;
    let mut outside_coords = HashSet::new();

    while let Some(node) = dfs.next(&empty_tile_graph) {
        // Count only the tiles whose location actually exists in the
        // unscaled graph
        let c @ (i, j) = *(empty_tile_graph.node_weight(node).unwrap());
        outside_coords.insert(c);
        if i % 2 == 0 && j % 2 == 0 {
            num_outside_nodes += 1;
        }
    }

    let num_loop = bigger_loop.node_count() as i32;
    num_total - (num_outside_nodes + num_loop)
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let mut lines = src.lines();
    let num_lines = lines.clone().count();
    let line_length = lines.next().unwrap().len();
    let pipe_coords = parse_pipe_coords(line_length).parse(src.clone()).unwrap();
    let map = map_from_pipes(&pipe_coords);

    println!("Question 1 answer is {}", farthest_steps(&map));
    println!(
        "Question 2 answer is {}",
        num_contained(&map, line_length, num_lines)
    )
}
