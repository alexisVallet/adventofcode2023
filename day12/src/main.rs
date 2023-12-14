use bitfield::BitRange;
use bitfield::Bit;
use chumsky::prelude::*;
use itertools::{Either, Itertools};
use rayon::prelude::*;
use std::ops::Range;

// Represent an arrangement as a bit field
// where 1 is # and 0 is from the least significant
// bits onward.

type Scalar = u128;

#[derive(Debug, Clone)]
struct SpringRecord {
    damaged: Scalar,
    operational: Scalar,
    sizes: Vec<usize>,
    total_size: usize,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
enum Tile {
    D,
    O,
}

fn parse_spring_records() -> impl Parser<char, Vec<SpringRecord>, Error = Simple<char>> {
    let tile = one_of("#.?").map_with_span(|c: char, r: Range<usize>| {
        (
            match c {
                '#' => Some(Tile::D),
                '.' => Some(Tile::O),
                _ => None,
            },
            r.start,
        )
    });
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
            let (damaged, operational): (Vec<Scalar>, Vec<Scalar>) = tiles
                .iter()
                .filter_map(|(t, i)| match t {
                    Some(t_) => Some((t_, i)),
                    None => None,
                })
                .partition_map(|(t, i)| {
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
                total_size: (tiles_span.end - tiles_span.start),
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
        actual_group_size.push(group_size as usize);
        i >>= i.trailing_ones();
    }
    actual_group_size == record.sizes
}

fn arrangements<'a>(record: &'a SpringRecord) -> impl Iterator<Item = Scalar> + 'a {
    // Taking rough lower and upper bounds based on the sizes
    // of the groups and the total size.
    let num_damaged = record.sizes.iter().sum();
    let min_val = (0..num_damaged)
        .map(|i| (2 as Scalar).pow(i as u32))
        .fold(0, |i1, i2| i1 | i2);
    let max_val = ((record.total_size - num_damaged) as Scalar..(record.total_size as Scalar))
        .map(|i| (2 as Scalar).pow(i as u32))
        .fold(0, |i1, i2| i1 | i2);
    (min_val..max_val).filter(move |i| {
        let is_correct = i & record.damaged == record.damaged
            && !i & record.operational == record.operational
            && i.count_ones() == num_damaged as u32
            && group_sizes_match(*i, record);
        is_correct
    })
}

fn expand(bitfield: Scalar, size: usize) -> Scalar {
    let mut out = bitfield;

    for _ in 1..5 {
        out |= out << size + 1;
    }
    out
}


fn num_arrangements_all_unk(record: &SpringRecord, cur_group_id: usize, prev_group_loc: Option<usize>) -> u64{
    // If all remains is ??? we don't need to make any checks or build the actual solution.
    if cur_group_id >= record.sizes.len() {
        1
    } else {
        let group_sizes_after = &record.sizes[cur_group_id..];
        let start_loc = prev_group_loc
            .map(|i| i + record.sizes[cur_group_id - 1] + 1)
            .unwrap_or(0);
        let end_loc = record.total_size
            - (group_sizes_after.iter().sum::<usize>() + (group_sizes_after.iter().count() - 1))
            + 1;
        let process_loc = |cur_group_loc: usize| -> u64{
            num_arrangements_all_unk(record, cur_group_id + 1, Some(cur_group_loc))
        };
        (start_loc..end_loc).into_iter().map(process_loc).sum()
    }
}


