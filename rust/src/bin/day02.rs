use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use utils::measure;

type Input = Vec<Command>;

#[derive(Debug)]
struct Command {
    dir: Direction,
    units: i32,
}

#[derive(Debug)]
enum Direction {
    Forward,
    Up,
    Down,
}

fn part1(input: &Input) -> i32 {
    let mut pos = 0;
    let mut depth = 0;

    for Command { dir, units } in input {
        match dir {
            Direction::Forward => pos += units,
            Direction::Down => depth += units,
            Direction::Up => depth -= units,
        }
    }

    pos * depth
}

fn part2(input: &Input) -> i32 {
    let mut pos = 0;
    let mut depth = 0;
    let mut aim = 0;

    for Command { dir, units } in input {
        match dir {
            Direction::Forward => {
                pos += units;
                depth += aim * units;
            }
            Direction::Down => aim += units,

            Direction::Up => aim -= units,
        }
    }

    pos * depth
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2: {}", part2(&input));
        Ok(())
    })
}

impl FromStr for Command {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(' ');
        Ok(Command {
            dir: splits
                .next()
                .ok_or_else(|| anyhow!("Direction missing"))?
                .parse::<Direction>()?,
            units: splits
                .next()
                .ok_or_else(|| anyhow!("Units missing"))?
                .parse::<i32>()?,
        })
    }
}

impl FromStr for Direction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dir = match s {
            "forward" => Direction::Forward,
            "down" => Direction::Down,
            "up" => Direction::Up,
            _ => return Err(anyhow!("Unknown direction {}", s)),
        };
        Ok(dir)
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<Command>()?))
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
        forward 5
        down 5
        forward 8
        up 3
        down 8
        forward 2";

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
        assert_eq!(part1(&as_input(INPUT)?), 150);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 900);
        Ok(())
    }
}
