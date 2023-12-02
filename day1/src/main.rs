fn text_to_digits(src: &String) -> String {
    if src.len() >= 1 {
        // First we try to match the text digits with the string,
        // and if so strip it.
        let text_to_digit_map = [
            ("one", "1"),
            ("two", "2"),
            ("three", "3"),
            ("four", "4"),
            ("five", "5"),
            ("six", "6"),
            ("seven", "7"),
            ("eight", "8"),
            ("nine", "9"),
        ];
        let mut parsed_prefix: Option<&str> = None;
        let mut parsed_suffix: Option<&str> = None;

        for (text, digit) in text_to_digit_map {
            match src.strip_prefix(text) {
                Some(suffix) => {
                    parsed_prefix = Some(digit);
                    parsed_suffix = Some(suffix);
                    break;
                }
                None => (),
            }
        }
        // If we did not match the text digit, we leave the current char is and skip it
        let (mut parsed_prefix, parsed_suffix) = match (parsed_prefix, parsed_suffix) {
            (Some(digit), Some(suffix)) => (digit.to_string(), suffix),
            _ => {
                let mut chars = src.chars();
                let first = chars.next().unwrap();
                (first.to_string(), chars.as_str())
            }
        };
        // We process the suffix recursively, and concatenate the results
        let processed_suffix = text_to_digits(&parsed_suffix.to_string());
        parsed_prefix.push_str(&processed_suffix);
        parsed_prefix
    } else {
        // If the string is empty, we finished processing so we return it as-is.
        src.to_string()
    }
}

fn sum_of_calibration_values(src: &String, parse_letters: bool) -> u32 {
    let mut sum_of_calibration_values: u32 = 0;

    for raw_line in src.lines() {
        let line = if parse_letters {
            text_to_digits(&raw_line.to_string())
        } else {
            raw_line.to_string()
        };

        let mut first: Option<u32> = None;
        let mut last: Option<u32> = None;

        for c in line.chars() {
            match c.to_digit(10) {
                Some(d) => {
                    if first.is_none() {
                        first = Some(d);
                    }
                    last = Some(d);
                }
                None => (),
            }
        }
        match (first, last) {
            (Some(d1), Some(d2)) => sum_of_calibration_values += d1 * 10 + d2,
            _ => panic!("Line missing digits: {line}"),
        }
    }

    sum_of_calibration_values
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let q1_answer = sum_of_calibration_values(&src, false);
    let q2_answer = sum_of_calibration_values(&src, true);

    println!("Question 1 answer is: {q1_answer}");
    println!("Question 2 answer is: {q2_answer}");
}
