use std::cell::RefCell;
use std::cmp;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<Number>;

#[derive(Debug, Clone, Eq, PartialEq)]
enum Number {
    Literal(i32),
    Pair(RefCell<Box<Number>>, RefCell<Box<Number>>),
}

impl Number {
    fn is_literal(&self) -> bool {
        match self {
            Number::Literal(_) => true,
            _ => false,
        }
    }

    fn literal(&self) -> i32 {
        match self {
            &Number::Literal(literal) => literal,
            _ => panic!("Not a literal"),
        }
    }

    fn reduce(&self) -> Number {
        fn explode(n: &Number, depth: i32) -> Option<(bool, Option<i32>, Option<i32>)> {
            match n {
                Number::Literal(_) => None,
                Number::Pair(p1, p2) => {
                    if depth >= 4 {
                        return Some((
                            true,
                            Some(p1.borrow().literal()),
                            Some(p2.borrow().literal()),
                        ));
                    }

                    let fallout = explode(&p1.borrow(), depth + 1);
                    if let Some((exploded, fa_left, fa_right)) = fallout {
                        if exploded {
                            p1.replace(Box::new(Number::Literal(0)));
                        }

                        if let Some(fa_right) = fa_right {
                            if p2.borrow().is_literal() {
                                let lit = p2.borrow().literal();
                                p2.replace(Box::new(Number::Literal(lit + fa_right)));
                            } else {
                                if !add_fallout_right(p2, fa_right) {
                                    return Some((false, fa_left, Some(fa_right)));
                                }
                            }
                        }
                        return Some((false, fa_left, None));
                    }

                    let fallout = explode(&p2.borrow(), depth + 1);
                    if let Some((exploded, fa_left, fa_right)) = fallout {
                        if exploded {
                            p2.replace(Box::new(Number::Literal(0)));
                        }

                        if let Some(fa_left) = fa_left {
                            if p1.borrow().is_literal() {
                                let lit = p1.borrow().literal();
                                p1.replace(Box::new(Number::Literal(lit + fa_left)));
                            } else {
                                if !add_fallout_left(p1, fa_left) {
                                    return Some((false, Some(fa_left), fa_right));
                                }
                            }
                        }
                        return Some((false, None, fa_right));
                    }
                    None
                }
            }
        }

        fn add_fallout_right(n: &RefCell<Box<Number>>, add: i32) -> bool {
            if n.borrow().is_literal() {
                let lit = n.borrow().literal();
                n.replace(Box::new(Number::Literal(lit + add)));
                true
            } else {
                match &**n.borrow() {
                    Number::Pair(p1, p2) => {
                        if add_fallout_right(p1, add) {
                            return true;
                        }
                        add_fallout_right(p2, add)
                    }
                    _ => unreachable!(),
                }
            }
        }

        fn add_fallout_left(n: &RefCell<Box<Number>>, add: i32) -> bool {
            if n.borrow().is_literal() {
                let lit = n.borrow().literal();
                n.replace(Box::new(Number::Literal(lit + add)));
                true
            } else {
                match &**n.borrow() {
                    Number::Pair(p1, p2) => {
                        if add_fallout_left(p2, add) {
                            return true;
                        }
                        add_fallout_left(p1, add)
                    }
                    _ => unreachable!(),
                }
            }
        }

        fn split(n: &Number) -> bool {
            if n.is_literal() && n.literal() >= 10 {
                true
            } else {
                match n {
                    Number::Pair(p1, p2) => {
                        if split_cell(p1) {
                            return true;
                        }
                        split_cell(p2)
                    }
                    _ => unreachable!(),
                }
            }
        }

        fn split_cell(n: &RefCell<Box<Number>>) -> bool {
            if n.borrow().is_literal() {
                let lit = n.borrow().literal();
                if lit >= 10 {
                    let l1 = lit / 2;
                    n.replace(Box::new(Number::Pair(
                        RefCell::new(Box::new(Number::Literal(l1))),
                        RefCell::new(Box::new(Number::Literal(lit - l1))),
                    )));
                    true
                } else {
                    false
                }
            } else {
                match &**n.borrow() {
                    Number::Pair(p1, p2) => {
                        if split_cell(p1) {
                            return true;
                        }
                        split_cell(p2)
                    }
                    _ => unreachable!(),
                }
            }
        }

        let n = self.clone();

        loop {
            if let Some(_) = explode(&n, 0) {
                continue;
            }
            if split(&n) {
                continue;
            }
            break;
        }
        n
    }

