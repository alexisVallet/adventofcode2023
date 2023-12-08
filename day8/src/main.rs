use chumsky::prelude::*;
use num::integer::lcm;
use std::collections::HashMap;

type NodeName = [char; 3];

#[derive(Debug, Clone)]
struct Node {
    children: [NodeName; 2],
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    L,
    R,
}

fn dir_to_index(d: &Direction) -> usize {
    match d {
        Direction::L => 0,
        Direction::R => 1,
    }
}

#[derive(Debug)]
struct Network {
    instructions: Vec<Direction>,
    nodes: HashMap<NodeName, Node>,
}

fn parser() -> impl Parser<char, Network, Error = Simple<char>> {
    let direction = one_of("LR").map(|c: char| match c {
        'L' => Direction::L,
        'R' => Direction::R,
        _ => panic!("This should never happen"),
    });
    let node_name = one_of(String::from_iter(('A'..='Z').chain('0'..='9')))
        .repeated()
        .exactly(3)
        .map(|s| s.try_into().unwrap());
    let node = node_name
        .clone()
        .then_ignore(just("=").padded())
        .then(
            node_name
                .clone()
                .separated_by(just(",").padded())
                .exactly(2)
                .delimited_by(just("("), just(")")),
        )
        .map(|(name, children)| {
            (
                name,
                Node {
                    children: children.try_into().unwrap(),
                },
            )
        });
    direction
        .clone()
        .repeated()
        .at_least(1)
        .then_ignore(text::whitespace())
        .then(node.clone().separated_by(text::newline()))
        .map(|(instructions, nodes)| Network {
            instructions: instructions,
            nodes: HashMap::from_iter(nodes.iter().map(|t| (*t).clone())),
        })
}

fn node_transition(network: &Network, from: NodeName, direction: Direction) -> NodeName {
    network.nodes[&from].children[dir_to_index(&direction)]
}

fn follow_until(network: &Network, start: NodeName, target: NodeName) -> u64 {
    let mut cur_node_name = start;
    let mut num_steps = 0_u64;

    for d in network.instructions.iter().cycle() {
        if cur_node_name == target {
            break;
        } else {
            cur_node_name = node_transition(network, cur_node_name, *d);
        }
        num_steps += 1;
    }
    num_steps
}

fn find_cycle(network: &Network, start_nodename: NodeName) -> u64 {
    let mut cycle_states: HashMap<(usize, NodeName), u64> = HashMap::new();
    let mut traversed_z_steps: Vec<u64> = Vec::new();
    let mut cur_node_name = start_nodename;
    let mut num_steps = 0_u64;
    let mut cycle_start_step: Option<u64> = None;

    for (inst_id, d) in network.instructions.iter().enumerate().cycle() {
        if cur_node_name[2] == 'Z' {
            traversed_z_steps.push(num_steps);
        }
        let cur_state = (inst_id, cur_node_name);
        cycle_start_step = cycle_states.get(&cur_state).copied();
        match cycle_start_step {
            Some(_) => break,
            _ => (),
        }
        cycle_states.insert(cur_state, num_steps);

        cur_node_name = node_transition(network, cur_node_name, *d);
        num_steps += 1;
    }
    // Here we have detected a cycle.
    let cycle_start_step = cycle_start_step.unwrap();
    let cycle_end_step = num_steps;
    let z_steps_in_cycle = Vec::from_iter(traversed_z_steps.iter().filter_map(|t| {
        if cycle_start_step < *t && *t <= cycle_end_step {
            Some(*t)
        } else {
            None
        }
    }));
    // start by skipping to the first z step in the cycle to make calculations easier
    let cycle_start_to_first_z = z_steps_in_cycle[0] - cycle_start_step;
    num_steps += cycle_start_to_first_z;

    // Then compute the time steps between successive z steps.
    let mut z_deltas: Vec<u64> = Vec::new();

    for i in 1..z_steps_in_cycle.len() {
        z_deltas.push(z_steps_in_cycle[i] - z_steps_in_cycle[i - 1]);
    }
    let last_z_to_cycle_end = cycle_end_step - z_steps_in_cycle.last().unwrap();
    z_deltas.push(last_z_to_cycle_end + cycle_start_to_first_z);

    assert!(z_deltas.len() == 1);
    assert!(traversed_z_steps.len() == 1);
    assert!(z_deltas[0] == traversed_z_steps[0]);

    z_deltas[0]
}

fn follow_parallel(network: &Network) -> u64 {
    network
        .nodes
        .keys()
        .filter(|name| name[2] == 'A')
        .map(|name| find_cycle(network, *name))
        .fold(1, |lcm_acc, cycle_length| lcm(lcm_acc, cycle_length))
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let network = parser().parse(src.clone()).unwrap();

    println!(
        "Question 1 answer is: {}",
        follow_until(&network, ['A', 'A', 'A'], ['Z', 'Z', 'Z'])
    );
    println!("Question 2 answer is: {}", follow_parallel(&network))
}
