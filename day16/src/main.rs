#![feature(btree_cursors)]
use rayon::prelude::*;
use std::collections::{BTreeMap, HashSet};
use std::ops::{Deref, Range};

use chumsky::prelude::*;

type Scalar = i32;

type Vector = [Scalar; 2];

// We only care about 90/270 degree rotation so
// we just store the sine.
type Rotation = Scalar;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Copy)]
struct Beam {
    pos: Vector,
    dir: Vector,
}

// Represent obstacles as the rotations
// applied to the input beam. Because we have
// separate storage for horizontal and vertical
// beams, mirrors will be stored with different
// matrices.
type Obstacle = Vec<Rotation>;

struct Map {
    row_col_obstacles: [Vec<BTreeMap<Scalar, Obstacle>>; 2],
    shape: [usize; 2],
}

const ROT_LEFT: Rotation = 1;
const ROT_RIGHT: Rotation = -1;

fn parse_obstacles(
) -> impl Parser<char, Vec<(Vec<([Obstacle; 2], Vector)>, usize)>, Error = Simple<char>> {
    let obstacle =
        one_of(r"\/|-.").map_with_span(|c: char, span: Range<usize>| -> ([Obstacle; 2], usize) {
            (
                match c {
                    '|' => [vec![ROT_LEFT, ROT_RIGHT], vec![]],
                    '-' => [vec![], vec![ROT_LEFT, ROT_RIGHT]],
                    '/' => [vec![ROT_LEFT], vec![ROT_RIGHT]],
                    '\\' => [vec![ROT_RIGHT], vec![ROT_LEFT]],
                    _ => [vec![], vec![]],
                },
                span.start,
            )
        });
    let obstacle_row = obstacle.repeated().at_least(1).map_with_span(
        |obstacles, row_span: Range<usize>| -> (Vec<([Obstacle; 2], Vector)>, usize) {
            let row_length = row_span.end - row_span.start;
            (
                obstacles
                    .into_iter()
                    .filter_map(|(row_col_obst, pos)| {
                        if row_col_obst.iter().all(|o| o.len() == 0) {
                            None
                        } else {
                            Some((
                                row_col_obst,
                                [
                                    (pos / (row_length + 1)) as Scalar,
                                    (pos % (row_length + 1)) as Scalar,
                                ],
                            ))
                        }
                    })
                    .collect(),
                row_length,
            )
        },
    );
    obstacle_row.separated_by(text::newline())
}

fn obstacles_to_map(obstacles: Vec<(Vec<([Obstacle; 2], Vector)>, usize)>) -> Map {
    let num_rows = obstacles.len();
    let num_cols = obstacles[0].1;
    let mut row_col_obstacles = [
        Vec::from_iter((0..num_rows).map(|_| BTreeMap::new())),
        Vec::from_iter((0..num_rows).map(|_| BTreeMap::new())),
    ];

    for (obsts, pos) in obstacles.iter().map(|r| r.0.deref()).flatten() {
        for dim1 in 0_usize..2 {
            if obsts[dim1].len() > 0 {
                let dim2 = 1 - dim1;
                row_col_obstacles[dim1][pos[dim1] as usize].insert(pos[dim2], obsts[dim1].clone());
            }
        }
    }

    Map {
        row_col_obstacles,
        shape: [num_rows, num_cols],
    }
}

fn rotate([x, y]: Vector, rot: Rotation) -> Vector {
    [-rot * y, rot * x]
}

fn simulate(map: &Map, beams: HashSet<Beam>) -> HashSet<Vector> {
    let mut energized = HashSet::new();
    let mut beams: HashSet<Beam> = beams;
    let mut traversed_states: HashSet<Vec<Beam>> = HashSet::new();

    while beams.len() > 0 {
        let mut sorted_beams: Vec<Beam> = beams.iter().map(|t| *t).collect();
        sorted_beams.sort();
        if traversed_states.contains(&sorted_beams) {
            break;
        }
        traversed_states.insert(sorted_beams);

        for beam in beams.clone() {
            // We remove the current beam no matter what.
            beams.remove(&beam);
            let beam_dim: usize = if beam.dir[0] == 0 { 0 } else { 1 };
            let beam_orient = beam.dir[1 - beam_dim];
            // Naming assuming horizontal beam, beam_dim = 0
            let beam_row = beam.pos[beam_dim] as usize;
            let beam_col = beam.pos[1 - beam_dim];
            // Looking up the next obstacle if any
            let opt_obst = if beam_orient > 0 {
                map.row_col_obstacles[beam_dim][beam_row]
                    .lower_bound(std::ops::Bound::Excluded(&beam_col))
            } else {
                map.row_col_obstacles[beam_dim][beam_row]
                    .upper_bound(std::ops::Bound::Excluded(&beam_col))
            }
            .key_value();
            // Creating new rotated/split beams as necessary while getting the last
            // column of the current beam.
            let last_col = match opt_obst {
                None => {
                    // No obstacle in that direction. We finished simulating
                    // this beam, so we don't add a new one.
                    if beam_orient > 0 {
                        map.shape[1 - beam_dim] as i32 - 1
                    } else {
                        0
                    }
                }
                Some((new_col, obsts)) => {
                    // We reached an obstacle, so we create new beams
                    // according to the obstacle rotations.
                    for obst in obsts {
                        let mut new_beam = beam.clone();
                        new_beam.pos[1 - beam_dim] = *new_col;
                        new_beam.dir = rotate(beam.dir, *obst);
                        beams.insert(new_beam);
                    }
                    *new_col
                }
            };
            // Adding energized locations between the current and last location.
            let mut col_range = [beam_col, last_col];
            col_range.sort();

            for col in col_range[0]..=col_range[1] {
                let mut en_pos = beam.pos;
                en_pos[1 - beam_dim] = col;
                energized.insert(en_pos);
            }
        }
    }

    energized
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let map = &obstacles_to_map(parse_obstacles().parse(src).unwrap());
    let energized = simulate(
        map,
        HashSet::from([Beam {
            pos: [0, 0],
            dir: [0, 1],
        }]),
    );

    println!("Question 1 answer is: {}", energized.len());
    let shape = map.shape.map(|i| i as Scalar - 1);
    let end_points = [[0, 0], [0, shape[1]], [shape[0], 0], shape];
    let edges = [
        (0, 1, [1, 0]),
        (0, 2, [0, 1]),
        (1, 3, [0, -1]),
        (2, 3, [-1, 0]),
    ];

    println!(
        "Question 2 answer is: {}",
        edges
            .into_iter()
            .flat_map(|(ix1, ix2, dir)| {
                let p1 @ [i1, _] = end_points[ix1];
                let p2 @ [i2, _] = end_points[ix2];
                let dim = if i1 < i2 { 0 } else { 1 };
                (p1[dim]..p2[dim]).map(move |i| {
                    let mut pos = p1;
                    pos[dim] = i;
                    HashSet::from([Beam { pos, dir }])
                })
            })
            .par_bridge()
            .map(|beams| simulate(map, beams).len())
            .max()
            .unwrap()
    );
}