    fn add(&self, other: &Number) -> Number {
        Number::Pair(
            RefCell::new(Box::new(self.clone())),
            RefCell::new(Box::new(other.clone())),
        )
    }

    fn add_all(numbers: &Vec<Number>) -> Number {
        let mut added = numbers[0].clone();

        for n in numbers.iter().skip(1) {
            added = added.add(n);
            added = added.reduce();
        }

        added
    }

    fn magnitude(&self) -> i32 {
        match self {
            &Number::Literal(literal) => literal,
            Number::Pair(p1, p2) => p1.borrow().magnitude() * 3 + p2.borrow().magnitude() * 2,
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Literal(literal) => write!(f, "{}", literal)?,
            Number::Pair(p1, p2) => {
                write!(f, "[")?;
                fmt::Display::fmt(&p1.borrow(), f)?;
                write!(f, ",")?;
                fmt::Display::fmt(&p2.borrow(), f)?;
                write!(f, "]")?;
            }
        }

        Ok(())
    }
}

struct Combinations {
    values: Vec<usize>,
    indexes: Vec<usize>,
    n: usize,
    first: bool,
}

impl Combinations {
    fn new(values: &Vec<usize>, n: usize) -> Combinations {
        Combinations {
            values: values.clone(),
            indexes: (0..n).collect(),
            n,
            first: true,
        }
    }
}

impl Iterator for Combinations {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            return Some(
                self.indexes
                    .iter()
                    .map(|i| self.values[*i])
                    .collect::<Self::Item>(),
            );
        }

        let n_values = self.values.len();
        let n = self.n;
        {
            let v = &mut self.indexes;
            let l = n - 1;
            for i in 0..n {
                if v[l - i] == n_values - 1 - i {
                    if i == n - 1 {
                        return None;
                    }
                    v[l - i] = v[l - i - 1] + 2;
                    if i > 0 {
                        for j in (0..=(i - 1)).rev() {
                            v[l - j] = v[l - (j + 1)] + 1;
                        }
                    }
                } else {
                    v[l - i] += 1;
                    break;
                }
            }
        }

        let mut next = Vec::with_capacity(n_values);
        for i in 0..n {
            next.push(self.values[self.indexes[i]]);
        }
        Some(next)
    }
}

fn part1(input: &Input) -> i32 {
    Number::add_all(input).magnitude()
}

fn part2(input: &Input) -> i32 {
    let mut max_magnitude = 0;
    let indexes = (0..input.len()).collect::<Vec<_>>();
    for comb in Combinations::new(&indexes, 2) {
        for numbers in [&comb, &comb.iter().rev().cloned().collect::<Vec<_>>()]
            .iter()
            .map(|idxs| idxs.iter().map(|&i| input[i].clone()).collect::<Vec<_>>())
        {
            let magnitude = Number::add_all(&numbers).magnitude();
            max_magnitude = cmp::max(max_magnitude, magnitude);
        }
    }
    max_magnitude
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2: {}", part2(&input));
        Ok(())
    })
}

impl FromStr for Number {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(s: &str) -> Result<(Number, usize), ParseIntError> {
            if s.chars().next() == Some('[') {
                let (p1, consumed1) = parse(&s[1..])?;
                let (p2, consumed2) = parse(&s[(1 + consumed1 + 1)..])?;

                Ok((
                    Number::Pair(RefCell::new(Box::new(p1)), RefCell::new(Box::new(p2))),
                    consumed1 + consumed2 + 3,
                ))
            } else {
                let s = s
                    .chars()
                    .take_while(|&c| c != '[' && c != ']' && c != ',')
                    .collect::<String>();
                Ok((Number::Literal(s.parse::<i32>()?), s.len()))
            }
        }

