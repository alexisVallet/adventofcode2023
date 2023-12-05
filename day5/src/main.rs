use chumsky::prelude::*;
use std::{ops::Range, cmp::Ordering};

type Location = u64;

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
    let containing_range = map.map_ranges.binary_search_by(
        |map_range| if map_range.input.contains(&i) {
            Ordering::Equal
        } else if map_range.input.start > i  {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    );
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


fn parser() -> impl Parser<char, Almanac, Error = Simple<char>> {
    let number = text::int(10)
        .map(|s: String| s.parse().unwrap());
    let seeds = number.clone()
        .separated_by(text::whitespace());
    let map_range = number.clone().padded()
        .then(number.clone().padded())
        .then(number.clone())
        .map(|((dst, src), size)| MapRange {input: src..src+size, output: dst..dst+size});
    let seed_line = just("seeds:").padded()
        .then(seeds.clone())
        .map(|(_, s)| s);
    let name = one_of(String::from_iter('a'..='z')).repeated().at_least(1)
            .map(|s| String::from_iter(s.iter()));
    let map = name.clone()
        .then_ignore(just("-to-"))
        .then(name.clone())
        .then_ignore(just(" map:"))
        .then_ignore(text::newline())
        .then(map_range.clone().separated_by(text::newline()))
        .map(|((src_name, dst_name), map_ranges)| Map {src_name: src_name, dst_name: dst_name, map_ranges: {
            let mut copy_ranges = map_ranges.clone();
            copy_ranges.sort_by(range_cmp);
            copy_ranges
        } });
    seed_line.clone().padded()
        .then(map.clone().separated_by(text::whitespace()))
        .map(|(seeds, maps)| Almanac { seeds: seeds, maps: maps})
}


fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let almanac = parser().parse(src.clone()).unwrap();
    let seed_locations = compute_seed_locations(&almanac);
    
    println!("Question 1 answer is: {}", seed_locations.iter().min().unwrap())
}
