use std::cmp::{max, min};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use utils::measure;

type Input = Area;

struct Area {
    x_range: [i32; 2],
    y_range: [i32; 2],
}

impl Area {
    fn within(&self, pos: &Pos) -> bool {
        let Area { x_range, y_range } = self;
        (x_range[0]..=x_range[1]).contains(&pos.x) && (y_range[0]..=y_range[1]).contains(&pos.y)
    }

    fn max_x(&self) -> i32 {
        max(self.x_range[0], self.x_range[1])
    }

    fn min_y(&self) -> i32 {
        min(self.y_range[0], self.y_range[1])
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Velocity {
    x: i32,
    y: i32,
}

impl Velocity {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

fn simulate(target: &Area, velocity: &Velocity) -> (bool, Vec<Pos>) {
    let mut trajectory = vec![];
    let mut pos = Pos::new(0, 0);
    let mut vel = *velocity;

    loop {
        pos = Pos::new(pos.x + vel.x, pos.y + vel.y);
        trajectory.push(pos);

        if target.within(&pos) {
            break (true, trajectory);
        }

        let n_v_x = if vel.x == 0 {
            0
        } else if vel.x > 0 {
            vel.x - 1
        } else {
            vel.x + 1
        };
        vel = Velocity::new(n_v_x, vel.y - 1);

        if pos.x > target.max_x() || pos.y < target.min_y() && vel.y < 0 {
            break (false, trajectory);
        }
    }
}

fn solve(input: &Input) -> (i32, i32) {
    let mut trajectories = vec![];

    let mut max_y = 0;
    let mut hits = 0;
    // TODO Lots of room for optimization here by reducing iterations
    for y in -1000..1000 {
        for x in 1..1000 {
            let vel = Velocity::new(x, y);
            let (hit, trajectory) = simulate(input, &vel);
            if hit {
                max_y = max(max_y, trajectory.iter().map(|p| p.y).max().unwrap_or(0));
                hits += 1;
                trajectories.push(trajectory);
            }
        }
    }

    #[cfg(feature = "print")]
    print(input, &trajectories);
    (max_y, hits)
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

impl FromStr for Area {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(":");
        splits.next();
        let mut splits = splits
            .next()
            .ok_or_else(|| anyhow!("Missing :"))?
            .trim()
            .split(", ");

        let x_range = splits.next().unwrap().split("=").skip(1).next().unwrap();
        let x_range = x_range
            .split("..")
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<i32>>();
        let x_range = [x_range[0], x_range[1]];

        let y_range = splits.next().unwrap().split("=").skip(1).next().unwrap();
        let y_range = y_range
            .split("..")
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<i32>>();
        let y_range = [y_range[0], y_range[1]];

        Ok(Area { x_range, y_range })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    Ok(reader.lines().next().unwrap()?.parse::<Area>()?)
}

fn input() -> Result<Input> {
    let path = env::args()
        .skip(1)
        .next()
        .with_context(|| format!("No input file given"))?;
    read_input(BufReader::new(File::open(path)?))
}

#[cfg(feature = "print")]
fn print(target: &Area, trajectories: &Vec<Vec<Pos>>) {
    let traveled = trajectories
        .iter()
        .flat_map(|t| t.iter())
        .collect::<std::collections::HashSet<_>>();

    let start_pos = Pos::new(0, 0);
    let (min_x, max_x, min_y, max_y) = traveled
        .iter()
        .map(|Pos { x, y }| (x, y))
        .chain(target.x_range.iter().zip(target.y_range.iter()))
        .fold((0, 0, 0, 0), |(min_x, max_x, min_y, max_y), (x, y)| {
            (
                min(min_x, *x),
                max(max_x, *x),
                min(min_y, *y),
                max(max_y, *y),
            )
        });

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let p = Pos::new(x, y);
            let c = if traveled.contains(&p) {
                '#'
            } else if target.within(&p) {
                'T'
            } else if p == start_pos {
                'S'
            } else {
                '.'
            };

            print!("{}", c);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &'static str = "target area: x=20..30, y=-10..-5";

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
        assert_eq!(solve(&as_input(INPUT)?).0, 45);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 112);
        Ok(())
    }
}