fn num_arrangements_fast(
    record: &SpringRecord,
    cur_group_id: usize,
    prev_group_loc: Option<usize>,
    cur_solution: Scalar,
) -> u64 {
    // Start by checking if the solution up to now is compatible with known
    // damaged/operational. Prune if that's not the case.
    let start_loc = prev_group_loc
        .map(|i| i + record.sizes[cur_group_id - 1] + 1)
        .unwrap_or(0);
    let do_slice = |bitfield: Scalar| -> Scalar {
        if start_loc == 0 {
            0
        } else {
            bitfield.bit_range(start_loc - 1, 0)
        }
    };
    let slice_sol = do_slice(cur_solution);
    let slice_dmg = do_slice(record.damaged);
    let slice_op = do_slice(record.operational);

    // Check whether we don't have any constraint ahead, if so switch to the
    // faster, unknown only solution.
    let remaining_dmg: Scalar = record.damaged.bit_range(Scalar::BITS as usize - 1, start_loc);
    let remaining_op: Scalar = record.operational.bit_range(Scalar::BITS as usize - 1, start_loc);
    
    if slice_sol & slice_dmg != slice_dmg || !slice_sol & slice_op != slice_op {
        0
    } else if cur_group_id >= record.sizes.len() {
        if cur_solution & record.damaged == record.damaged
            && !cur_solution & record.operational == record.operational
        {
            // Finished and passed checks, the solution is correct.
            1
        } else {
            0
        }
    } else if remaining_dmg == 0 && remaining_op == 0 {
        num_arrangements_all_unk(record, cur_group_id, prev_group_loc)
    } else {
        let group_sizes_after = &record.sizes[cur_group_id..];
        let end_loc = record.total_size
            - (group_sizes_after.iter().sum::<usize>() + (group_sizes_after.iter().count() - 1))
            + 1;
        let process_loc = |cur_group_loc: usize| -> u64{
            let group_mask = (0_u32..record.sizes[cur_group_id] as u32)
                .map(|i| 2_u128.pow(i))
                .fold(0, |i1, i2| i1 | i2)
                << cur_group_loc;
            let new_solution = cur_solution | group_mask;
            num_arrangements_fast(record, cur_group_id + 1, Some(cur_group_loc), new_solution)
        };
        (start_loc..end_loc).into_iter().map(process_loc).sum()
    }
}


fn flip_record(record: &SpringRecord) -> SpringRecord {
    // Flip it around the side of the longest unknown suffix to speed up search.
    let dmg_bits: Vec<bool> = (0..record.total_size).into_iter().map(|i|
        record.damaged.bit(i)
    ).collect();
    let op_bits: Vec<bool> = (0..record.total_size).into_iter().map(|i|
        record.operational.bit(i)
    ).collect();
    let mut prefix_size = 0;
    
    for i in 0..record.total_size {
        if dmg_bits[i] {
            break;
        }
        prefix_size += 1;
    }
    let mut suffix_size = 0;

    for i in 0..record.total_size {
        if dmg_bits[record.total_size - 1 - i] {
            break;
        }
        suffix_size +=1;
    }

    if prefix_size > suffix_size {
        SpringRecord
    } else {
        record.clone()
    }
}


fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let records = parse_spring_records().parse(src.clone()).unwrap();
    let output_fast = records.iter().map(|r| num_arrangements_fast(r, 0, None, 0));
    let output_slow = records.iter().map(|r| arrangements(r).count() as u64);

    for (i, (o_fast, o_slow)) in output_fast.zip(output_slow).enumerate() {
        if o_fast != o_slow {
            println!(
                "Different output at line {}: o_fast={}, o_slow={}, record={:?}",
                i + 1, o_fast, o_slow, records[i]
            );
        }
    }

    println!(
        "Question 1 answer is: {}",
        records
            .par_iter()
            .map(|r| num_arrangements_fast(r, 0, None, 0))
            .sum::<u64>()
    );

    let expanded: Vec<SpringRecord> = records
        .iter()
        .map(|r| {
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
        })
        .collect();

    println!(
        "Question 2 answer is: {}",
        expanded
            .par_iter()
            .enumerate()
            .map(|(i, r)| {
                println!("Computing line {}...", i+1);
                let n = num_arrangements_fast(r, 0, None, 0);
                println!("Finished line {}!", i+1);
                n
            })
            .sum::<u64>()
    )
}
