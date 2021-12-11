use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = Heightmap;

#[derive(Debug)]
struct Heightmap {
    rows: Vec<Vec<u8>>,
}

impl Heightmap {
    fn width(&self) -> usize {
        self.rows[0].len()
    }

    fn height(&self) -> usize {
        self.rows.len()
    }

    fn at(&self, x: usize, y: usize) -> u8 {
        self.rows[y][x]
    }

    fn low_points(&self) -> Vec<(usize, usize)> {
        let mut lp = vec![];
        for y in 0..self.height() {
            for x in 0..self.width() {
                let h = self.at(x, y);
                if self.adjacent(x, y).iter().all(|&(x, y)| self.at(x, y) > h) {
                    lp.push((x, y));
                }
            }
        }
        lp
    }

    fn adjacent(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let x = x as i32;
        let y = y as i32;
        let w = self.width() as i32;
        let h = self.height() as i32;
        [(1_i32, 0_i32), (0, 1), (-1, 0), (0, -1)]
            .iter()
            .map(|&(rx, ry)| (x + rx, y + ry))
            .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h)
            .map(|(x, y)| (x as usize, y as usize))
            .collect()
    }

    fn basins(&self) -> Vec<usize> {
        let mut basins = vec![];

        for (x, y) in self.low_points() {
            let mut basin = vec![];

            let mut queue = VecDeque::new();
            queue.push_back((x, y));

            while let Some((x, y)) = queue.pop_front() {
                if basin.contains(&(x, y)) {
                    continue;
                }
                if self.at(x, y) == 9 {
                    continue;
                }

                basin.push((x, y));

                for pos in self.adjacent(x, y) {
                    queue.push_back(pos);
                }
            }

            basins.push(basin);
        }

        basins.iter().map(|b| b.len()).collect()
    }
}

fn part1(input: &Input) -> u32 {
    input
        .low_points()
        .iter()
        .map(|&(x, y)| input.at(x, y) as u32 + 1)
        .sum()
}

fn part2(input: &Input) -> usize {
    let mut basins = input.basins();
    basins.sort();

    basins.iter().rev().take(3).product()
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
    let rows = reader
        .lines()
        .map(|line| Ok(line?.bytes().map(|c| (c - b'0') as u8).collect::<Vec<_>>()))
        .collect::<Result<Vec<_>>>()?;
    Ok(Heightmap { rows })
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
        2199943210
        3987894921
        9856789892
        8767896789
        9899965678";

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
        assert_eq!(part1(&as_input(INPUT)?), 15);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 1134);
        Ok(())
    }
}
