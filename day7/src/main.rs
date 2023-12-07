use chumsky::prelude::*;
use itertools::Itertools;
use std::{cmp::Ordering, collections::HashMap};

// Encoding cards
type Card = u8;

static CHAR_TO_CARD: [(char, Card); 13] = [
    ('2', 0),
    ('3', 1),
    ('4', 2),
    ('5', 3),
    ('6', 4),
    ('7', 5),
    ('8', 6),
    ('9', 7),
    ('T', 8),
    ('J', 9),
    ('Q', 10),
    ('K', 11),
    ('A', 12),
];

type Hand = [Card; 5];

fn parser() -> impl Parser<char, Vec<(Hand, u64)>, Error = Simple<char>> {
    let char_to_card = HashMap::from(CHAR_TO_CARD);
    let card_chars = String::from_iter(char_to_card.keys());
    let hand = one_of(card_chars)
        .map(move |c: char| *(char_to_card.get(&c).unwrap()))
        .repeated()
        .exactly(5);
    hand.padded()
        .then(text::int(10).map(|s: String| s.parse().unwrap()))
        .map(|(hand, bet)| (hand.try_into().unwrap(), bet))
        .separated_by(text::newline())
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAkind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

fn hand_type(hand: &Hand) -> HandType {
    let mut hand_vec: [u8; 13] = [0; 13];
    for card in hand {
        hand_vec[*card as usize] += 1;
    }
    hand_vec.sort();
    match hand_vec {
        [.., 5] => HandType::FiveOfAKind,
        [.., 4] => HandType::FourOfAKind,
        [.., 2, 3] => HandType::FullHouse,
        [.., 1, 1, 3] => HandType::ThreeOfAkind,
        [.., 2, 2] => HandType::TwoPair,
        [.., 2] => HandType::OnePair,
        _ => HandType::HighCard,
    }
}

fn hand_cmp(h1: &Hand, h2: &Hand) -> Ordering {
    let type1 = hand_type(h1);
    let type2 = hand_type(h2);
    let type_comp = type1.cmp(&type2);

    match type_comp {
        Ordering::Equal => h1.cmp(&h2),
        _ => type_comp,
    }
}

const J: u8 = 9;

fn joker_shift(card: Card) -> Card {
    if card == J {
        0
    } else if card < J {
        card + 1
    } else {
        card
    }
}

fn process_joker(hand: &Hand) -> (HandType, Hand) {
    let card_for_tie = hand.map(joker_shift);
    let j_positions = hand.iter().enumerate().filter(|(i, c)| **c == J);
    let hand_type = j_positions
        .map(|(pos, _)| (0_u8..=12).map(move |i| (pos, i)))
        .multi_cartesian_product()
        .map(|pos_card_val| {
            let mut new_hand = hand.clone();
            for (pos, val) in pos_card_val {
                new_hand[pos] = val;
            }
            hand_type(&new_hand)
        })
        .max()
        .unwrap_or(hand_type(hand));
    (hand_type, card_for_tie)
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let mut hand_bets = parser().parse(src.clone()).unwrap();
    hand_bets.sort_by(|(h1, _), (h2, _)| hand_cmp(h1, h2));

    println!(
        "Question 1 answer is: {}",
        hand_bets
            .iter()
            .enumerate()
            .map(|(rank, (_, bet))| (rank as u64 + 1) * bet)
            .sum::<u64>()
    );

    let mut joker_hands = Vec::from_iter(
        hand_bets
            .iter()
            .map(|(hand, bet)| (process_joker(hand), bet)),
    );
    joker_hands.sort_by(|(h1, _), (h2, _)| h1.cmp(h2));

    println!(
        "Question 2 answer is: {}",
        joker_hands
            .iter()
            .enumerate()
            .map(|(rank, (_, bet))| (rank as u64 + 1) * *bet)
            .sum::<u64>()
    )
}
