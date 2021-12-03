use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};
use utils::measure;

type Input = Vec<Vec<u8>>;

fn part1(input: &Input) -> i32 {
    let n_numbers = input.len() as i32;
    let n_bits = input[0].len();
    let mut bit_counts = vec![0; n_bits];

    for n in input {
        for (i, &c) in n.iter().enumerate() {
            bit_counts[i] += (c == b'1') as i32;
        }
    }

    let mut gamma_rate = 0;
    for (i, bc) in bit_counts.iter().rev().enumerate() {
        if (bc << 1) > n_numbers {
            gamma_rate |= 1 << i;
        }
    }

    let mask = (0..n_bits).fold(0, |m, _| (m << 1) | 1);
    let epsilon_rate = (!gamma_rate) & mask;

    gamma_rate * epsilon_rate
}

enum BitCriteria {
    MostCommon,
    LeastCommon,
}

fn search(input: &Input, bit_criteria: BitCriteria) -> Option<i32> {
    let n_numbers = input.len();
    let n_bits = input[0].len();

    let mut queued = (Some((0..n_numbers).collect::<Vec<_>>()), 0);
    let mut found = None;
    while let (Some(n_idxs), pos) = queued {
        if n_idxs.len() == 1 {
            found = Some(n_idxs[0]);
            break;
        }
        if pos >= n_bits {
            break;
        }

        let (ones, zeroes): (Vec<usize>, Vec<usize>) =
            n_idxs.iter().partition(|&&idx| input[idx][pos] == b'1');

        let (most_common, least_common) = if ones.len() >= zeroes.len() {
            (ones, zeroes)
        } else {
            (zeroes, ones)
        };
        let next = match bit_criteria {
            BitCriteria::MostCommon => most_common,
            BitCriteria::LeastCommon => least_common,
        };

        queued = (Some(next), pos + 1)
    }

    found
        .and_then(|found_idx| std::str::from_utf8(&input[found_idx]).ok())
        .and_then(|s| i32::from_str_radix(s, 2).ok())
}

fn part2(input: &Input) -> i32 {
    let oxygen_generator_rating = search(input, BitCriteria::MostCommon).unwrap_or(0);
    let co2_scrubbing_rating = search(input, BitCriteria::LeastCommon).unwrap_or(0);

    oxygen_generator_rating * co2_scrubbing_rating
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2: {}", part2(&input));
        Ok(())
    })
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.as_bytes().into_iter().cloned().collect()))
        .collect()
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
        00100
        11110
        10110
        10111
        10101
        01111
        00111
        11100
        10000
        11001
        00010
        01010";

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
        assert_eq!(part1(&as_input(INPUT)?), 198);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 230);
        Ok(())
    }
}
