use std::error::Error;
use std::process;

use itertools::Itertools;

fn main() {
    let word_list = get_words().unwrap_or_else(|err| {
        eprintln!("Error fetching words: {err}");
        process::exit(1);
    });

    println!("Fetched words");

    let encoded = preprocess_words(word_list).unwrap_or_else(|err| {
        eprintln!("Error preprocessing words: {err}");
        process::exit(1);
    });

    println!("Preprocessed words");
    println!("Number of words: {}", encoded.len());

    let solutions = find_solutions(&encoded);

    println!("Found {} solutions (should be ~5)", solutions.len());

}

fn get_words() -> Result<Vec<String>, Box<dyn Error>> {
    const URL: &str = "https://raw.githubusercontent.com/tabatkins/wordle-list/main/words";
    let text = reqwest::blocking::get(URL)?.text()?;

    let words: Vec<String> = text.lines().map(String::from).collect();

    Ok(words)
    
}

fn encode_word(word: String) -> u32 {
    const ASCII_LOWER: [char; 26] = [
        'a', 'b', 'c', 'd', 'e', 
        'f', 'g', 'h', 'i', 'j', 
        'k', 'l', 'm', 'n', 'o',
        'p', 'q', 'r', 's', 't', 
        'u', 'v', 'w', 'x', 'y', 
        'z',
    ];

    const TWO: u32 = 2;

    let mut bytes: u32 = 0;

    for c in word.chars() {
        let index: usize = ASCII_LOWER.iter().position(|&r| r == c).unwrap();
        bytes |= TWO.pow(index as u32)
    }

    bytes
}

fn preprocess_words(raw_words: Vec<String>) -> Result<Vec<u32>, Box<dyn Error>> {
    let encoded_words: Vec<u32> = raw_words.iter()
        .map(|word| encode_word(word.to_string()))
        .unique()
        .filter(|n| n.count_ones() == 5)
        .collect();

    Ok(encoded_words)
}

fn find_solutions(encoded_words: &Vec<u32>) -> Vec<[u32; 5]> {
    let mut solutions = Vec::new();

    for (i, a) in encoded_words.iter().enumerate() {
        for (j, b) in encoded_words[i..].iter().enumerate() {
            if (a & b) != 0 {
                continue;
            }

            let ab: u32 = a | b;

            for (k, c) in encoded_words[j..].iter().enumerate() {
                if (ab & c) != 0 {
                    continue;
                }
    
                let abc: u32 = ab | c;

                for (l, d) in encoded_words[k..].iter().enumerate() {
                    if (abc & d) != 0 {
                        continue;
                    }
        
                    let abcd: u32 = abc | d;

                    for (m, e) in encoded_words[l..].iter().enumerate() {
                        if (abcd & e) != 0 {
                            continue;
                        }

                        let solution = [*a, *b, *c, *d, *e];

                        println!("Found solution {solution:?}");

                        solutions.push(solution);
                    }
                }
            }
        }
    }

    solutions
}