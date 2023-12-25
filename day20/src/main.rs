use std::fs::File;
use std::io::prelude::*;
use std::{
    borrow::BorrowMut,
    collections::{HashMap, VecDeque},
};

use chumsky::prelude::*;
use num::integer::lcm;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Pulse {
    High,
    Low,
}

#[derive(Debug, Clone)]
enum ModuleType {
    FlipFlop {
        is_on: bool,
    },
    Conjunction {
        latest_inputs: HashMap<String, Pulse>,
    },
    Broadcast,
}

impl ModuleType {
    pub fn process_pulse(&mut self, src: String, input: Pulse) -> Option<Pulse> {
        match self {
            Self::Broadcast => Some(input),
            Self::FlipFlop { is_on } => {
                if input == Pulse::Low {
                    Some(if *is_on {
                        *is_on = false;
                        Pulse::Low
                    } else {
                        *is_on = true;
                        Pulse::High
                    })
                } else {
                    None
                }
            }
            Self::Conjunction { latest_inputs } => Some({
                latest_inputs.insert(src, input);
                if latest_inputs.values().all(|p| *p == Pulse::High) {
                    Pulse::Low
                } else {
                    Pulse::High
                }
            }),
        }
    }
}

#[derive(Debug, Clone)]
struct Module {
    module_type: ModuleType,
    outputs: Vec<String>,
}

type ModuleConfig = HashMap<String, Module>;

fn parse_module_config() -> impl Parser<char, ModuleConfig, Error = Simple<char>> {
    let mod_name = text::ident();
    let conj = just("&").map(|_| ModuleType::Conjunction {
        latest_inputs: HashMap::new(),
    });
    let flip_flop = just("%").map(|_| ModuleType::FlipFlop { is_on: false });
    let broadcast = just("").map(|_| ModuleType::Broadcast);
    let module = conj
        .or(flip_flop)
        .or(broadcast)
        .then(mod_name.clone())
        .then_ignore(just("->").padded())
        .then(mod_name.clone().separated_by(just(",").padded()))
        .map(|((module_type, module_name), outputs)| {
            (
                module_name,
                Module {
                    module_type,
                    outputs,
                },
            )
        });
    module
        .separated_by(text::newline())
        .map(|mods| HashMap::from_iter(mods.into_iter()))
}

fn init_conjunctions(module_config: &mut ModuleConfig) {
    let mod_names: Vec<String> = module_config.keys().map(|s| s.clone()).collect();
    for mod_name in mod_names {
        let mod_tgts = module_config.get(&mod_name).unwrap().outputs.clone();
        for tgt_name in mod_tgts {
            let opt_tgt_module = module_config.get_mut(&tgt_name);
            if let Some(tgt_module) = opt_tgt_module {
                if let ModuleType::Conjunction {
                    ref mut latest_inputs,
                } = tgt_module.module_type.borrow_mut()
                {
                    latest_inputs.insert(mod_name.clone(), Pulse::Low);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Simulation {
    pulse_queue: VecDeque<(String, String, Pulse)>,
    module_config: ModuleConfig,
    num_high_pulse: usize,
    num_low_pulse: usize,
}

impl Simulation {
    pub fn new(module_config: &ModuleConfig) -> Simulation {
        Simulation {
            pulse_queue: VecDeque::new(),
            module_config: module_config.clone(),
            num_high_pulse: 0,
            num_low_pulse: 0,
        }
    }

    pub fn press_button(&mut self, opt_out_node: Option<String>) -> bool {
        self.pulse_queue
            .push_front(("button".to_string(), "broadcaster".to_string(), Pulse::Low));

        while !self.pulse_queue.is_empty() {
            let (src_name, tgt_name, pulse) = self.pulse_queue.pop_back().unwrap();
            match pulse {
                Pulse::High => self.num_high_pulse += 1,
                Pulse::Low => self.num_low_pulse += 1,
            }

            if let Some(tgt) = self.module_config.get_mut(&tgt_name) {
                if let Some(out_pulse) = tgt.module_type.process_pulse(src_name, pulse) {
                    if let Some(ref out_node) = opt_out_node {
                        if tgt_name == *out_node && out_pulse == Pulse::Low {
                            return true;
                        }
                    }
                    for new_tgt_name in tgt.outputs.iter() {
                        self.pulse_queue.push_front((
                            tgt_name.clone(),
                            new_tgt_name.clone(),
                            out_pulse,
                        ));
                    }
                }
            }
        }
        false
    }
}

fn count_pulses_product(module_config: &ModuleConfig, num_presses: usize) -> usize {
    let mut simulation = Simulation::new(module_config);

    for _ in 0..num_presses {
        simulation.press_button(None);
    }
    simulation.num_high_pulse * simulation.num_low_pulse
}

fn computation_graph(module_config: &ModuleConfig) -> DiGraph<String, ()> {
    let mut graph = Graph::new();
    let mut name_to_node: HashMap<String, NodeIndex> = HashMap::new();

    for (name, module) in module_config {
        let prefix = match module.module_type {
            ModuleType::Broadcast => "",
            ModuleType::Conjunction { .. } => "&",
            ModuleType::FlipFlop { .. } => "%",
        };
        let node = graph.add_node(format!("{prefix}{name}"));
        name_to_node.insert(name.to_string(), node);
    }

    for (name, module) in module_config {
        let src = name_to_node.get(name).unwrap();
        for dst_name in module.outputs.clone() {
            if let Some(dst) = name_to_node.get(&dst_name) {
                graph.add_edge(*src, *dst, ());
            }
        }
    }

    graph
}

fn subgraph_low_iter(output_node: String, module_config: &ModuleConfig) -> usize {
    let mut simulation = Simulation::new(module_config);
    let mut i = 0;
    loop {
        i += 1;
        if simulation.press_button(Some(output_node.clone())) {
            break;
        }
    }
    i
}

fn main() -> std::io::Result<()> {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let mut module_config = parse_module_config().parse(src).unwrap();
    init_conjunctions(&mut module_config);

    println!(
        "Question 1 answer is: {}",
        count_pulses_product(&module_config, 1000)
    );
    let mut graph_dot_file = File::create("graph.dot")?;
    let graph = computation_graph(&module_config);
    let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    graph_dot_file.write(format!("{:?}", dot).as_bytes())?;

    println!(
        "Question 2 answer is: {}",
        ["jc", "vm", "fj", "qq"]
            .map(|s| subgraph_low_iter(s.to_string(), &module_config))
            .into_iter()
            .fold(1, lcm)
    );
    Ok(())
}
