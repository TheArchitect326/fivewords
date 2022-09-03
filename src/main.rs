use std::error::Error;
use std::process;
use std::time::Instant;

use itertools::Itertools;

use rayon::prelude::*;

fn main() {
    let now = Instant::now();

    let word_list = get_words().unwrap_or_else(|err| {
        eprintln!("Error fetching words: {err}");
        process::exit(1);
    });

    println!("Fetched words");

    let encoded = preprocess_words(&word_list).unwrap_or_else(|err| {
        eprintln!("Error preprocessing words: {err}");
        process::exit(1);
    });

    println!("Preprocessed words");
    println!("Number of words: {}", encoded.len());

    let elapsed = now.elapsed();
    println!(
        "Took {} seconds to fetch and preprocess words",
        elapsed.as_secs_f32()
    );
    let now = Instant::now();

    let solutions = find_solutions(&encoded);

    let elapsed = now.elapsed();
    println!(
        "Single Thread took {} seconds to find solutions",
        elapsed.as_secs_f32()
    );
    let now = Instant::now();

    let solutions = find_solutions1(&encoded);

    let elapsed = now.elapsed();
    println!(
        "Multi Thread took {} seconds to find solutions",
        elapsed.as_secs_f32()
    );
    let now = Instant::now();

    for solution in solutions {
        for sol in decode_solution(solution, &word_list) {
            println!("Solutions found: {:?}", sol)
        }
    }

    let elapsed = now.elapsed();
    println!("Took {} seconds to decode solutions", elapsed.as_secs_f32());
}

fn get_words() -> Result<Vec<String>, Box<dyn Error>> {
    const URL: &str = "https://raw.githubusercontent.com/tabatkins/wordle-list/main/words";
    let text = reqwest::blocking::get(URL)?.text()?;

    let words: Vec<String> = text.lines().map(String::from).collect();

    Ok(words)
}

fn encode_word(word: String) -> u32 {
    const ASCII_LOWER: [char; 26] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    const TWO: u32 = 2;

    let mut bytes: u32 = 0;

    for c in word.chars() {
        let index: usize = ASCII_LOWER.iter().position(|&r| r == c).unwrap();
        bytes |= TWO.pow(index as u32)
    }

    bytes
}

fn decode_word(word: u32, raw_words: &Vec<String>) -> Vec<String> {
    let mut decoded: Vec<String> = Vec::new();

    for raw_word in raw_words {
        if encode_word(raw_word.to_owned()) == word {
            decoded.push(String::from(raw_word.to_owned()))
        }
    }

    decoded
}

fn decode_solution(solution: [u32; 5], raw_words: &Vec<String>) -> Vec<[String; 5]> {
    let mut decoded: Vec<[String; 5]> = Vec::new();

    for word1 in decode_word(solution[0], raw_words) {
        for word2 in decode_word(solution[1], raw_words) {
            for word3 in decode_word(solution[2], raw_words) {
                for word4 in decode_word(solution[3], raw_words) {
                    for word5 in decode_word(solution[4], raw_words) {
                        decoded.push([
                            word1.to_owned(),
                            word2.to_owned(),
                            word3.to_owned(),
                            word4.to_owned(),
                            word5.to_owned(),
                        ]);
                    }
                }
            }
        }
    }

    decoded
}

fn preprocess_words(raw_words: &Vec<String>) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut encoded_words: Vec<u32> = raw_words
        .iter()
        .map(|word| encode_word(word.to_string()))
        .unique()
        .filter(|n| n.count_ones() == 5)
        .collect();

    encoded_words.sort();

    Ok(encoded_words)
}

fn find_solutions(encoded_words: &Vec<u32>) -> Vec<[u32; 5]> {
    let mut solutions: Vec<[u32; 5]> = Vec::new();

    for (i, a) in encoded_words.iter().enumerate() {
        for (j, b) in encoded_words[i + 1..].iter().enumerate() {
            if (a & b) != 0 {
                continue;
            }

            let ab: u32 = a | b;

            for (k, c) in encoded_words[i + j + 1..].iter().enumerate() {
                if (ab & c) != 0 {
                    continue;
                }

                let abc: u32 = ab | c;

                for (l, d) in encoded_words[i + j + k + 1..].iter().enumerate() {
                    if (abc & d) != 0 {
                        continue;
                    }

                    let abcd: u32 = abc | d;

                    for (_m, e) in encoded_words[i + j + k + l + 1..].iter().enumerate() {
                        if (abcd & e) != 0 {
                            continue;
                        }

                        solutions.push([*a, *b, *c, *d, *e]);
                    }
                }
            }
        }
    }

    solutions
}

fn find_solutions1(encoded_words: &[u32]) -> Vec<[u32; 5]> {
    encoded_words
        .par_iter()
        .enumerate()
        .flat_map(|(i, a)| {
            encoded_words[i + 1..]
                .iter()
                .enumerate()
                .filter_map(|(j, b)| {
                    if (a & b) != 0 {
                        None
                    } else {
                        Some(((i, a), (j, b)))
                    }
                })
                .collect::<Vec<_>>()
        })
        .flat_map(|((i, a), (j, b))| {
            let ab: u32 = a | b;
            encoded_words[i + j + 1..]
                .iter()
                .enumerate()
                .filter_map(|(k, c)| {
                    if (ab & c) != 0 {
                        None
                    } else {
                        Some((ab, (i, a), (j, b), (k, c)))
                    }
                })
                .collect::<Vec<_>>()
        })
        .flat_map(|(ab, (i, a), (j, b), (k, c))| {
            let abc: u32 = ab | c;
            encoded_words[i + j + k + 1..]
                .iter()
                .enumerate()
                .filter_map(|(l, d)| {
                    if (abc & d) != 0 {
                        None
                    } else {
                        Some((abc, (i, a), (j, b), (k, c), (l, d)))
                    }
                })
                .collect::<Vec<_>>()
        })
        .flat_map(|(abc, (i, a), (j, b), (k, c), (l, d))| {
            let abcd: u32 = abc | d;
            encoded_words[i + j + k + l + 1..]
                .iter()
                .enumerate()
                .filter_map(|(m, e)| {
                    if (abcd & e) != 0 {
                        None
                    } else {
                        Some(((i, a), (j, b), (k, c), (l, d), (m, e)))
                    }
                })
                .collect::<Vec<_>>()
        })
        .map(|((_i, a), (_j, b), (_k, c), (_l, d), (_m, e))| [*a, *b, *c, *d, *e])
        .collect::<Vec<_>>()
}
