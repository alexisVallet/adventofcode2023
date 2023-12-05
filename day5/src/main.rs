use chumsky::prelude::*;
use std::{cmp::Ordering, ops::Range};

type Location = usize;

#[derive(Debug, Clone)]
struct MapRange {
    input: Range<Location>,
    output: Range<Location>,
}

fn range_cmp(m1: &MapRange, m2: &MapRange) -> Ordering {
    m1.input.start.cmp(&m2.input.start)
}

#[derive(Debug)]
struct Map {
    src_name: String,
    dst_name: String,
    map_ranges: Vec<MapRange>,
}

fn map_value(map: &Map, i: Location) -> Location {
    let containing_range = map.map_ranges.binary_search_by(|map_range| {
        if map_range.input.contains(&i) {
            Ordering::Equal
        } else if map_range.input.start > i {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });
    match containing_range {
        Err(_) => i,
        Ok(range_id) => {
            let map_range = &map.map_ranges[range_id];
            let offset = i - map_range.input.start;
            map_range.output.start + offset
        }
    }
}

#[derive(Debug)]
struct Almanac {
    seeds: Vec<Location>,
    maps: Vec<Map>,
}

fn compute_seed_locations(almanac: &Almanac) -> Vec<Location> {
    let mut cur_locs = almanac.seeds.clone();

    for map in &almanac.maps {
        for i in 0..cur_locs.len() {
            cur_locs[i] = map_value(&map, cur_locs[i]);
        }
    }
    cur_locs
}

fn map_dense(seeds: Vec<bool>, map: &Map) -> Vec<bool> {
    let mut out_seeds = seeds.clone();

    for map_range in map.map_ranges.clone() {
        out_seeds[map_range.output].copy_from_slice(&seeds[map_range.input]);
    }
    out_seeds
}

fn dense_seeds(seed_ranges: &Vec<Range<Location>>) -> Vec<bool> {
    let mut out_seeds = vec![false; 2_usize.pow(33)];

    for range in seed_ranges.clone() {
        out_seeds[range.clone()].copy_from_slice(&vec![true; range.end - range.start])
    }

    out_seeds
}

fn seed_ranges(seeds: &Vec<Location>) -> Vec<Range<Location>> {
    Vec::from_iter(
        seeds
            .chunks(2)
            .map(|start_size| start_size[0]..start_size[0] + start_size[1]),
    )
}

fn compute_seed_locations_dense(almanac: &Almanac) -> Vec<bool> {
    let input_seeds = dense_seeds(&seed_ranges(&almanac.seeds));

    almanac.maps.iter().fold(input_seeds, map_dense)
}

fn parser() -> impl Parser<char, Almanac, Error = Simple<char>> {
    let number = text::int(10).map(|s: String| s.parse().unwrap());
    let seeds = number.clone().separated_by(text::whitespace());
    let map_range = number
        .clone()
        .padded()
        .then(number.clone().padded())
        .then(number.clone())
        .map(|((dst, src), size)| MapRange {
            input: src..src + size,
            output: dst..dst + size,
        });
    let seed_line = just("seeds:").padded().then(seeds.clone()).map(|(_, s)| s);
    let name = one_of(String::from_iter('a'..='z'))
        .repeated()
        .at_least(1)
        .map(|s| String::from_iter(s.iter()));
    let map = name
        .clone()
        .then_ignore(just("-to-"))
        .then(name.clone())
        .then_ignore(just(" map:"))
        .then_ignore(text::newline())
        .then(map_range.clone().separated_by(text::newline()))
        .map(|((src_name, dst_name), map_ranges)| Map {
            src_name: src_name,
            dst_name: dst_name,
            map_ranges: {
                let mut copy_ranges = map_ranges.clone();
                copy_ranges.sort_by(range_cmp);
                copy_ranges
            },
        });
    seed_line
        .clone()
        .padded()
        .then(map.clone().separated_by(text::whitespace()))
        .map(|(seeds, maps)| Almanac {
            seeds: seeds,
            maps: maps,
        })
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let almanac = parser().parse(src.clone()).unwrap();
    let seed_locations = compute_seed_locations(&almanac);
    let dense_seed_locations = compute_seed_locations_dense(&almanac);

    println!(
        "Question 1 answer is: {}",
        seed_locations.iter().min().unwrap()
    );
    println!(
        "Question 2 answer is: {}",
        dense_seed_locations.iter().position(|t| *t).unwrap()
    );
}
