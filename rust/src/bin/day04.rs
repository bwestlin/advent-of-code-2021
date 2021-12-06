use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

#[derive(Debug)]
struct Input {
    numbers: Vec<u8>,
    boards: Vec<Board>,
}

struct Drawn {
    numbers: u128,
}

impl Drawn {
    fn new() -> Self {
        Self { numbers: 0 }
    }
    fn draw(&mut self, n: u8) {
        self.numbers |= 1 << n;
    }
    fn contains(&self, n: &u8) -> bool {
        (self.numbers & (1 << n)) > 0
    }
}

#[derive(Debug)]
struct Board {
    numbers: Vec<u8>,
}

impl Board {
    fn is_winner(&self, drawn: &Drawn) -> bool {
        for i in 0..5 {
            if self
                .numbers
                .iter()
                .skip(i * 5)
                .take(5)
                .all(|n| drawn.contains(n))
            {
                return true;
            }

            //columnes
            if self
                .numbers
                .iter()
                .skip(i)
                .step_by(5)
                .all(|n| drawn.contains(n))
            {
                return true;
            }
        }
        false
    }

    fn score(&self, drawn: &Drawn) -> i32 {
        self.numbers
            .iter()
            .filter(|n| !drawn.contains(n))
            .map(|&n| n as i32)
            .sum()
    }
}

fn solve(Input { numbers, boards }: &Input) -> (i32, i32) {
    let mut first_score = None;
    let mut winning_boards_idx = vec![];
    let mut drawn = Drawn::new();

    for &n in numbers {
        drawn.draw(n);

        for (idx, board) in boards.iter().enumerate() {
            if winning_boards_idx.contains(&idx) {
                continue;
            }
            if board.is_winner(&drawn) {
                if first_score.is_none() {
                    first_score = Some(board.score(&drawn) * (n as i32));
                }

                winning_boards_idx.push(idx);
                if winning_boards_idx.len() == boards.len() {
                    return (first_score.unwrap_or(0), board.score(&drawn) * (n as i32));
                }
            }
        }
    }

    (0, 0)
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
    let lines = lines.by_ref();
    let numbers = lines
        .next()
        .map(|line| {
            line.unwrap()
                .split(',')
                .map(|s| s.parse::<u8>().unwrap())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![]);

    let mut boards = vec![];

    while let Some(_) = lines.next() {
        let numbers = lines
            .take(5)
            .map(|line| {
                line.unwrap()
                    .split(' ')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<u8>().unwrap())
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();
        boards.push(Board { numbers });
    }

    Ok(Input { numbers, boards })
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
        7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

        22 13 17 11  0
        8  2 23  4 24
        21  9 14 16  7
        6 10  3 18  5
        1 12 20 15 19

        3 15  0  2 22
        9 18 13 17  5
        19  8  7 25 23
        20 11 10 24  4
        14 21 16 12  6

        14 21 17 24  4
        10 16 15  9 19
        18  8 23 26 20
        22 11 13  6  5
        2  0 12  3  7";

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
        assert_eq!(solve(&as_input(INPUT)?).0, 4512);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 1924);
        Ok(())
    }
}
