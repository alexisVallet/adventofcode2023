use chumsky::prelude::*;
use itertools::Itertools;
use std::{collections::HashSet, ops::Range};

type Scalar = i64;

type Galaxy = [Scalar; 2];

type Image = Vec<Galaxy>;

fn parse_image(line_length: usize) -> impl Parser<char, Image, Error = Simple<char>> {
    let galaxy = just("#").map_with_span(move |_: &str, span: Range<usize>| {
        let i = span.start as Scalar;
        let line_length = (line_length + 1) as Scalar;
        [i / line_length, i % line_length]
    });
    galaxy
        .padded_by(one_of(".\n").repeated())
        .repeated()
        .at_least(1)
}

fn empty_dim(image: &Image, dim: usize) -> Vec<Scalar> {
    let all_is: HashSet<Scalar> = HashSet::from_iter(image.iter().map(|t| t[dim]));
    let max_i = all_is.iter().max().unwrap();
    (0..*max_i)
        .into_iter()
        .filter(move |i| !all_is.contains(i))
        .collect()
}

fn expand_image(image: &Image, scale: Scalar) -> Image {
    let empty_rows = empty_dim(image, 0);
    let empty_cols = empty_dim(image, 1);
    image
        .into_iter()
        .map(|[i, j]| {
            let num_before_i = empty_rows.iter().filter(|i_| *i_ < i).count() as Scalar;
            let num_before_j = empty_cols.iter().filter(|j_| *j_ < j).count() as Scalar;
            [i + num_before_i * scale, j + num_before_j * scale]
        })
        .collect()
}

fn all_pair_distance<'a>(image: &'a Image) -> impl Iterator<Item = Scalar> + 'a {
    image.iter().combinations(2).map(|points| {
        let [i1, j1] = points[0];
        let [i2, j2] = points[1];
        (i2 - i1).abs() + (j2 - j1).abs()
    })
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let line_length = src.lines().next().unwrap().len();
    let image = parse_image(line_length).parse(src.clone()).unwrap();

    println!(
        "Question 1 answer is: {}",
        all_pair_distance(&expand_image(&image, 1)).sum::<Scalar>()
    );
    println!(
        "Question 2 answer is: {}",
        all_pair_distance(&expand_image(&image, 999999)).sum::<Scalar>()
    );
}
