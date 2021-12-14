use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Pair = [char; 2];

#[derive(Debug)]
struct Input {
    polymer_template: Vec<char>,
    pair_insertion: HashMap<Pair, char>,
}

fn solve(input: &Input) -> (i64, i64) {
    let mut pair_freqs = HashMap::<Pair, i64>::new();
    for pair in input.polymer_template.windows(2) {
        let pair = [pair[0], pair[1]];
        *pair_freqs.entry(pair).or_default() += 1;
    }

    let mut results = vec![];

    for step in 1..=40 {
        let mut next_pair_freqs = HashMap::<Pair, i64>::new();
        for (pair, freq) in pair_freqs {
            let insert_elem = input.pair_insertion[&pair];
            let pair1 = [pair[0], insert_elem];
            let pair2 = [insert_elem, pair[1]];
            *next_pair_freqs.entry(pair1).or_default() += freq;
            *next_pair_freqs.entry(pair2).or_default() += freq;
        }

        pair_freqs = next_pair_freqs;

        if step == 10 || step == 40 {
            let mut elem_pair_freqs = HashMap::<char, i64>::new();
            for (pair, freq) in &pair_freqs {
                for &c in pair {
                    *elem_pair_freqs.entry(c).or_default() += freq;
                }
            }
            let elem_freqs = elem_pair_freqs
                .into_iter()
                .map(|(c, f)| (c, (f / 2) + (f % 2)))
                .collect::<HashMap<_, _>>();

            let max_freq = elem_freqs.values().max().unwrap_or(&0);
            let min_freq = elem_freqs.values().min().unwrap_or(&0);
            results.push(max_freq - min_freq);
        }
    }

    (results[0], results[1])
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        let (part1, part2) = solve(&input);
        println!("Part1: {}", part1);
        println!("Part2: {}", part2);
        Ok(())
    })
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    let mut lines = reader.lines();

    let polymer_template = lines
        .next()
        .context("Missing polymer template")??
        .chars()
        .collect::<Vec<_>>();
    lines.next();

    let mut pair_insertion = HashMap::new();
    for line in lines {
        let line = line?;

        let mut splits = line.split(' ');
        let from = splits
            .next()
            .context("Missing rule from")?
            .chars()
            .collect::<Vec<_>>();
        let from = [from[0], from[1]];
        splits.next();
        let to = splits
            .next()
            .context("Missing rule to")?
            .chars()
            .next()
            .context("Missing rule to char")?;

        pair_insertion.insert(from, to);
    }

    Ok(Input {
        polymer_template,
        pair_insertion,
    })
}

fn input() -> Result<Input> {
    let path = env::args()
        .skip(1)
        .next()
        .with_context(|| format!("No input file given"))?;
    read_input(BufReader::new(File::open(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &'static str = "
        NNCB

        CH -> B
        HH -> N
        CB -> H
        NH -> C
        HB -> C
        HC -> B
        HN -> C
        NN -> C
        BH -> H
        NC -> B
        NB -> B
        BN -> B
        BB -> N
        BC -> B
        CC -> N
        CN -> C";

    fn as_input(s: &str) -> Result<Input> {
        read_input(BufReader::new(
            s.split('\n')
                .skip(1)
                .map(|s| s.trim())
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes(),
        ))
    }

    #[test]
    fn test_part1() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).0, 1588);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 2188189693529);
        Ok(())
    }
}
