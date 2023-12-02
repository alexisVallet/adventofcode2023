use core::num;
use std::collections::HashMap;

use chumsky::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash)]
enum Color {
    R,
    G,
    B,
}

type Cubes = HashMap<Color, u32>;

#[derive(Debug)]
struct Game {
    id: u32,
    cubes: Vec<Cubes>,
}

fn parse_cubes(color_value_pairs: Vec<(u32, Color)>) -> Cubes {
    let mut cubes = HashMap::from([(Color::R, 0), (Color::G, 0), (Color::B, 0)]);

    for (value, color) in color_value_pairs {
        cubes.insert(color, value);
    }
    cubes
}

fn parser() -> impl Parser<char, Vec<Game>, Error = Simple<char>> {
    let number = text::int(10).map(|s: String| s.parse().unwrap());
    let color = just("red")
        .or(just("green"))
        .or(just("blue"))
        .map(|s| match s {
            "red" => Color::R,
            "green" => Color::G,
            "blue" => Color::B,
            _ => panic!("this should never happen"),
        });
    let color_value_pair = number.clone().padded().then(color.clone());
    let cubes = color_value_pair
        .clone()
        .separated_by(just(',').padded())
        .map(parse_cubes);
    let cubes_list = cubes.clone().separated_by(just(';').padded());
    let game = just("Game")
        .padded()
        .ignored()
        .then(number.clone())
        .then_ignore(just(":").padded())
        .then(cubes_list.clone())
        .map(|((_, id), cs)| Game { id: id, cubes: cs });
    game.clone().separated_by(text::newline())
}

fn observation_possible(cubes: &Cubes, observation: &Cubes) -> bool {
    let mut is_possible = true;

    for (color, value) in observation.into_iter() {
        if cubes.get(&color).unwrap() < &value {
            is_possible = false;
            break;
        }
    }
    is_possible
}

fn game_possible(cubes: &Cubes, game: &Game) -> bool {
    (&game.cubes)
        .into_iter()
        .all(|observation| observation_possible(cubes, &observation))
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let games = parser().parse(src).unwrap();
    let q1_cubes = HashMap::from([(Color::R, 12), (Color::G, 13), (Color::B, 14)]);

    println!(
        "Question 1 answer is: {}",
        games
            .into_iter()
            .map(|g| if game_possible(&q1_cubes, &g) {
                g.id
            } else {
                0
            })
            .sum::<u32>()
    );
}
