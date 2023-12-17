#![feature(btree_cursors)]
use chumsky::prelude::*;

use std::ops::Range;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound,
};

type Scalar = i32;

type Coord = [Scalar; 2];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Platform {
    row_col_obstacles: [Vec<BTreeMap<Scalar, Rock>>; 2],
    o_rocks: Vec<Coord>,
    shape: Coord,
}

impl Platform {
    pub fn get_obstacles(&mut self, dim: usize, idx: i32) -> &mut BTreeMap<Scalar, Rock> {
        self.row_col_obstacles
            .get_mut(dim)
            .and_then(|col_obst| col_obst.get_mut(idx as usize))
            .unwrap()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
enum Rock {
    O,
    C,
}

fn parse_rocks() -> impl Parser<char, Vec<(Vec<(Rock, Coord)>, Scalar)>, Error = Simple<char>> {
    let tile = one_of("O.#").map_with_span(|c: char, tile_span: Range<usize>| {
        (
            match c {
                'O' => Some(Rock::O),
                '#' => Some(Rock::C),
                _ => None,
            },
            tile_span.start,
        )
    });
    let row = tile.repeated().at_least(1);
    row.map_with_span(|row_tiles, row_span: Range<usize>| {
        let num_cols = row_span.end - row_span.start;
        let rock_tiles = row_tiles
            .into_iter()
            .filter_map(move |(opt_t, i)| {
                opt_t.map(|t| {
                    (
                        t,
                        [
                            (i / (num_cols + 1)) as Scalar,
                            (i % (num_cols + 1)) as Scalar,
                        ],
                    )
                })
            })
            .collect();
        (rock_tiles, num_cols as Scalar)
    })
    .separated_by(text::newline())
}

fn rocks_to_platform(rocks: Vec<(Vec<(Rock, Coord)>, Scalar)>) -> Platform {
    let num_rows = rocks.len();
    let num_cols = rocks[0].1;
    let mut row_col_obstacles: [Vec<BTreeMap<Scalar, Rock>>; 2] = [
        Vec::from_iter((0..num_cols).map(|_| BTreeMap::new())),
        Vec::from_iter((0..num_cols).map(|_| BTreeMap::new())),
    ];
    let mut o_rocks = Vec::new();

    for (rock, c) in rocks.iter().map(|t| &t.0).flatten() {
        for dim1 in 0_usize..=1 {
            let dim2 = 1 - dim1;
            let j = c[dim1];
            let i = c[dim2];
            row_col_obstacles[dim1][j as usize].insert(i, *rock);
            if *rock == Rock::O && dim1 == 0 {
                o_rocks.push(*c);
            }
        }
    }

    Platform {
        row_col_obstacles,
        o_rocks,
        shape: [num_rows as Scalar, num_cols as Scalar],
    }
}

type Direction = (usize, Scalar);

const NORTH: Direction = (0, -1);
const SOUTH: Direction = (0, 1);
const WEST: Direction = (1, -1);
const EAST: Direction = (1, 1);

fn tilt(platform: &mut Platform, (dim1, dir): Direction) {
    let shape = platform.shape;
    let mut orig_o_rocks = platform.o_rocks.clone();
    // When going north, need to iterate from top to bottom, etc.
    orig_o_rocks.sort_by_key(|c| c[dim1] * -dir);
    let mut new_o_rocks = Vec::new();
    // NORTH: dim1 = 0, dir = -1
    for c in orig_o_rocks {
        // NORTH: dim2 = 1
        let dim2 = 1 - dim1;
        // NORTH: j is column index, i is row index
        let col_idx = c[dim2];
        let orig_row_idx = c[dim1];
        // North: dim2 == 1 so this is the column btree.
        let col_btree = platform.get_obstacles(dim2, col_idx);
        col_btree.remove(&orig_row_idx);
        let new_row;
        // North: dir < 0 so upper bound of row_idx excluded
        let mut obstacle_cursor = if dir < 0 {
            col_btree.upper_bound_mut(Bound::Excluded(&orig_row_idx))
        } else {
            col_btree.lower_bound_mut(Bound::Excluded(&orig_row_idx))
        };
        // North: dir == -1 so -dir works out to +1.
        let border = if dir < 0 { -1 } else { shape[dim1] };
        new_row = obstacle_cursor.key().map(|t| *t).unwrap_or(border) - dir;
        let mut new_o_rock = c;
        new_o_rock[dim1] = new_row;
        new_o_rocks.push(new_o_rock);
        // North: dir < 0 so we insert after at the new row.
        if dir < 0 {
            obstacle_cursor.insert_after(new_row, Rock::O);
        } else {
            obstacle_cursor.insert_before(new_row, Rock::O);
        }
        // Updating the other btree.
        let old_row_btree = platform.get_obstacles(dim1, orig_row_idx);
        old_row_btree.remove(&col_idx);
        let new_row_btree = platform.get_obstacles(dim1, new_row);
        new_row_btree.insert(col_idx, Rock::O);
    }
    platform.o_rocks = new_o_rocks;
}

fn total_load_north(plaform: &Platform) -> Scalar {
    plaform
        .o_rocks
        .iter()
        .map(|[i, _]| plaform.shape[0] - i)
        .sum()
}

fn visualize(plaform: &Platform) {
    for i in 0..plaform.shape[0] {
        for j in 0..plaform.shape[1] {
            let s = match plaform.row_col_obstacles[1][j as usize].get(&i) {
                None => ".",
                Some(Rock::O) => "O",
                Some(Rock::C) => "#",
            };
            print!("{s}");
        }
        println!("");
    }
    println!("");
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let platform = rocks_to_platform(parse_rocks().parse(src).unwrap());
    let mut platform_q1 = platform.clone();
    tilt(&mut platform_q1, NORTH);

    println!("Question 1 answer is: {}", total_load_north(&platform_q1));
    let mut platform_q2 = platform;
    let num_iter = 4000000000;
    let mut dir_iter = [NORTH, WEST, SOUTH, EAST]
        .into_iter()
        .cycle()
        .take(num_iter)
        .enumerate();
    let mut state_cache: HashMap<(Direction, Vec<Coord>), usize> = HashMap::new();
    let mut cycle_start_end = None;

    while let Some((i, dir)) = dir_iter.next() {
        tilt(&mut platform_q2, dir);
        let mut rock_coords = platform_q2.o_rocks.clone();
        rock_coords.sort();
        let state_key = (dir, rock_coords);
        if let Some(start) = state_cache.get(&state_key) {
            cycle_start_end = Some((*start, i));
            break;
        }
        state_cache.insert(state_key, i);
    }

    // Found a cycle, fast forwarding.
    let (start, end) = cycle_start_end.unwrap();
    let remaining_steps = num_iter - end;
    let fast_forward_amount = (remaining_steps / (end - start)) * (end - start);
    let mut i = end + fast_forward_amount + 1;

    while i < num_iter {
        let (_, dir) = dir_iter.next().unwrap();
        tilt(&mut platform_q2, dir);
        i += 1;
    }

    println!("Question 2 answer is: {}", total_load_north(&platform_q2));
}
