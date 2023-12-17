use std::collections::HashSet;
use std::ops::Range;

use chumsky::prelude::*;
use itertools::{Either, Itertools};

type Scalar = i32;

type Coord = [Scalar; 2];

#[derive(Debug, Clone)]
struct Pattern {
    rocks: HashSet<Coord>,
    shape: Coord,
}

fn parse_patterns() -> impl Parser<char, Vec<(usize, Pattern)>, Error = Simple<char>> {
    let tile = one_of("#.").map_with_span(|c: char, span: Range<usize>| match c {
        '#' => (true, span.start),
        _ => (false, span.start),
    });
    let tile_row = tile.repeated().at_least(1);
    let pattern = tile_row
        .separated_by(text::newline())
        .at_least(1)
        .map_with_span(|rows, pat_span: Range<usize>| {
            let num_rows = rows.len();
            let num_cols = rows[0].len();
            let rocks = rows.into_iter().flatten().filter_map(|(is_rock, abs_pos)| {
                if is_rock {
                    Some({
                        let rel_pos = abs_pos - pat_span.start;
                        [
                            2 * (rel_pos / (num_cols + 1)) as i32,
                            2 * (rel_pos % (num_cols + 1)) as i32,
                        ]
                    })
                } else {
                    None
                }
            });
            Pattern {
                rocks: rocks.collect(),
                shape: [(num_rows * 2) as i32, (num_cols * 2) as i32],
            }
        });
    pattern
        .map_with_span(|v, s: Range<usize>| (s.start, v))
        .separated_by(text::whitespace().at_least(1))
}

fn is_symmetric_around_axis(pattern: &Pattern, axis: Scalar, dim: usize) -> bool {
    let (left, right): (HashSet<Coord>, HashSet<Coord>) = pattern.rocks.iter().partition_map(|c| {
        let mut c = *c;
        c[dim] -= axis;

        if c[dim] < 0 {
            c[dim] *= -1;
            Either::Left(c)
        } else {
            Either::Right(c)
        }
    });
    let max_coord = std::cmp::min(axis, pattern.shape[dim] - axis - 1);
    let left_filtered: HashSet<Coord> = left
        .iter()
        .filter_map(|c| if c[dim] <= max_coord { Some(*c) } else { None })
        .collect();
    let right_filtered = right
        .iter()
        .filter_map(|c| if c[dim] <= max_coord { Some(*c) } else { None })
        .collect();
    let out = left_filtered == right_filtered;
    out
}

fn find_symmetries<'a>(pattern: &'a Pattern, dim: usize) -> impl Iterator<Item = Scalar> + 'a {
    (1..pattern.shape[dim] - 1).filter_map(move |i| {
        if i % 2 == 1 && is_symmetric_around_axis(pattern, i, dim) {
            Some(i / 2 + 1)
        } else {
            None
        }
    })
}

fn summarize_pattern<'a>(
    pattern: &'a Pattern,
) -> impl Iterator<Item = (Scalar, bool, Scalar)> + 'a {
    find_symmetries(pattern, 0)
        .map(|row| (100 * row, true, row))
        .chain(find_symmetries(pattern, 1).map(|col| (col, false, col)))
}

fn fix_smudge_summarize(pattern: &Pattern) -> Option<Scalar> {
    let mut new_summary = None;
    let old_summary = summarize_pattern(pattern).next().unwrap();

    'outer: for i in 0..pattern.shape[0] {
        if i % 2 == 0 {
            for j in 0..pattern.shape[1] {
                if j % 2 == 0 {
                    let k = [i, j];
                    let mut new_rocks = pattern.rocks.clone();
                    if pattern.rocks.contains(&k) {
                        new_rocks.remove(&k);
                    } else {
                        new_rocks.insert(k);
                    }
                    let candidate = Pattern {
                        rocks: new_rocks,
                        shape: pattern.shape,
                    };
                    new_summary = summarize_pattern(&candidate)
                        .filter(|new_s| *new_s != old_summary)
                        .next();
                    if new_summary.is_some() {
                        break 'outer;
                    }
                }
            }
        }
    }
    new_summary.map(|t| t.0)
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let patterns = parse_patterns().parse(src.clone()).unwrap();

    println!(
        "Question 1 answer is: {}",
        patterns
            .iter()
            .map(|(_, p)| summarize_pattern(p).next().unwrap().0)
            .sum::<i32>()
    );
    println!(
        "Question 2 answer is: {}",
        patterns
            .iter()
            .map(|(i, p)| {
                let opt_sum = fix_smudge_summarize(p);
                match opt_sum {
                    Some(j) => j,
                    None => {
                        let num_lines = src[0..*i].chars().filter(|c| *c == '\n').count();
                        panic!("Did not find smudge for pattern at line {num_lines}")
                    }
                }
            })
            .sum::<i32>()
    );
}
