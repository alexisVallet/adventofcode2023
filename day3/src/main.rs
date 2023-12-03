use std::collections::HashMap;
use std::ops::Range;

use chumsky::{prelude::*, text::digits};

#[derive(Debug, Clone, Copy)]
enum Node {
    Num(i32),
    Symbol(char),
}

#[derive(Debug)]
struct Schematic {
    nodes: Vec<Node>,
    // Coords maps 2D coordinates to:
    // - empty location if the key is not present
    // - node index in the nodes field. For numbers,
    //   multiple coordinates may point to the same node.
    coords: HashMap<(usize, usize), usize>,
    symbols: HashMap<usize, (usize, usize)>,
}

#[derive(Debug)]
struct NodeSpan {
    span: Range<usize>,
    node: Node,
}

type SchematicArr = Vec<Vec<NodeSpan>>;

fn parser(line_length: usize) -> impl Parser<char, SchematicArr, Error = Simple<char>> {
    let number = text::int(10).map(|s: String| Node::Num(s.parse().unwrap()));
    let symbol = one_of("*+/$-%#@=&").map(|c: char| Node::Symbol(c));
    let node_span =
        number
            .clone()
            .or(symbol.clone())
            .map_with_span(move |node, span: Range<usize>| NodeSpan {
                span: Range {
                    start: span.start % (line_length + 1),
                    end: span.end % (line_length + 1),
                },
                node: node,
            });
    let line = node_span.clone().padded_by(just('.').repeated()).repeated();
    line.clone().separated_by(text::newline())
}

fn schematic_from_arr(arr: &SchematicArr) -> Schematic {
    let mut nodes = Vec::new();
    let mut coords = HashMap::new();
    let mut symbols = HashMap::new();

    for (i, line) in arr.iter().enumerate() {
        for node_span in line.iter() {
            let node_id = nodes.len();
            nodes.push(node_span.node);
            match node_span.node {
                Node::Symbol(_) => {
                    symbols.insert(node_id, (i, node_span.span.start));
                }
                _ => (),
            }

            for j in node_span.span.clone().into_iter() {
                coords.insert((i, j), node_id);
            }
        }
    }
    Schematic {
        nodes: nodes,
        coords: coords,
        symbols: symbols,
    }
}

fn sum_part_numbers(schematic: &Schematic) -> i32 {
    let mut traversed_num_ids = Vec::new();
    let mut sum_part_numbers: i32 = 0;

    for (sym_id, (sym_i, sym_j)) in schematic.symbols.clone().into_iter() {
        for i in sym_i - 1..=sym_i + 1 {
            for j in sym_j - 1..=sym_j + 1 {
                match schematic.coords.get(&(i, j)) {
                    Some(node_id) => match schematic.nodes[*node_id] {
                        Node::Num(num) => {
                            if !traversed_num_ids.contains(node_id) {
                                sum_part_numbers += num;
                                traversed_num_ids.push(*node_id);
                            }
                        }
                        _ => (),
                    },
                    None => (),
                }
            }
        }
    }
    sum_part_numbers
}

fn sum_gear_ratios(schematic: &Schematic) -> i32 {
    let mut sum_gear_ratios = 0;

    for (sym_id, (sym_i, sym_j)) in schematic.symbols.clone().into_iter() {
        match schematic.nodes[sym_id] {
            Node::Symbol('*') => {
                let mut neighbor_numbers = Vec::new();
                let mut traversed = Vec::new();

                for i in sym_i - 1..=sym_i + 1 {
                    for j in sym_j - 1..=sym_j + 1 {
                        match schematic.coords.get(&(i, j)) {
                            Some(node_id) => {
                                if !traversed.contains(node_id) {
                                    match schematic.nodes[*node_id] {
                                        Node::Num(num) => {
                                            neighbor_numbers.push(num);
                                        }
                                        _ => (),
                                    }
                                    traversed.push(*node_id);
                                }
                            }
                            _ => (),
                        }
                    }
                }
                if neighbor_numbers.len() == 2 {
                    sum_gear_ratios += neighbor_numbers.iter().product::<i32>();
                }
            }
            _ => (),
        }
    }
    sum_gear_ratios
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let line_length = src.lines().next().unwrap().len();
    let schematic_arr = parser(line_length).parse(src.clone()).unwrap();
    let schematic = schematic_from_arr(&schematic_arr);

    println!("Question 1 answer is: {}", sum_part_numbers(&schematic));
    println!("Question 2 answer is: {}", sum_gear_ratios(&schematic));
}
