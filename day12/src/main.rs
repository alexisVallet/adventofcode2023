#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(coroutine_clone)]
#![feature(core_intrinsics)]
use chumsky::{prelude::*, primitive::OneOf};
use core::{num, panic};
use std::{ops::Range, iter::Scan};
use itertools::{Itertools, Either};
use std::ops::{Coroutine, CoroutineState};
use bitfield::Bit;
use rayon::prelude::*;

// Represent an arrangement as a bit field
// where 1 is # and 0 is from the least significant
// bits onward.

type Scalar = u128;

#[derive(Debug)]
struct SpringRecord {
    damaged: Scalar,
    operational: Scalar,
    sizes: Vec<Scalar>,
    total_size: Scalar
}

#[derive(PartialEq, PartialOrd, Ord, Eq)]
enum Tile {
    D,
    O
}


fn parse_spring_records() -> impl Parser<char, Vec<SpringRecord>, Error = Simple<char>> {
    let tile = one_of("#.?")
            .map_with_span(|c: char, r: Range<usize>| (match c {
                '#' => Some(Tile::D),
                '.' => Some(Tile::O),
                _ => None,
            }, r.start));
    let sizes = text::int(10)
            .map(|s: String| s.parse().unwrap())
            .separated_by(just(","));
    let spring_record = tile
            .repeated()
            .at_least(1)
            .map_with_span(|t, s: Range<usize>| (t, s))
            .padded()
            .then(sizes)
            .map_with_span(|((tiles, tiles_span), sizes), span: Range<usize>| {
                let (damaged, operational): (Vec<Scalar>, Vec<Scalar>) = tiles.iter().filter_map(|(t, i)| match t {
                    Some(t_) => Some((t_, i)),
                    None => None
                }).partition_map(|(t, i)| {
                    let tile_bits = (2 as Scalar).pow((i - span.start) as u32);
                    match t {
                        Tile::D => Either::Left(tile_bits),
                        Tile::O => Either::Right(tile_bits),
                    }
                });
                SpringRecord {
                    damaged: damaged.into_iter().fold(0, |i1, i2| i1 | i2),
                    operational: operational.into_iter().fold(0, |i1, i2| i1 | i2),
                    sizes: sizes,
                    total_size: (tiles_span.end - tiles_span.start) as Scalar
                }
            });
    spring_record.separated_by(text::newline())
}


fn group_sizes_match(i: Scalar, record: &SpringRecord) -> bool {
    let mut i = i;
    let mut actual_group_size = Vec::new();

    while i != 0 {
        i >>= i.trailing_zeros();
        let group_size = i.trailing_ones();
        actual_group_size.push(group_size as Scalar);
        i >>= i.trailing_ones();
    }
    actual_group_size == record.sizes
}


fn arrangements<'a>(record: &'a SpringRecord) -> impl Iterator<Item = Scalar> + 'a {
    // Taking rough lower and upper bounds based on the sizes
    // of the groups and the total size.
    let num_damaged = record.sizes.iter().sum();
    let min_val = (0..num_damaged).map(|i| (2 as Scalar).pow(i as u32)).fold(0, |i1, i2| i1 | i2);
    let max_val = ((record.total_size-num_damaged)..(record.total_size)).map(|i| (2 as Scalar).pow(i as u32)).fold(0, |i1, i2| i1 | i2);
    (min_val..max_val).filter(move |i| {
        i & record.damaged == record.damaged && 
        !i & record.operational == record.operational && 
        i.count_ones() == num_damaged as u32 &&
        group_sizes_match(*i, record)
    })
}


fn expand(bitfield: Scalar, size: Scalar) -> Scalar {
    let mut out = bitfield;

    for _ in 1..=5 {
        out |= out << size + 1;
    }
    out
}


fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let records = parse_spring_records().parse(src.clone()).unwrap();

    println!("Question 1 answer is: {}", records.par_iter().map(|r| arrangements(r).count()).sum::<usize>());

    let expanded: Vec<SpringRecord> = records.iter().map(|r| {
        let mut new_sizes = Vec::new();
        for _ in 0..5 {
            new_sizes.append(&mut r.sizes.clone());
        }
        SpringRecord {
            damaged: expand(r.damaged, r.total_size),
            operational: expand(r.operational, r.total_size),
            sizes: new_sizes,
            total_size: r.total_size * 5 + 4,
        }
    }).collect();
    println!("Question 2 answer is: {}", expanded.par_iter().map(|r| arrangements(r).count()).sum::<usize>())
}
