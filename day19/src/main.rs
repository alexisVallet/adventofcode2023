use std::collections::HashMap;
use std::ops::Range;

use chumsky::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Op {
    L,
    G,
}

fn compute(op: Op, a: i32, b: i32) -> bool {
    match op {
        Op::L => a < b,
        Op::G => a > b,
    }
}

#[derive(Debug, Clone)]
enum Rule {
    A,
    R,
    ToWF(String),
    Cond(usize, Op, i32, Box<Rule>),
}

type Workflow = Vec<Rule>;

type Part = [i32; 4];

const X: usize = 0;
const M: usize = 1;
const A: usize = 2;
const S: usize = 3;

fn parse_workflows_parts(
) -> impl Parser<char, (HashMap<String, Workflow>, Vec<Part>), Error = Simple<char>> {
    let wf_name = text::ident();
    let accept = just("A").map(|_| Rule::A);
    let reject = just("R").map(|_| Rule::R);
    let part_cat = one_of("xmas").map(|c: char| match c {
        'x' => X,
        'm' => M,
        'a' => A,
        's' => S,
        _ => panic!("Should never happen"),
    });
    let op = one_of("<>").map(|c: char| match c {
        '<' => Op::L,
        '>' => Op::G,
        _ => panic!("Should never happen"),
    });
    let towf = wf_name.clone().map(|s: String| Rule::ToWF(s));
    let num = text::int(10).map(|s: String| s.parse().unwrap());
    let cond = part_cat
        .clone()
        .then(op.clone())
        .then(num.clone())
        .then_ignore(just(":"))
        .then(accept.clone().or(reject.clone()).or(towf.clone()))
        .map(|(((part_cat, op), cst), tgt)| Rule::Cond(part_cat, op, cst, Box::new(tgt)));
    let rule = cond
        .clone()
        .or(accept.clone())
        .or(reject.clone())
        .or(towf.clone());
    let workflow = wf_name.clone().then(
        rule.clone()
            .separated_by(just(","))
            .delimited_by(just("{"), just("}")),
    );
    let part = part_cat
        .clone()
        .then_ignore(just("="))
        .then(num.clone())
        .separated_by(just(","))
        .exactly(4)
        .map(|cat_vals| {
            cat_vals.into_iter().fold([0, 0, 0, 0], |cur, (cat, val)| {
                let mut out = cur;
                out[cat] = val;
                out
            })
        })
        .delimited_by(just("{"), just("}"));
    workflow
        .clone()
        .separated_by(text::newline())
        .at_least(1)
        .padded()
        .then(part.clone().separated_by(text::newline()).at_least(1))
        .map(|(workflows, parts)| (workflows.into_iter().collect(), parts))
}

fn process_part(workflows: &HashMap<String, Workflow>, cur_wf: String, part: Part) -> bool {
    let wf = workflows.get(&cur_wf).unwrap();
    let process_non_cond_rule = |rule: &Rule| -> bool {
        match rule {
            Rule::A => true,
            Rule::R => false,
            Rule::ToWF(wf) => return process_part(workflows, wf.clone(), part),
            _ => panic!("Should never happen"),
        }
    };

    for rule in wf {
        match rule {
            Rule::Cond(cat, op, cst, tgt_rule) => {
                if compute(*op, part[*cat], *cst) {
                    return process_non_cond_rule(&tgt_rule);
                }
            }
            rule => return process_non_cond_rule(rule),
        }
    }
    panic!("Reached end of wf without reject/accept!")
}

type PartIntervals = [Range<i32>; 4];

fn interval_branches(
    r @ Range { start, end }: Range<i32>,
    op: Op,
    cst: i32,
) -> (Range<i32>, Range<i32>) {
    if cst < start {
        // The entire range is greater.
        match op {
            Op::L => (0..0, r),
            Op::G => (r, 0..0),
        }
    } else if end <= cst {
        // The entier range is lower.
        match op {
            Op::L => (r, 0..0),
            Op::G => (0..0, r),
        }
    } else {
        // Otherwise we split into the true and false branches.
        match op {
            Op::L => (start..cst, cst..end),
            Op::G => (cst + 1..end, start..cst + 1),
        }
    }
}

fn valid_intervals(
    workflows: &HashMap<String, Workflow>,
    cur_rule: Rule,
    remaining_rules: &[Rule],
    part_intervals: PartIntervals,
) -> Vec<PartIntervals> {
    match cur_rule {
        Rule::A => vec![part_intervals],
        Rule::R => vec![],
        Rule::ToWF(wf) => {
            let new_vf = workflows.get(&wf).unwrap();
            valid_intervals(workflows, new_vf[0].clone(), &new_vf[1..], part_intervals)
        }
        Rule::Cond(cat, op, cst, tgt_rule) => {
            let (true_interval, false_interval) =
                interval_branches(part_intervals[cat].clone(), op, cst);
            let mut out = Vec::new();
            if !true_interval.is_empty() {
                let mut true_part_intervals = part_intervals.clone();
                true_part_intervals[cat] = true_interval;
                let mut true_out =
                    valid_intervals(workflows, *tgt_rule, remaining_rules, true_part_intervals);
                out.append(&mut true_out);
            }
            if !false_interval.is_empty() {
                let mut false_part_intervals = part_intervals.clone();
                false_part_intervals[cat] = false_interval;
                let mut false_out = valid_intervals(
                    workflows,
                    remaining_rules[0].clone(),
                    &remaining_rules[1..],
                    false_part_intervals,
                );
                out.append(&mut false_out);
            }
            out
        }
    }
}

fn num_valid_parts(workflows: &HashMap<String, Workflow>) -> usize {
    let start_wf = workflows.get(&"in".to_string()).unwrap();
    valid_intervals(
        workflows,
        start_wf[0].clone(),
        &start_wf[1..],
        [1..4001, 1..4001, 1..4001, 1..4001],
    )
    .into_iter()
    .map(|valid_inter| {
        valid_inter
            .into_iter()
            .map(|Range { start, end }| (end - start) as usize)
            .product::<usize>()
    })
    .sum()
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let (workflows, parts) = parse_workflows_parts().parse(src).unwrap();
    println!(
        "Question 1 answer is: {}",
        parts
            .into_iter()
            .filter_map(|part| if process_part(&workflows, "in".to_string(), part) {
                Some(part.into_iter().sum::<i32>())
            } else {
                None
            })
            .sum::<i32>()
    );
    println!("Question 2 answer is: {}", num_valid_parts(&workflows))
}
