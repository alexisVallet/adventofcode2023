#![feature(ascii_char)]
use std::collections::HashMap;

use chumsky::prelude::*;

#[derive(Debug)]
enum Inst {
    Eq(String, u8),
    Dash(String),
}

#[derive(Debug)]
struct Lens {
    value: u8,
    insert_order: usize,
}

type Boxes = Vec<HashMap<String, Lens>>;

fn instruction_str(inst: &Inst) -> String {
    match inst {
        Inst::Eq(var, i) => format!("{var}={i}"),
        Inst::Dash(var) => format!("{var}-"),
    }
}

fn parse_instruction_list() -> impl Parser<char, Vec<Inst>, Error = Simple<char>> {
    let scalar = text::int(10).map(|s: String| s.parse().unwrap());
    let eq = text::ident()
        .then_ignore(just("="))
        .then(scalar)
        .map(|(v, s)| Inst::Eq(v, s));
    let minus = text::ident().then_ignore(just("-")).map(|v| Inst::Dash(v));
    let inst = eq.or(minus);
    inst.separated_by(just(","))
}

fn hash(str: &String) -> u8 {
    let mut cur_val = 0_u16;

    for c in str.chars() {
        cur_val += c.as_ascii().unwrap().to_u8() as u16;
        cur_val *= 17;
        cur_val %= 256;
    }

    cur_val as u8
}

fn instruction_hash(inst: &Inst) -> u8 {
    hash(&instruction_str(inst))
}

fn compute_boxes(inst_list: Vec<Inst>) -> Boxes {
    let mut boxes = Vec::from_iter((0..256).map(|_| HashMap::new()));

    for (i, inst) in inst_list.into_iter().enumerate() {
        match inst {
            Inst::Eq(label, value) => {
                let hash = hash(&label);
                let lens_box = boxes.get_mut(hash as usize).unwrap();
                let to_insert = match lens_box.remove(&label) {
                    None => Lens {
                        value,
                        insert_order: i,
                    },
                    Some(Lens {
                        value: _,
                        insert_order,
                    }) => Lens {
                        value,
                        insert_order,
                    },
                };
                lens_box.insert(label, to_insert);
            }
            Inst::Dash(label) => {
                let hash = hash(&label);
                let lens_box = boxes.get_mut(hash as usize).unwrap();
                lens_box.remove(&label);
            }
        }
    }
    boxes
}

fn focusing_power(boxes: &Boxes) -> usize {
    boxes
        .into_iter()
        .enumerate()
        .map(|(box_num, lenses)| {
            let mut lenses = Vec::from_iter((*lenses).values());
            lenses.sort_by_key(|l| l.insert_order);
            lenses
                .into_iter()
                .enumerate()
                .map(|(slot, lens)| lens.value as usize * (slot + 1) * (box_num + 1))
                .sum::<usize>()
        })
        .sum()
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let instructions = parse_instruction_list().parse(src).unwrap();
    println!(
        "Question 1 answer is: {}",
        instructions
            .iter()
            .map(|i| instruction_hash(i) as u64)
            .sum::<u64>()
    );
    println!(
        "Question 2 answer is: {}",
        focusing_power(&compute_boxes(instructions))
    );
}
