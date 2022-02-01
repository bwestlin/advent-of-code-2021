use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = [usize; 2];

fn part1(input: &Input) -> usize {
    let mut player_pos = [input[0] - 1, input[1] - 1];
    let mut player_score = [0, 0];

    let mut player_turn = 0;
    let mut dice = (1..=100).cycle();

    let mut cnt = 0;
    while !player_score.iter().any(|&s| s >= 1000) {
        let roll = dice.by_ref().take(3).sum::<usize>();
        player_pos[player_turn] = (player_pos[player_turn] + roll) % 10;
        player_score[player_turn] += player_pos[player_turn] + 1;

        #[cfg(feature = "print")]
        println!(
            "Player {} rolls {} and moves to space {} for a total score of {}.",
            player_turn + 1,
            roll,
            player_pos[player_turn] + 1,
            player_score[player_turn]
        );

        player_turn = (player_turn + 1) % 2;

        cnt += 3;
    }

    player_score.iter().min().unwrap_or(&0) * cnt
}

fn part2(input: &Input) -> usize {
    let player_pos = [input[0] - 1, input[1] - 1];
    let player_score = [0, 0];
    let player_turn = 0;

    let mut queue = VecDeque::new();
    queue.push_back((player_pos, player_score, player_turn, 1));

    let mut wins = [0_usize, 0];
    let mut roll_freqs = BTreeMap::<usize, usize>::new();
    for roll1 in 1..=3 {
        for roll2 in 1..=3 {
            for roll3 in 1..=3 {
                let roll = roll1 + roll2 + roll3;
                *roll_freqs.entry(roll).or_default() += 1;
            }
        }
    }

    while let Some((player_pos, player_score, player_turn, factor)) = queue.pop_front() {
        let next_player_turn = (player_turn + 1) % 2;

        for (roll, freq) in &roll_freqs {
            let mut player_pos = player_pos.clone();
            let mut player_score = player_score.clone();
            player_pos[player_turn] = (player_pos[player_turn] + roll) % 10;
            player_score[player_turn] += player_pos[player_turn] + 1;

            if player_score[player_turn] >= 21 {
                wins[player_turn] += factor * freq;
                continue;
            }

            queue.push_front((player_pos, player_score, next_player_turn, factor * freq));
        }
    }

    wins.into_iter().max().unwrap_or(0)
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
    let positions = reader
        .lines()
        .map(|line| {
            let line = line?;
            let splits = line.split(": ");

            Ok(splits
                .skip(1)
                .next()
                .context(format!("Missing :"))?
                .parse::<usize>()?)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok([positions[0], positions[1]])
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
        Player 1 starting position: 4
        Player 2 starting position: 8";

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
        assert_eq!(part1(&as_input(INPUT)?), 739785);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 444356092776315);
        Ok(())
    }
}
