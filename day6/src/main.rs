use chumsky::prelude::*;
use std::iter::zip;
use std::ops::Range;

type IntUnit = u64;

#[derive(Debug, Clone)]
struct Race {
    time: IntUnit,
    record: IntUnit,
}

fn parser() -> impl Parser<char, Vec<Race>, Error = Simple<char>> {
    let number = text::int(10).map(|s: String| s.parse().unwrap());
    let number_list = number
        .clone()
        .separated_by(just(' ').repeated().at_least(1));
    let time_list = just("Time:").ignored().padded().then(number_list.clone());
    let distance_list = just("Distance:")
        .ignored()
        .padded()
        .then(number_list.clone());
    time_list
        .clone()
        .then_ignore(text::newline())
        .then(distance_list.clone())
        .map(|((_, times), (_, records))| {
            Vec::from_iter(
                zip(times.iter(), records.iter()).map(|(time, record)| Race {
                    time: *time,
                    record: *record,
                }),
            )
        })
}

fn solve_f64(time: f64, record: f64, scale: f64) -> (f64, f64) {
    let time = time * scale;
    let record = record * scale;
    let a = -1. * scale;
    let delta = time.powf(2.) - 4. * a * (-record - 1.);
    assert!(delta > 0.);
    let r1 = (-time as f64 + f64::sqrt(delta as f64)) / -2.;
    let r2 = (-time as f64 - f64::sqrt(delta as f64)) / -2.;
    let start = f64::min(r1, r2);
    let end = f64::max(r1, r2);
    (start / scale, end / scale)
}

fn solve_race(race: &Race, scale: f64) -> Range<IntUnit> {
    let (start, end) = solve_f64(race.time as f64, race.record as f64, scale);
    let start = start.clamp(0., race.time as f64);
    let end = end.clamp(0., race.time as f64);
    Range {
        start: start.ceil() as IntUnit,
        end: end.floor() as IntUnit,
    }
}

fn make_big_race(races: &Vec<Race>) -> Race {
    let mut time_str: String = "".to_string();
    let mut record_str: String = "".to_string();

    for race in races {
        time_str.push_str(&race.time.to_string());
        record_str.push_str(&race.record.to_string());
    }
    Race {
        time: time_str.parse().unwrap(),
        record: record_str.parse().unwrap(),
    }
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let races = parser().parse(src.clone()).unwrap();

    println!(
        "Question 1 answer is: {}",
        races
            .iter()
            .map(|r| {
                let range = solve_race(r, 1.);
                range.end - range.start + 1
            })
            .product::<IntUnit>()
    );
    let big_race = make_big_race(&races);
    println!("{:?}", big_race);
    let big_race_solution = solve_race(&big_race, 0.001);
    println!(
        "Question 2 answer is: {}",
        big_race_solution.end - big_race_solution.start + 1
    )
}
