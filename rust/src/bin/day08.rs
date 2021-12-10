use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

use utils::measure;

type Input = Vec<Entry>;

#[derive(Debug)]
struct Entry {
    signal_patterns: Vec<String>,
    output_values: Vec<String>,
}

impl Entry {
    const DIGITS: [&'static str; 10] = [
        "abcefg",  // 0
        "cf",      // 1
        "acdeg",   // 2
        "acdfg",   // 3
        "bcdf",    // 4
        "abdfg",   // 5
        "abdefg",  // 6
        "acf",     // 7
        "abcdefg", // 8
        "abcdfg",  // 9
    ];

    fn unique_output_values(&self) -> i32 {
        let mut unique = 0;
        for o in &self.output_values {
            let len = o.len();
            if len == 2 || len == 3 || len == 4 || len == 7 {
                unique += 1;
            }
        }
        unique
    }

    fn digits(&self) -> Vec<String> {
        let seg_1 = self.signal_patterns.iter().find(|s| s.len() == 2).unwrap();
        let seg_4 = self.signal_patterns.iter().find(|s| s.len() == 4).unwrap();
        let seg_7 = self.signal_patterns.iter().find(|s| s.len() == 3).unwrap();

        let mut freqs: BTreeMap<char, usize> = BTreeMap::new();
        for sig in &self.signal_patterns {
            for ch in sig.chars() {
                *freqs.entry(ch).or_default() += 1;
            }
        }

        let mut mapping = BTreeMap::new();
        mapping.insert('e', *freqs.iter().find(|(_, &f)| f == 4).unwrap().0);
        mapping.insert('b', *freqs.iter().find(|(_, &f)| f == 6).unwrap().0);
        mapping.insert('f', *freqs.iter().find(|(_, &f)| f == 9).unwrap().0);

        mapping.insert(
            'a',
            seg_7
                .chars()
                .filter(|&c| !seg_1.contains(c))
                .next()
                .unwrap(),
        );
        mapping.insert(
            'c',
            *freqs
                .iter()
                .find(|(&ch, &f)| f == 8 && ch != mapping[&'a'])
                .unwrap()
                .0,
        );

        mapping.insert(
            'd',
            *freqs
                .iter()
                .find(|(&ch, &f)| f == 7 && seg_4.contains(ch))
                .unwrap()
                .0,
        );
        mapping.insert(
            'g',
            *freqs
                .iter()
                .find(|(&ch, &f)| f == 7 && ch != mapping[&'d'])
                .unwrap()
                .0,
        );

        let known_set = &self
            .signal_patterns
            .iter()
            .enumerate()
            .map(|(i, s)| (s.chars().collect::<HashSet<_>>(), i))
            .collect::<Vec<_>>();

        Entry::DIGITS
            .iter()
            .map(|&digit| {
                let set = digit.chars().map(|ch| mapping[&ch]).collect::<HashSet<_>>();
                known_set
                    .iter()
                    .find(|(s, _)| s == &set)
                    .map(|&(_, i)| &self.signal_patterns[i])
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    fn output(&self) -> i32 {
        let digits = self
            .digits()
            .iter()
            .map(|d| d.chars().collect::<HashSet<_>>())
            .collect::<Vec<_>>();

        let mut out = 0;
        let mut mul = 1;
        for ov in self.output_values.iter().rev() {
            let ov = ov.chars().collect::<HashSet<_>>();

            let digit = digits
                .iter()
                .enumerate()
                .find(|&(_, d)| d == &ov)
                .map(|(i, _)| i)
                .unwrap();

            out += (digit as i32) * mul;
            mul *= 10;
        }

        out
    }
}

fn solve(input: &Input) -> (i32, i32) {
    input.iter().fold((0, 0), |(p1, p2), e| {
        (p1 + e.unique_output_values(), p2 + e.output())
    })
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

impl FromStr for Entry {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split('|');
        let signal_patterns = splits
            .next()
            .ok_or_else(|| anyhow!(format!("no signal patterns")))?
            .trim()
            .split(' ')
            .map(|s| s.to_owned())
            .collect();
        let output_values = splits
            .next()
            .ok_or_else(|| anyhow!(format!("no output values")))?
            .trim()
            .split(' ')
            .map(|s| s.to_owned())
            .collect();

        Ok(Entry {
            signal_patterns,
            output_values,
        })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<Entry>()?))
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
        be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
        edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
        fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
        fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
        aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
        fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
        dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
        bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
        egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
        gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce";

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
        assert_eq!(solve(&as_input(INPUT)?).0, 26);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 61229);
        Ok(())
    }
}
