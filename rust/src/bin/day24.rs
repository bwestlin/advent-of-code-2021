use std::collections::{HashSet, VecDeque};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::{bail, Context, Result};

use utils::measure;

type Input = Vec<Instruction>;

#[derive(Debug, Clone, Copy)]
enum Variable {
    W,
    X,
    Y,
    Z,
}

impl Variable {
    fn ordinal(&self) -> usize {
        use Variable::*;
        match self {
            W => 0,
            X => 1,
            Y => 2,
            Z => 3,
        }
    }

    fn from_char(c: char) -> Option<Self> {
        use Variable::*;
        match c {
            'w' => Some(W),
            'x' => Some(X),
            'y' => Some(Y),
            'z' => Some(Z),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
enum VarOrLit {
    Variable(Variable),
    Literal(i64),
}

#[derive(Debug, Clone)]
enum Instruction {
    Inp(Variable),
    Add(Variable, VarOrLit),
    Mul(Variable, VarOrLit),
    Div(Variable, VarOrLit),
    Mod(Variable, VarOrLit),
    Eql(Variable, VarOrLit),
}

#[derive(Debug)]
struct ALU {
    variables: [i64; 4],
}

impl ALU {
    fn new() -> Self {
        Self { variables: [0; 4] }
    }

    fn read(&self, v: Variable) -> i64 {
        self.variables[v.ordinal()]
    }

    fn write(&mut self, v: Variable, value: i64) {
        self.variables[v.ordinal()] = value;
    }

    fn execute(&mut self, instructions: &Vec<Instruction>, mut inputs: VecDeque<i64>) {
        use Instruction::*;
        use VarOrLit::*;

        fn resolve(alu: &ALU, var_or_lit: &VarOrLit) -> i64 {
            match var_or_lit {
                &Variable(var) => alu.read(var),
                &Literal(lit) => lit,
            }
        }

        for ins in instructions {
            match ins {
                Inp(a) => {
                    self.variables[a.ordinal()] = inputs.pop_front().unwrap();
                }
                Add(a, b) => {
                    self.variables[a.ordinal()] += resolve(self, b);
                }
                Mul(a, b) => {
                    self.variables[a.ordinal()] *= resolve(self, b);
                }
                Div(a, b) => {
                    self.variables[a.ordinal()] /= resolve(self, b);
                }
                Mod(a, b) => {
                    self.variables[a.ordinal()] %= resolve(self, b);
                }
                Eql(a, b) => {
                    self.variables[a.ordinal()] =
                        (self.variables[a.ordinal()] == resolve(self, b)) as i64;
                }
            }
        }
    }
}

fn split_instructions(ins: &Vec<Instruction>) -> Vec<Vec<Instruction>> {
    let mut splitted = vec![];
    let mut current = vec![];

    for (i, ins) in ins.iter().enumerate() {
        let is_inp = match ins {
            Instruction::Inp(_) => true,
            _ => false,
        };

        if i != 0 && is_inp {
            splitted.push(current);
            current = vec![];
        }

        current.push(ins.clone());
    }
    splitted.push(current);

    splitted
}

fn find_valid_zs(splitted_ins: &Vec<Vec<Instruction>>) -> Vec<HashSet<i64>> {
    use Variable::*;

    let mut valid_zs = splitted_ins
        .iter()
        .map(|_| HashSet::<i64>::new())
        .collect::<Vec<_>>();

    let mut prev_valid_zs = [0].into_iter().collect::<HashSet<i64>>();

    for (i, ins) in splitted_ins.iter().enumerate().rev() {
        let mut max_z = 100;
        for z in -100.. {
            if z > max_z {
                break;
            }
            for w in 1..=9 {
                let mut alu = ALU::new();
                alu.write(Z, z);
                alu.execute(&ins, [w].into_iter().collect());

                if prev_valid_zs.contains(&alu.read(Z)) {
                    valid_zs[i].insert(z);
                    let next_max_z = z + z / 2;
                    if next_max_z > max_z {
                        max_z = next_max_z;
                    }
                }
            }
        }

        prev_valid_zs = valid_zs[i].clone();
    }

    valid_zs
}

fn solve(input: &Input) -> (u64, u64) {
    use Variable::*;

    // Split instructions per inp instruction
    let ins_per_inp = split_instructions(input);

    // Valid z values for each instruction group, valid in the sense that they can lead to a valid model no
    let mut valid_zs = find_valid_zs(&ins_per_inp);
    // Expected z value is 0 after last instruction
    valid_zs.push([0].into_iter().collect());

    // Initial z value is 0
    let mut last_z = vec![0, 0];

    let mut serial_no = [String::new(), String::new()];
    let try_digits = [(1..=9).rev().collect::<Vec<_>>(), (1..=9).collect()];

    for (i, ins) in ins_per_inp.iter().enumerate() {
        'next: for p in 0..=1 {
            for w in &try_digits[p] {
                let mut alu = ALU::new();
                alu.write(Z, last_z[p]);
                alu.execute(&ins, [*w].into_iter().collect());

                if valid_zs[i + 1].contains(&alu.read(Z)) {
                    serial_no[p].push_str(&format!("{}", w));
                    last_z[p] = alu.read(Z);
                    continue 'next;
                }
            }
        }
    }

    (
        serial_no[0].parse().unwrap_or(0),
        serial_no[1].parse().unwrap_or(0),
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

impl FromStr for Instruction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Instruction::*;

        let mut splits = s.split(' ');

        let ins = splits.next();
        let a = splits
            .next()
            .and_then(|s| s.chars().next())
            .and_then(Variable::from_char);
        let b = splits.next().map(VarOrLit::from_str).transpose()?;

        let ins = match (ins, a, b) {
            (Some("inp"), Some(a), None) => Inp(a),
            (Some("add"), Some(a), Some(b)) => Add(a, b),
            (Some("mul"), Some(a), Some(b)) => Mul(a, b),
            (Some("div"), Some(a), Some(b)) => Div(a, b),
            (Some("mod"), Some(a), Some(b)) => Mod(a, b),
            (Some("eql"), Some(a), Some(b)) => Eql(a, b),
            (ins, a, b) => bail!("Unable to map instruction for: {:?} {:?} {:?}", ins, a, b),
        };

        Ok(ins)
    }
}

impl FromStr for VarOrLit {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(v) = s.chars().next().and_then(Variable::from_char) {
            Ok(VarOrLit::Variable(v))
        } else {
            Ok(VarOrLit::Literal(s.parse::<i64>()?))
        }
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<Instruction>()?))
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
    fn test_alu() -> Result<()> {
        use Variable::*;

        let ins = as_input(
            "
            inp x
            mul x -1",
        )?;

        let mut alu = ALU::new();
        alu.execute(&ins, [10].into_iter().collect());
        assert_eq!(alu.read(X), -10);

        let ins = as_input(
            "
            inp z
            inp x
            mul z 3
            eql z x",
        )?;

        let mut alu = ALU::new();
        alu.execute(&ins, [10, 30].into_iter().collect());
        assert_eq!(alu.read(Z), 1);

        let mut alu = ALU::new();
        alu.execute(&ins, [10, 29].into_iter().collect());
        assert_eq!(alu.read(Z), 0);

        let ins = as_input(
            "
            inp w
            add z w
            mod z 2
            div w 2
            add y w
            mod y 2
            div w 2
            add x w
            mod x 2
            div w 2
            mod w 2",
        )?;

        let mut alu = ALU::new();
        alu.execute(&ins, [0b1_1_1_1].into_iter().collect());
        assert_eq!(alu.read(Z), 0b1);
        assert_eq!(alu.read(Y), 0b1);
        assert_eq!(alu.read(X), 0b1);
        assert_eq!(alu.read(W), 0b1);

        let mut alu = ALU::new();
        alu.execute(&ins, [0b0_0_0_0].into_iter().collect());
        assert_eq!(alu.read(Z), 0b0);
        assert_eq!(alu.read(Y), 0b0);
        assert_eq!(alu.read(X), 0b0);
        assert_eq!(alu.read(W), 0b0);

        let mut alu = ALU::new();
        alu.execute(&ins, [0b0_1_0_1].into_iter().collect());
        assert_eq!(alu.read(Z), 0b1);
        assert_eq!(alu.read(Y), 0b0);
        assert_eq!(alu.read(X), 0b1);
        assert_eq!(alu.read(W), 0b0);

        Ok(())
    }
}
