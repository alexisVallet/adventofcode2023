use chumsky::prelude::*;
use itertools::{Itertools, MinMaxResult};
use petgraph::{graph::UnGraph, visit::Dfs};
use std::cmp::min;
use std::collections::{HashMap, HashSet};

type Scalar = i64;

type Vector = [Scalar; 2];

#[derive(Debug, Clone)]
struct Edge {
    dir: Vector,
    length: Scalar,
    color: String,
}

fn parse_dig_plan_q1() -> impl Parser<char, Vec<Edge>, Error = Simple<char>> {
    let dir = one_of("UDLR").map(|c: char| match c {
        'U' => [-1, 0],
        'D' => [1, 0],
        'L' => [0, -1],
        'R' => [0, 1],
        _ => panic!("Should never happen"),
    });
    let color = text::digits(16).delimited_by(just("(#"), just(")"));
    let edge = dir
        .then(
            text::int(10)
                .padded()
                .map(|s: String| s.parse::<Scalar>().unwrap()),
        )
        .then(color)
        .map(|((dir, length), color)| Edge { dir, length, color });
    edge.separated_by(text::newline()).at_least(2)
}

fn convert_q2(dig_plan: Vec<Edge>) -> Vec<Edge> {
    dig_plan
        .into_iter()
        .map(|edge| {
            let length = Scalar::from_str_radix(&edge.color[..5], 16).unwrap();
            let dir = match edge.color.chars().nth(5).unwrap() {
                '0' => [0, 1],
                '1' => [1, 0],
                '2' => [0, -1],
                '3' => [-1, 0],
                c => panic!("Got unexpected char {c}"),
            };
            Edge {
                dir,
                length,
                color: "".to_string(),
            }
        })
        .collect()
}

const UP: Vector = [-1, 0];
const DOWN: Vector = [1, 0];
const LEFT: Vector = [0, -1];
const RIGHT: Vector = [0, 1];

fn interior_size(dig_plan: &Vec<Edge>) -> i32 {
    let mut trench_coords: HashSet<Vector> = HashSet::new();
    let mut cur_loc = [0, 0];

    // Computing the trench edge coordinates
    for edge in dig_plan {
        for _ in 0..edge.length {
            trench_coords.insert(cur_loc);
            cur_loc = [0, 1].map(|dim| cur_loc[dim] + edge.dir[dim]);
        }
    }

    // Build a graph for the exterior coordinates
    let MinMaxResult::MinMax(min_i, max_i) = trench_coords.iter().map(|c| c[0]).minmax() else {
        panic!("")
    };
    let MinMaxResult::MinMax(min_j, max_j) = trench_coords.iter().map(|c| c[1]).minmax() else {
        panic!("")
    };
    let total_area = ((max_i + 3 - min_i) * (max_j + 3 - min_j)) as usize;
    let mut graph: UnGraph<Vector, ()> = UnGraph::with_capacity(total_area, total_area * 4);
    let mut coord_to_node = HashMap::new();

    for i in min_i - 1..=max_i + 1 {
        for j in min_j - 1..=max_j + 1 {
            let coord = [i, j];
            if !trench_coords.contains(&coord) {
                let node = graph.add_node([i, j]);
                coord_to_node.insert(coord, node);
            }
        }
    }

    for node in graph.node_indices() {
        let coord = *(graph.node_weight(node).unwrap());
        for dir in [UP, DOWN, RIGHT, LEFT] {
            let neighbor_coord = [0, 1].map(|d| coord[d] + dir[d]);
            if let Some(neighbor_node) = coord_to_node.get(&neighbor_coord) {
                graph.add_edge(node, *neighbor_node, ());
            }
        }
    }

    let start_node = *(coord_to_node.get(&[min_i - 1, min_j - 1]).unwrap());
    let mut exterior_dfs = Dfs::new(&graph, start_node);
    let mut num_exterior = 0;

    while let Some(_) = exterior_dfs.next(&graph) {
        num_exterior += 1;
    }
    // compute the interior size from it
    total_area as i32 - num_exterior
}

fn shoelace_interior_size(dig_plan: &Vec<Edge>) -> Scalar {
    // Making sure the origin is on the top left.
    let mut cur_loc = {
        let mut cur_loc = [0, 0];
        let mut min_loc = cur_loc;

        for edge in dig_plan {
            for d in 0..2 {
                cur_loc[d] += edge.dir[d] * edge.length;
                min_loc[d] = min(min_loc[d], cur_loc[d]);
            }
        }
        [-min_loc[0], -min_loc[1]]
    };
    let mut area_sum = 0;
    let mut dig_plan_wrapped = dig_plan.clone();
    dig_plan_wrapped.insert(0, dig_plan[dig_plan.len() - 1].clone());
    dig_plan_wrapped.push(dig_plan[0].clone());

    for (prev, edge, next) in dig_plan_wrapped.into_iter().tuple_windows() {
        let new_loc = [0, 1].map(|d| cur_loc[d] + edge.dir[d] * edge.length);

        if edge.dir == UP || edge.dir == DOWN {
            // We ignore both extremities.
            let mut signed_edge_length = new_loc[0] - cur_loc[0];
            if signed_edge_length > 0 {
                signed_edge_length -= 1;
            } else if signed_edge_length < 0 {
                signed_edge_length += 1;
            }
            // If we are on a vertical edge, we add the signed area under the edge.
            area_sum += signed_edge_length * cur_loc[1];
            // If the edge is positive, we add the edge tiles as well.
            if signed_edge_length > 0 {
                area_sum += signed_edge_length;
            }
        } else {
            // For horizontal edges, we want to count the area under us:
            // - as negative if it's outside the loop
            // - as positive if it's inside the loop
            // Which we can decide by looking at the previous and next
            // edge direction.
            let (under_inside, over_inside) = match (prev.dir, edge.dir, next.dir) {
                (UP, RIGHT, DOWN) => (false, false),
                (UP, RIGHT, UP) => (false, true),
                (UP, LEFT, DOWN) => (true, true),
                (UP, LEFT, UP) => (false, true),
                (DOWN, RIGHT, DOWN) => (true, false),
                (DOWN, RIGHT, UP) => (true, true),
                (DOWN, LEFT, DOWN) => (true, false),
                (DOWN, LEFT, UP) => (false, false),
                _ => panic!("This should never happen"),
            };
            // Edge cases:
            // - U shapes where everything is inside: you don't count anything, it will be counted by
            //   something else.
            // - U shapes where everything is outside: you just count the edge tiles
            let is_u_shaped = under_inside == over_inside;

            if !is_u_shaped {
                area_sum += min(cur_loc[1], new_loc[1]) * if under_inside { 1 } else { -1 };
            }
            // Then we only add the edge tiles themselves if over is outside.
            if !over_inside {
                // Because the start tile is excluded from vertical edges, need
                // + 1 here.
                area_sum += edge.length + 1;
            }
        };
        cur_loc = new_loc;
    }

    area_sum
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let dig_plan = parse_dig_plan_q1().parse(src).unwrap();
    let q1_answer = interior_size(&dig_plan);
    println!("Question 1 answer is: {}", q1_answer);
    assert_eq!(shoelace_interior_size(&dig_plan), q1_answer as i64);
    let dig_plan_q2 = convert_q2(dig_plan);
    println!(
        "Question 2 answer is: {}",
        shoelace_interior_size(&dig_plan_q2)
    );
}
