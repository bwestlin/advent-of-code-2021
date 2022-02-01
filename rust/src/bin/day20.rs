use std::cmp::max;
use std::cmp::min;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

#[derive(Debug)]
struct Input {
    algorithm: Vec<u8>,
    lit_pixels: Vec<Pos>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn add(&self, other: &Pos) -> Self {
        let x = self.x + other.x;
        let y = self.y + other.y;
        Self { x, y }
    }
}

fn solve(input: &Input) -> (usize, usize) {
    let mut lit_pixels = input.lit_pixels.iter().cloned().collect::<HashSet<_>>();
    let offsets = [
        Pos::new(-1, -1),
        Pos::new(0, -1),
        Pos::new(1, -1),
        Pos::new(-1, 0),
        Pos::new(0, 0),
        Pos::new(1, 0),
        Pos::new(-1, 1),
        Pos::new(0, 1),
        Pos::new(1, 1),
    ];
    let mut outside_lit = false;
    let mut res = vec![lit_pixels.len()];

    for _ in 0..50 {
        let (lit_min_x, lit_max_x, lit_min_y, lit_max_y) = lit_pixels.iter().fold(
            (0, 0, 0, 0),
            |(min_x, max_x, min_y, max_y), Pos { x, y }| {
                (
                    min(min_x, *x),
                    max(max_x, *x),
                    min(min_y, *y),
                    max(max_y, *y),
                )
            },
        );

        let mut next_lit_pixels = HashSet::new();
        let off = 1;
        for y in (lit_min_y - off)..=(lit_max_y + off) {
            for x in (lit_min_x - off)..=(lit_max_x + off) {
                let mut num = 0;
                for i in 0..9 {
                    let check = Pos::new(x, y).add(&offsets[i]);
                    if lit_pixels.contains(&check)
                        || outside_lit
                            && (check.x < lit_min_x
                                || check.y < lit_min_y
                                || check.x > lit_max_x
                                || check.y > lit_max_y)
                    {
                        num |= 1 << (8 - i);
                    }
                }

                if input.algorithm[num] == b'#' {
                    next_lit_pixels.insert(Pos::new(x, y));
                }
            }
        }

        lit_pixels = next_lit_pixels;

        if outside_lit {
            outside_lit = input.algorithm[0b111111111] == b'#';
        } else {
            outside_lit = input.algorithm[0] == b'#';
        }

        res.push(lit_pixels.len());

        #[cfg(feature = "print")]
        print(&lit_pixels, outside_lit);
    }

    (res[2], res[50])
}

#[cfg(feature = "print")]
fn print(lit_pixels: &HashSet<Pos>, outside_lit: bool) {
    let (lit_min_x, lit_max_x, lit_min_y, lit_max_y) = lit_pixels.iter().fold(
        (0, 0, 0, 0),
        |(min_x, max_x, min_y, max_y), Pos { x, y }| {
            (
                min(min_x, *x),
                max(max_x, *x),
                min(min_y, *y),
                max(max_y, *y),
            )
        },
    );

    for y in (lit_min_y - 5)..=(lit_max_y + 5) {
        for x in (lit_min_x - 5)..=(lit_max_x + 5) {
            let c = if lit_pixels.contains(&Pos::new(x, y)) {
                '#'
            } else {
                '.'
            };

            let c = if x < lit_min_x || y < lit_min_y || x > lit_max_x || y > lit_max_y {
                if outside_lit {
                    '%'
                } else {
                    ','
                }
            } else {
                c
            };
            print!("{}", c);
        }
        println!();
    }
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

    let mut algorithm = String::new();
    for line in lines.by_ref() {
        let line = line?;

        if line.is_empty() {
            break;
        }

        algorithm.push_str(line.as_str());
    }

    let algorithm = algorithm.into_bytes();

    let mut lit_pixels: Vec<Pos> = vec![];

    for (y, line) in lines.enumerate() {
        let line = line?;

        for (x, c) in line.chars().enumerate() {
            if c == '#' {
                lit_pixels.push(Pos::new(x as i32, y as i32));
            }
        }
    }

    Ok(Input {
        algorithm,
        lit_pixels,
    })
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
        ..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..##
        #..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###
        .######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#.
        .#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#.....
        .#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#..
        ...####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.....
        ..##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

        #..#.
        #....
        ##..#
        ..#..
        ..###";

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
        assert_eq!(solve(&as_input(INPUT)?).0, 35);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 3351);
        Ok(())
    }
}
