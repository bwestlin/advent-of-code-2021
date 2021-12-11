use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<String>;

fn solve(input: &Input) -> (i32, i64) {
    let delimiters = [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')]
        .into_iter()
        .flat_map(|(o, c)| [(o, (Some(o), Some(c))), (c, (Some(o), Some(c)))].into_iter())
        .collect::<HashMap<_, _>>();

    let scores_illegal_char = [(')', 3_i32), (']', 57), ('}', 1197), ('>', 25137)]
        .into_iter()
        .collect::<HashMap<_, _>>();

    let scores_completion_char = [(')', 1_i64), (']', 2), ('}', 3), ('>', 4)]
        .into_iter()
        .collect::<HashMap<_, _>>();

    let mut illegal = vec![];
    let mut completion_scores = vec![];

    'outer: for i in input {
        let mut chunks = vec![];
        for c in i.chars() {
            let delim = delimiters[&c];

            match delim {
                (Some(oc), _) if oc == c => {
                    chunks.push(c);
                }
                (Some(oc), Some(cc)) if cc == c => {
                    let popped = chunks.pop();
                    if popped != Some(oc) {
                        illegal.push(c);
                        continue 'outer;
                    }
                }
                _ => unreachable!(),
            }
        }

        if !chunks.is_empty() {
            let mut completed_by = String::new();
            for c in chunks.iter().rev() {
                if let (_, Some(cc)) = delimiters[&c] {
                    completed_by.push(cc);
                }
            }

            let mut completion_score = 0_i64;
            for c in completed_by.chars() {
                let score = scores_completion_char[&c];
                completion_score = completion_score * 5 + score;
            }
            completion_scores.push(completion_score);
        }
    }

    completion_scores.sort();

    (
        illegal.iter().map(|c| scores_illegal_char[c]).sum(),
        completion_scores[completion_scores.len() / 2],
    )
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
    reader.lines().map(|line| Ok(line?)).collect()
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
        [({(<(())[]>[[{[]{<()<>>
        [(()[<>])]({[<{<<[]>>(
        {([(<{}[<>[]}>{[]{[(<()>
        (((({<>}<{<{<>}{[]{[]{}
        [[<[([]))<([[{}[[()]]]
        [{[{({}]{}}([{[{{{}}([]
        {<[[]]>}<{[{[{[]{()[[[]
        [<(<(<(<{}))><([]([]()
        <{([([[(<>()){}]>(<<{{
        <{([{{}}[<[[[<>{}]]]>[]]";

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
        assert_eq!(solve(&as_input(INPUT)?).0, 26397);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 288957);
        Ok(())
    }
}
