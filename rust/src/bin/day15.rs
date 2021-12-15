use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = Cavern;

#[derive(Debug)]
struct Cavern {
    risk_levels: Vec<Vec<u8>>,
}

impl Cavern {
    fn width(&self) -> usize {
        self.risk_levels[0].len()
    }

    fn height(&self) -> usize {
        self.risk_levels.len()
    }

    fn risk_level(&self, pos: &Pos) -> u8 {
        self.risk_levels[pos.y][pos.x]
    }

    fn adjacent(&self, x: usize, y: usize) -> Vec<Pos> {
        let x = x as i32;
        let y = y as i32;
        let w = self.width() as i32;
        let h = self.height() as i32;
        [(1_i32, 0_i32), (0, 1), (-1, 0), (0, -1)]
            .iter()
            .map(|&(rx, ry)| (x + rx, y + ry))
            .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h)
            .map(|(x, y)| Pos::new(x as usize, y as usize))
            .collect()
    }

    fn lowest_risk_path(&self) -> Vec<Pos> {
        #[derive(Clone, Debug)]
        struct Queued {
            pos: Pos,
            risk_level: i32,
        }
        impl Ord for Queued {
            fn cmp(&self, other: &Queued) -> Ordering {
                other.risk_level.cmp(&self.risk_level)
            }
        }
        impl PartialOrd for Queued {
            fn partial_cmp(&self, other: &Queued) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl PartialEq for Queued {
            fn eq(&self, other: &Queued) -> bool {
                self.risk_level == other.risk_level
            }
        }
        impl Eq for Queued {}

        let start_pos = Pos::new(0, 0);
        let end_pos = Pos::new(self.width() - 1, self.height() - 1);

        let mut heap = BinaryHeap::new();
        heap.push(Queued {
            pos: start_pos,
            risk_level: 0,
        });

        let mut visited = HashMap::<Pos, (i32, Option<Pos>)>::new();
        visited.insert(Pos::new(0, 0), (0, None));

        while let Some(Queued { pos, risk_level }) = heap.pop() {
            if pos == end_pos {
                break;
            }

            for adj in self.adjacent(pos.x, pos.y) {
                let risk_level = risk_level + self.risk_level(&adj) as i32;

                if let Some(&(visited_rl, _)) = visited.get(&adj) {
                    if risk_level >= visited_rl {
                        continue;
                    }
                }

                visited.insert(adj, (risk_level, Some(pos)));

                heap.push(Queued {
                    pos: adj,
                    risk_level,
                });
            }
        }

        let mut path = vec![];
        let mut next_pos = Some(end_pos);
        while let Some(pos) = next_pos {
            path.push(pos);
            next_pos = visited[&pos].1;
        }
        path.reverse();
        path
    }

    fn expand(&self, times: usize) -> Self {
        let w = self.width();
        let h = self.height();
        let mut risk_levels = vec![];

        for row in &self.risk_levels {
            risk_levels.push(
                row.iter()
                    .cycle()
                    .take(w * times)
                    .cloned()
                    .collect::<Vec<u8>>(),
            );
        }

        risk_levels.append(
            &mut risk_levels
                .iter()
                .cycle()
                .take(h * (times - 1))
                .cloned()
                .collect::<Vec<Vec<u8>>>(),
        );

        for ay in 0..times {
            for ax in 0..times {
                let add = ay as u8 + ax as u8;
                let tx = ax * self.width();
                let ty = ay * self.height();
                for y in 0..self.height() {
                    for x in 0..self.width() {
                        let x = x + tx;
                        let y = y + ty;
                        let rl = risk_levels[y][x] + add;
                        let rl = if rl > 9 { rl - 9 } else { rl };
                        risk_levels[y][x] = rl;
                    }
                }
            }
        }

        Cavern { risk_levels }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn new(x: usize, y: usize) -> Pos {
        Self { x, y }
    }
}

fn part1(input: &Input) -> i32 {
    let path = input.lowest_risk_path();
    path.iter()
        .skip(1)
        .map(|p| input.risk_level(p) as i32)
        .sum()
}

fn part2(input: &Input) -> i32 {
    let input = input.expand(5);
    let path = input.lowest_risk_path();
    path.iter()
        .skip(1)
        .map(|p| input.risk_level(p) as i32)
        .sum()
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
    let mut risk_levels = vec![];

    for line in reader.lines() {
        let mut row = vec![];
        for c in line?.chars() {
            row.push(c as u8 - b'0');
        }
        risk_levels.push(row);
    }

    Ok(Cavern { risk_levels })
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
        1163751742
        1381373672
        2136511328
        3694931569
        7463417111
        1319128137
        1359912421
        3125421639
        1293138521
        2311944581";

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
        assert_eq!(part1(&as_input(INPUT)?), 40);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 315);
        Ok(())
    }
}
