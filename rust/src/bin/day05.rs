use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<Line>;

#[derive(Debug)]
struct Line {
    p1: Point,
    p2: Point,
}

impl Line {
    fn is_straight(&self) -> bool {
        self.p1.x == self.p2.x || self.p1.y == self.p2.y
    }

    fn points(&self) -> Vec<Point> {
        let Line { p1, p2 } = self;
        let mut points = vec![];
        if p1.x == p2.x {
            let x = p1.x;
            let mut y1 = p1.y;
            let mut y2 = p2.y;
            if y1 > y2 {
                std::mem::swap(&mut y1, &mut y2);
            }
            for y in y1..=y2 {
                points.push(Point { x, y });
            }
        } else if p1.y == p2.y {
            let y = p1.y;
            let mut x1 = p1.x;
            let mut x2 = p2.x;
            if x1 > x2 {
                std::mem::swap(&mut x1, &mut x2);
            }
            for x in x1..=x2 {
                points.push(Point { x, y });
            }
        } else {
            let mut x1 = p1.x;
            let mut x2 = p2.x;
            let mut y1 = p1.y;
            let mut y2 = p2.y;
            if x1 > x2 {
                std::mem::swap(&mut x1, &mut x2);
                std::mem::swap(&mut y1, &mut y2);
            }
            let mut y = y1;
            let y_dir = if y2 > y1 { 1 } else { -1 };
            for x in x1..=x2 {
                points.push(Point { x, y });
                y += y_dir;
            }
        }

        points
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn part1(input: &Input) -> usize {
    let mut map_points: HashMap<Point, i32> = HashMap::new();

    for line in input {
        if !line.is_straight() {
            continue;
        }

        for point in line.points() {
            *map_points.entry(point).or_default() += 1;
        }
    }

    map_points.values().filter(|&&v| v >= 2).count()
}

fn part2(input: &Input) -> usize {
    let mut map_points: HashMap<Point, i32> = HashMap::new();

    for line in input {
        for point in line.points() {
            *map_points.entry(point).or_default() += 1;
        }
    }

    map_points.values().filter(|&&v| v >= 2).count()
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2: {}", part2(&input));
        Ok(())
    })
}

impl FromStr for Line {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(' ');
        let p1 = splits.next().unwrap().parse().unwrap();
        splits.next();
        let p2 = splits.next().unwrap().parse().unwrap();

        Ok(Line { p1, p2 })
    }
}

impl FromStr for Point {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(',');
        let x = splits.next().unwrap().parse().unwrap();
        let y = splits.next().unwrap().parse().unwrap();
        Ok(Point { x, y })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<Line>()?))
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
        0,9 -> 5,9
        8,0 -> 0,8
        9,4 -> 3,4
        2,2 -> 2,1
        7,0 -> 7,4
        6,4 -> 2,0
        0,9 -> 2,9
        3,4 -> 1,4
        0,0 -> 8,8
        5,5 -> 8,2";

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
        assert_eq!(part1(&as_input(INPUT)?), 5);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 12);
        Ok(())
    }
}
