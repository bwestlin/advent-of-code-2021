use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<Connection>;

#[derive(Debug)]
struct Connection {
    caves: [String; 2],
}

fn count_paths(input: &Input, permit_double_visit: bool) -> usize {
    const START: u8 = 0;
    const END: u8 = 1;
    let mut name_to_id: HashMap<&str, u8> = [("start", START), ("end", END)].into_iter().collect();
    let mut connections = (0..256).map(|_| HashSet::<u8>::new()).collect::<Vec<_>>();
    let mut big_caves = [false; 256];

    for Connection { caves } in input {
        for cave in caves {
            let id = if let Some(&id) = name_to_id.get(cave.as_str()) {
                id
            } else {
                let id = name_to_id.len() as u8;
                name_to_id.insert(cave, id);
                id
            };
            if cave.chars().all(|c| c.is_uppercase()) {
                big_caves[id as usize] = true;
            }
        }
        for (a, b) in [(0, 1), (1, 0)] {
            let id = name_to_id[&caves[a].as_str()];
            let target = name_to_id[&caves[b].as_str()];
            connections[id as usize].insert(target);
        }
    }

    let mut paths: HashSet<Vec<u8>> = HashSet::new();

    let mut queue: VecDeque<(Vec<u8>, bool)> = VecDeque::new();
    for &cave in &connections[START as usize] {
        queue.push_back((vec![START, cave], false));
    }

    while let Some((caves, has_double_visit)) = queue.pop_front() {
        if let Some(&last_cave) = caves.last() {
            for &next_cave in &connections[last_cave as usize] {
                if next_cave == START {
                    continue;
                }

                if next_cave == END {
                    let mut caves = caves.clone();
                    caves.push(next_cave);
                    paths.insert(caves);
                    continue;
                }

                let is_big = big_caves[next_cave as usize];

                if is_big
                    || (permit_double_visit && !has_double_visit)
                    || !caves.contains(&next_cave)
                {
                    let mut next_caves = caves.clone();
                    next_caves.push(next_cave);
                    queue.push_front((
                        next_caves,
                        has_double_visit || (!is_big && caves.contains(&next_cave)),
                    ));
                }
            }
        }
    }

    paths.len()
}

fn part1(input: &Input) -> usize {
    count_paths(input, false)
}

fn part2(input: &Input) -> usize {
    count_paths(input, true)
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2: {}", part2(&input));
        Ok(())
    })
}

impl FromStr for Connection {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split('-').collect::<Vec<_>>();
        Ok(Connection {
            caves: [splits[0].to_owned(), splits[1].to_owned()],
        })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<Connection>()?))
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
        start-A
        start-b
        A-c
        A-b
        b-d
        A-end
        b-end";

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
        assert_eq!(part1(&as_input(INPUT)?), 10);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 36);
        Ok(())
    }
}
