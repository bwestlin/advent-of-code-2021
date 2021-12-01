use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<i32>;

fn count_measurement_increases(mut measurements: impl Iterator<Item = i32>) -> i32 {
    let first = measurements.next().unwrap();
    let (_, n_increases) =
        measurements.fold((first, 0), |(prev, acc), m| (m, acc + (m > prev) as i32));
    n_increases
}

fn part1(input: &Input) -> i32 {
    count_measurement_increases(input.into_iter().cloned())
}

fn part2(input: &Input) -> i32 {
    count_measurement_increases(input.windows(3).map(|m| m.iter().sum::<i32>()))
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
        .map(|line| Ok(line?.parse::<i32>()?))
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
    199
    200
    208
    210
    200
    207
    240
    269
    260
    263";

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
        assert_eq!(part1(&as_input(INPUT)?), 7);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 5);
        Ok(())
    }
}
