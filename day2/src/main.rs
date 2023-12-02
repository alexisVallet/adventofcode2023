use core::num;

use chumsky::prelude::*;

#[derive(Debug)]
enum Color {
    R,
    G,
    B,
}

type Cubes = Vec<(u32, Color)>;

#[derive(Debug)]
struct Game {
    id: u32,
    cubes: Vec<Cubes>,
}

fn parser() -> impl Parser<char, Vec<Game>, Error = Simple<char>> {
    let number =
        text::int(10).map(|s: String| s.parse().unwrap());
    let color = 
        just("red")
        .or(just("green"))
        .or(just("blue"))
        .map(|s| match s {
            "red" => Color::R,
            "green" => Color::G,
            "blue" => Color::B,
            _ => panic!("this should never happen")
        });
    let color_value_pair =
        number.clone().padded().then(color.clone());
    let cubes =
        color_value_pair.clone()
        .separated_by(just(',').padded());
    let cubes_list =
        cubes.clone()
        .separated_by(just(';').padded());
    let game =
        just("Game").padded().ignored()
        .then(number.clone())
        .then_ignore(just(":").padded())
        .then(cubes_list.clone())
        .map(|((_, id), cs)| Game {id: id, cubes: cs});
    game.clone().separated_by(text::newline())
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    println!("{:?}", parser().parse(src));
}
