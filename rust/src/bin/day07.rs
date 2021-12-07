use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{anyhow, Context, Result};

use utils::measure;

type Input = Vec<i32>;

fn least_fuel<F>(positions: &Vec<i32>, fuel_cost_per_steps: F) -> i32
where
    F: Fn(i32) -> i32,
{
    let max_pos = *positions.iter().max().unwrap();
    let mut least_fuel = 0;

    for pos in 0..max_pos {
        let mut fuel = 0;
        for p in positions {
            let steps = (p - pos).abs();
            fuel += fuel_cost_per_steps(steps);
        }
        if least_fuel == 0 || fuel < least_fuel {
            least_fuel = fuel;
        }
    }

    least_fuel
}

fn part1(input: &Input) -> i32 {
    least_fuel(input, |steps| steps)
}

fn part2(input: &Input) -> i32 {
    let max_pos = *input.iter().max().unwrap();

    let (fuel_per_steps, _) =
        (1..(max_pos + 2)).fold((vec![], 0), |(mut fuel_per_step, fuel), pos| {
            fuel_per_step.push(fuel);
            (fuel_per_step, fuel + pos)
        });

    least_fuel(input, |steps| fuel_per_steps[steps as usize])
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
        .next()
        .ok_or_else(|| anyhow!("Expected input row"))??
        .split(',')
        .map(|s| s.parse::<i32>().with_context(|| format!("Expected i32")))
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

    const INPUT: &'static str = "16,1,2,0,4,2,7,1,2,14";

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
        assert_eq!(part1(&as_input(INPUT)?), 37);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 168);
        Ok(())
    }
}
