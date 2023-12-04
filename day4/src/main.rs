use chumsky::{chain::Chain, prelude::*};
use std::collections::HashSet;

#[derive(Debug)]
struct Card {
    id: i32,
    winning: Vec<i32>,
    yours: Vec<i32>,
}

fn parser() -> impl Parser<char, Vec<Card>, Error = Simple<char>> {
    let number = text::int(10).map(|s: String| s.parse().unwrap());
    let num_seq = number
        .clone()
        .padded_by(just(' ').repeated())
        .repeated()
        .at_least(1);
    let card = just("Card")
        .ignored()
        .padded()
        .then(number.clone())
        .then_ignore(just(':').padded())
        .then(num_seq.clone())
        .then_ignore(just('|').padded())
        .then(num_seq.clone())
        .map(|((((), card_id), winning), yours)| Card {
            id: card_id,
            winning: winning,
            yours: yours,
        });
    card.clone().separated_by(text::newline())
}

fn matching_number(card: &Card) -> usize {
    let winning: HashSet<&i32> = HashSet::from_iter(card.winning.iter());
    let yours: HashSet<&i32> = HashSet::from_iter(card.yours.iter());
    let inter: HashSet<&&i32> = HashSet::from_iter(winning.intersection(&yours));
    inter.len()
}

fn worth(card: &Card) -> i32 {
    let num_common = matching_number(card) as u32;
    let worth = if num_common == 0 {
        0
    } else {
        2_i32.pow(num_common - 1)
    };
    worth
}

fn total_cards(cards: &Vec<Card>) -> i32 {
    let mut card_counts = vec![1; cards.len()];

    for (i, card) in cards.iter().enumerate() {
        let num_matching = matching_number(card);
        if i < cards.len() - 1 {
            let start = i + 1;
            let end = std::cmp::min(cards.len() - 1, i + (num_matching as usize));
            let new_slice =
                Vec::from_iter(card_counts[start..=end].iter().map(|c| c + card_counts[i]));
            card_counts[start..=end].copy_from_slice(&new_slice);
        }
    }
    card_counts.iter().sum()
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let cards = parser().parse(src.clone()).unwrap();

    println!(
        "Question 1 answer is {}",
        cards.iter().map(worth).sum::<i32>()
    );
    println!("Question 2 answer is {}", total_cards(&cards));
}
