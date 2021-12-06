use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<u8>;

fn solve(input: &Input) -> (usize, usize) {
    let mut fishes_by_timer = [0_usize; 9];
    let mut daily_fishcount = [0_usize; 256];

    for &i in input {
        fishes_by_timer[i as usize] += 1;
    }

    for day in 0..256 {
        let new = fishes_by_timer[0];
        let resetted = fishes_by_timer[0];
        for tidx in 0..=7 {
            fishes_by_timer[tidx] = fishes_by_timer[tidx + 1];
        }
        fishes_by_timer[8] = new;
        fishes_by_timer[6] += resetted;
        daily_fishcount[day] = fishes_by_timer.iter().sum();
    }

    (daily_fishcount[80 - 1], daily_fishcount[256 - 1])
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
    Ok(reader
        .lines()
        .next()
        .unwrap()?
        .split(',')
        .map(|s| s.parse::<u8>().unwrap())
        .collect())
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

    const INPUT: &'static str = "3,4,3,1,2";

    fn as_input(s: &str) -> Result<Input> {
        read_input(BufReader::new(
            s.split('\n')
                .map(|s| s.trim())
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes(),
        ))
    }

    #[test]
    fn test_part1() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).0, 5934);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 26984457539);
        Ok(())
    }
}
