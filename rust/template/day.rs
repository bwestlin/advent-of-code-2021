use std::cmp;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::{Context, Result};
use regex::Regex;

use utils::measure;

type Input = Vec<Data>;

#[derive(Debug)]
struct Data {
    d: i32,
}

fn part1(input: &Input) -> i32 {
    dbg!(input);
    0
}

fn part2(input: &Input) -> i32 {
    // dbg!(input);
    0
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2: {}", part2(&input));
        Ok(())
    })
}

impl FromStr for Data {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Data {
            d: s.parse::<i32>()?,
        })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<Data>()?))
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

    const INPUT: &'static str = "";

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
        assert_eq!(part1(&as_input(INPUT)?), 1337);
        Ok(())
    }

    // #[test]
    // fn test_part2() -> Result<()> {
    //     assert_eq!(part2(&as_input(INPUT)?), 1337);
    //     Ok(())
    // }
}
