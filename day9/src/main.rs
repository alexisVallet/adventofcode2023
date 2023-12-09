use chumsky::prelude::*;

type History = Vec<i32>;
type Oasis = Vec<History>;

fn parser() -> impl Parser<char, Oasis, Error = Simple<char>> {
    let number = just("-").repeated().at_most(1).then(text::int(10)).map(
        |(opt_sign, int_str): (Vec<&str>, String)| {
            let nat: i32 = int_str.parse().unwrap();
            match opt_sign.get(0) {
                None => nat,
                Some(_) => -nat,
            }
        },
    );
    let line = number.separated_by(just(" ").repeated().at_least(1));
    line.separated_by(text::newline())
}

fn sequences(history: &History) -> Vec<History> {
    if history.into_iter().all(|i| *i == 0) {
        Vec::from([history.clone()])
    } else {
        let deltas = Vec::from_iter((1..history.len()).map(|i| history[i] - history[i - 1]));
        let mut rest = sequences(&deltas);
        rest.push(history.to_vec());
        rest
    }
}

fn extrapolate(history: &History, forward: bool) -> Vec<i32> {
    let deltas = sequences(history);
    let mut extrapolated = Vec::from([0]);

    for i in 1..deltas.len() {
        let below = *extrapolated.last().unwrap();
        if forward {
            let above = deltas[i].last().unwrap();
            extrapolated.push(above + below);
        } else {
            let above = deltas[i].first().unwrap();
            extrapolated.push(above - below);
        }
    }
    extrapolated
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let oasis = parser().parse(src.clone()).unwrap();

    println!(
        "Question 1 answer is {}",
        oasis
            .iter()
            .map(|h| extrapolate(&h, true).last().unwrap().clone())
            .sum::<i32>()
    );
    println!(
        "Question 2 answer is {}",
        oasis
            .iter()
            .map(|h| extrapolate(&h, false).last().unwrap().clone())
            .sum::<i32>()
    )
}
