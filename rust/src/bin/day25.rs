use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridState {
    Empty,
    CucumberEast,
    CucumberSouth,
}

impl GridState {
    fn from_char(c: char) -> Self {
        match c {
            '.' => GridState::Empty,
            '>' => GridState::CucumberEast,
            'v' => GridState::CucumberSouth,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid {
    rows: Vec<Vec<GridState>>,
}

impl Grid {
    fn width(&self) -> usize {
        self.rows[0].len()
    }

    fn height(&self) -> usize {
        self.rows.len()
    }

    fn step(&self) -> Self {
        let prev = self;
        let mut next = self.empty();

        // Update the >
        for y in 0..self.height() {
            for x in 0..self.width() {
                let gs = &prev.rows[y][x];

                match gs {
                    GridState::CucumberEast => {
                        let (n_x, n_y) = ((x + 1) % self.width(), y);

                        if let GridState::Empty = prev.rows[n_y][n_x] {
                            next.rows[n_y][n_x] = *gs;
                        } else {
                            next.rows[y][x] = *gs;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update the v
        for y in 0..self.height() {
            for x in 0..self.width() {
                let gs = &prev.rows[y][x];

                match gs {
                    GridState::CucumberSouth => {
                        let (n_x, n_y) = (x, (y + 1) % self.height());

                        let n_prev_gs = prev.rows[n_y][n_x];
                        let n_next_gs = next.rows[n_y][n_x];

                        if n_next_gs == GridState::Empty && n_prev_gs != GridState::CucumberSouth {
                            next.rows[n_y][n_x] = *gs;
                        } else {
                            next.rows[y][x] = *gs;
                        }
                    }
                    _ => {}
                }
            }
        }

        next
    }

    fn empty(&self) -> Self {
        let mut rows = Vec::with_capacity(self.height());
        for _ in 0..self.height() {
            rows.push(vec![GridState::Empty; self.width()]);
        }
        Self { rows }
    }

    #[cfg(feature = "print")]
    fn print(&self) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                let c = match self.rows[y][x] {
                    GridState::Empty => '.',
                    GridState::CucumberEast => '>',
                    GridState::CucumberSouth => 'v',
                };
                print!("{}", c);
            }

            println!();
        }
    }
}

fn part1(input: &Input) -> usize {
    let mut grid = input.clone();

    #[cfg(feature = "print")]
    {
        println!("Initial state:");
        grid.print();
    }

    for step in 1.. {
        let next = grid.step();

        if next == grid {
            return step;
        }
        grid = next;

        #[cfg(feature = "print")]
        {
            println!();
            println!("After {} steps:", step);
            grid.print();
        }
    }

    unreachable!()
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        Ok(())
    })
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    let mut rows = vec![];

    for line in reader.lines() {
        let line = line?;

        let row = line.chars().map(GridState::from_char).collect();
        rows.push(row);
    }

    Ok(Grid { rows })
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
        v...>>.vv>
        .vv>>.vv..
        >>.>v>...v
        >>v>>.>.v.
        v>v.vv.v..
        >.>>..v...
        .vv..>.>v.
        v.v..>>v.v
        ....v..v.>";

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
        assert_eq!(part1(&as_input(INPUT)?), 58);
        Ok(())
    }
}