        let (n, _) = parse(s)?;
        Ok(n)
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<Number>()?))
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
        [[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
        [[[5,[2,8]],4],[5,[[9,9],0]]]
        [6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
        [[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
        [[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
        [[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
        [[[[5,4],[7,7]],8],[[8,3],8]]
        [[9,3],[[9,9],[6,[4,9]]]]
        [[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
        [[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]";

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
    fn test_reduce() -> Result<()> {
        let n = "[[[[[9,8],1],2],3],4]".parse::<Number>().unwrap().reduce();
        assert_eq!(format!("{}", n), "[[[[0,9],2],3],4]".to_owned());

        let n = "[7,[6,[5,[4,[3,2]]]]]".parse::<Number>().unwrap().reduce();
        assert_eq!(format!("{}", n), "[7,[6,[5,[7,0]]]]".to_owned());

        let n = "[[6,[5,[4,[3,2]]]],1]".parse::<Number>().unwrap().reduce();
        assert_eq!(format!("{}", n), "[[6,[5,[7,0]]],3]".to_owned());

        let n = "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]"
            .parse::<Number>()
            .unwrap()
            .reduce();
        assert_eq!(format!("{}", n), "[[3,[2,[8,0]]],[9,[5,[7,0]]]]".to_owned());

        let n = "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]"
            .parse::<Number>()
            .unwrap()
            .reduce();
        assert_eq!(
            format!("{}", n),
            "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_adding() -> Result<()> {
        let numbers = as_input(
            "
        [1,1]
        [2,2]
        [3,3]
        [4,4]",
        )?;
        let n = Number::add_all(&numbers);
        assert_eq!(format!("{}", n), "[[[[1,1],[2,2]],[3,3]],[4,4]]".to_owned());

        let numbers = as_input(
            "
        [1,1]
        [2,2]
        [3,3]
        [4,4]
        [5,5]",
        )?;
        let n = Number::add_all(&numbers);
        assert_eq!(format!("{}", n), "[[[[3,0],[5,3]],[4,4]],[5,5]]".to_owned());

        let numbers = as_input(
            "
        [1,1]
        [2,2]
        [3,3]
        [4,4]
        [5,5]
        [6,6]",
        )?;
        let n = Number::add_all(&numbers);
        assert_eq!(format!("{}", n), "[[[[5,0],[7,4]],[5,5]],[6,6]]".to_owned());

        let numbers = as_input(
            "
        [[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
        [7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
        [[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
        [[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
        [7,[5,[[3,8],[1,4]]]]
        [[2,[2,2]],[8,[8,1]]]
        [2,9]
        [1,[[[9,3],9],[[9,0],[0,7]]]]
        [[[5,[7,4]],7],1]
        [[[[4,2],2],6],[8,7]]",
        )?;
        let n = Number::add_all(&numbers);
        assert_eq!(
            format!("{}", n),
            "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_magnitude() -> Result<()> {
        assert_eq!("[9,1]".parse::<Number>().unwrap().magnitude(), 29);
        assert_eq!("[1,9]".parse::<Number>().unwrap().magnitude(), 21);
        assert_eq!("[[9,1],[1,9]]".parse::<Number>().unwrap().magnitude(), 129);
        assert_eq!(
            "[[1,2],[[3,4],5]]".parse::<Number>().unwrap().magnitude(),
            143
        );
        assert_eq!(
            "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]"
                .parse::<Number>()
                .unwrap()
                .magnitude(),
            1384
        );
        assert_eq!(
            "[[[[1,1],[2,2]],[3,3]],[4,4]]"
                .parse::<Number>()
                .unwrap()
                .magnitude(),
            445
        );
        assert_eq!(
            "[[[[3,0],[5,3]],[4,4]],[5,5]]"
                .parse::<Number>()
                .unwrap()
                .magnitude(),
            791
        );
        assert_eq!(
            "[[[[5,0],[7,4]],[5,5]],[6,6]]"
                .parse::<Number>()
                .unwrap()
                .magnitude(),
            1137
        );
        assert_eq!(
            "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]"
                .parse::<Number>()
                .unwrap()
                .magnitude(),
            3488
        );

        Ok(())
    }

    #[test]
    fn test_part1() -> Result<()> {
        assert_eq!(part1(&as_input(INPUT)?), 4140);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 3993);
        Ok(())
    }
}
