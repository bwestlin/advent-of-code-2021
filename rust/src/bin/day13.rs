use std::collections::{HashSet, VecDeque};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

use anyhow::{Context, Result};

use utils::measure;

type Input = TransparentPaper;

#[derive(Debug, Clone)]
struct TransparentPaper {
    dots: HashSet<Pos>,
    folds: VecDeque<Fold>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
enum Fold {
    Vertical(usize),
    Horizontal(usize),
}

impl TransparentPaper {
    fn fold(&mut self) -> Option<()> {
        self.folds.pop_front().map(|fold| {
            match fold {
                Fold::Vertical(fold_x) => {
                    self.fold_decide_translate(
                        |dot| dot.x <= fold_x,
                        |Pos { x, y }| Pos {
                            x: fold_x - (x - fold_x),
                            y,
                        },
                    );
                }
                Fold::Horizontal(fold_y) => {
                    self.fold_decide_translate(
                        |dot| dot.y <= fold_y,
                        |Pos { x, y }| Pos {
                            x,
                            y: fold_y - (y - fold_y),
                        },
                    );
                }
            }
            ()
        })
    }

    fn fold_decide_translate(
        &mut self,
        fold_decide_fn: impl Fn(&&Pos) -> bool,
        fold_translate_fn: impl Fn(Pos) -> Pos,
    ) {
        let (mut stay, to_fold): (HashSet<Pos>, HashSet<Pos>) =
            self.dots.iter().partition(fold_decide_fn);

        for pos in to_fold {
            stay.insert(fold_translate_fn(pos));
        }

        self.dots = stay;
    }

    fn print(&self) {
        let (max_x, max_y) = self.dots.iter().fold((0, 0), |(mx, my), pos| {
            (std::cmp::max(mx, pos.x), std::cmp::max(my, pos.y))
        });

        for y in 0..=max_y {
            for x in 0..=max_x {
                let c = self.dots.get(&Pos { x, y }).map(|_| '#').unwrap_or('.');
                print!("{}", c);
            }
            println!();
        }
    }
}

fn part1(input: &Input) -> usize {
    let mut tpaper = input.clone();
    tpaper.fold();
    tpaper.dots.len()
}

fn part2(input: &Input) {
    let mut tpaper = input.clone();
    while let Some(_) = tpaper.fold() {}
    tpaper.print();
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2:");
        part2(&input);
        Ok(())
    })
}

impl FromStr for Pos {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.splitn(2, ',');
        Ok(Pos {
            x: splits.next().context("No dot x")?.parse::<usize>()?,
            y: splits.next().context("No dot y")?.parse::<usize>()?,
        })
    }
}

impl FromStr for Fold {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(' ');
        let mut splits = splits.last().context("No fold input")?.split('=');
        let axis = splits.next().context("No fold axis")?;
        let val = splits.next().context("No fold value")?.parse::<usize>()?;
        Ok(match axis {
            "x" => Fold::Vertical(val),
            "y" => Fold::Horizontal(val),
            _ => unreachable!(),
        })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    let mut dots = HashSet::new();
    let mut folds = VecDeque::new();

    let mut lines = reader.lines();

    for line in lines.by_ref() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        dots.insert(line.parse::<Pos>()?);
    }

    for line in lines {
        let line = line?;
        let line = line.trim();
        folds.push_back(line.parse::<Fold>()?);
    }

    Ok(TransparentPaper { dots, folds })
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
        6,10
        0,14
        9,10
        0,3
        10,4
        4,11
        6,0
        6,12
        4,1
        0,13
        10,12
        3,4
        3,0
        8,4
        1,10
        2,14
        8,10
        9,0

        fold along y=7
        fold along x=5";

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
        assert_eq!(part1(&as_input(INPUT)?), 17);
        Ok(())
    }
}
