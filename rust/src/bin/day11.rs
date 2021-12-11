use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = OctoGrid;

const W: usize = 10;
const H: usize = 10;

#[derive(Debug, Default, Clone)]
struct OctoGrid {
    energy_levels: [[u8; W]; H],
}

impl OctoGrid {
    fn adjacent(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let x = x as i32;
        let y = y as i32;
        let w = W as i32;
        let h = H as i32;
        [
            (1_i32, 0_i32),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ]
        .into_iter()
        .map(|(rx, ry)| (x + rx, y + ry))
        .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h)
        .map(|(x, y)| (x as usize, y as usize))
        .collect()
    }

    fn step(&mut self) -> u64 {
        for y in 0..H {
            for x in 0..W {
                self.energy_levels[y][x] += 1;
            }
        }

        let mut flashed = [[false; W]; H];
        let mut any_flash = true;
        let mut flash_count = 0;

        while any_flash {
            any_flash = false;
            for y in 0..H {
                for x in 0..W {
                    if !flashed[y][x] && self.energy_levels[y][x] > 9 {
                        flashed[y][x] = true;
                        for (x, y) in self.adjacent(x, y) {
                            self.energy_levels[y][x] += 1;
                        }
                        any_flash = true;
                        flash_count += 1;
                    }
                }
            }
        }

        for y in 0..H {
            for x in 0..W {
                if self.energy_levels[y][x] > 9 {
                    self.energy_levels[y][x] = 0;
                }
            }
        }
        flash_count
    }

    fn is_synchronized(&self) -> bool {
        for y in 0..H {
            for x in 0..W {
                if self.energy_levels[y][x] != 0 {
                    return false;
                }
            }
        }
        true
    }
}

fn solve(input: &Input) -> (u64, usize) {
    let mut energy_levels = input.clone();

    let mut p1_total_flashes = 0;
    for step in 1.. {
        let flashes = energy_levels.step();
        if step <= 100 {
            p1_total_flashes += flashes;
        }

        if energy_levels.is_synchronized() {
            return (p1_total_flashes, step);
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
    let mut octo_grid = OctoGrid {
        ..Default::default()
    };

    for (y, line) in reader.lines().enumerate() {
        for (x, c) in line?.chars().enumerate() {
            octo_grid.energy_levels[y][x] = c as u8 - b'0';
        }
    }

    Ok(octo_grid)
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
        5483143223
        2745854711
        5264556173
        6141336146
        6357385478
        4167524645
        2176841721
        6882881134
        4846848554
        5283751526";

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
        assert_eq!(solve(&as_input(INPUT)?).0, 1656);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 195);
        Ok(())
    }
}
