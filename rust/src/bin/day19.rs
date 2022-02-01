// use std::cmp;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<Scanner>;

#[derive(Debug)]
struct Scanner {
    beacons: Vec<Vec3>,
}

impl Scanner {
    #[must_use]
    fn rotate(&self, x: i32, y: i32, z: i32) -> Self {
        let beacons = self.beacons.iter().map(|b| b.rotate(x, y, z)).collect();
        Self { beacons }
    }

    fn all_diffs(&self) -> Vec<Vec<Vec3>> {
        let mut all_diffs = vec![]; // TODO Capacity

        for b_idx in 0..self.beacons.len() {
            let mut diffs = Vec::new(); // TODO Capacity
            let curr = self.beacons[b_idx];

            for other in self.beacons.iter() {
                let diff = curr.diff(other);
                diffs.push(diff);
            }
            all_diffs.push(diffs);
        }

        all_diffs
    }
}

enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3 {
    #[must_use]
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    fn rot90(&self, axis: Axis) -> Self {
        let &Self { x, y, z } = self;
        match axis {
            Axis::X => Self {
                y: z * 1,
                z: y * -1,
                x,
            },
            Axis::Y => Self {
                z: x * 1,
                x: z * -1,
                y,
            },
            Axis::Z => Self {
                x: y * 1,
                y: x * -1,
                z,
            },
        }
    }

    #[must_use]
    fn rotate(&self, x: i32, y: i32, z: i32) -> Self {
        let mut b = self.clone();

        for _ in 0..x {
            b = b.rot90(Axis::X);
        }
        for _ in 0..y {
            b = b.rot90(Axis::Y);
        }
        for _ in 0..z {
            b = b.rot90(Axis::Z);
        }

        b
    }

    #[must_use]
    fn translate(&self, other: &Vec3) -> Self {
        let &Self { x, y, z } = self;
        Self {
            x: x + other.x,
            y: y + other.y,
            z: z + other.z,
        }
    }
    #[must_use]
    fn diff(&self, other: &Vec3) -> Self {
        let &Self { x, y, z } = self;
        Self {
            x: x - other.x,
            y: y - other.y,
            z: z - other.z,
        }
    }

    fn manh_dist(&self, other: &Vec3) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}

/// The 24 possble 90 degree per axis rotations
fn rotations() -> Vec<(i32, i32, i32)> {
    let mut rots = BTreeSet::new();
    for x in 0..=3 {
        for y in 0..=3 {
            for z in 0..=3 {
                rots.insert((x, y, z));
            }
        }
    }

    let pos = Vec3::new(1, 2, 3);
    let mut rot_results = HashMap::<Vec3, BTreeSet<(i32, i32, i32)>>::new();

    for (x, y, z) in rots {
        let mut p = pos.clone();

        for _ in 0..x {
            p = p.rot90(Axis::X);
        }
        for _ in 0..y {
            p = p.rot90(Axis::Y);
        }
        for _ in 0..z {
            p = p.rot90(Axis::Z);
        }

        rot_results.entry(p).or_default().insert((x, y, z));
    }

    let mut rots = rot_results
        .values()
        .filter_map(|rots| rots.iter().next())
        .cloned()
        .collect::<Vec<_>>();
    rots.sort();
    rots
}

fn solve(scanners: &Input) -> (usize, i32) {
    let rots = rotations();

    let scanner_diffs = scanners
        .iter()
        .map(|s| {
            rots.iter()
                .map(|&(rx, ry, rz)| s.clone().rotate(rx, ry, rz).all_diffs())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut common_by_idx = HashMap::<usize, HashMap<usize, (usize, Vec<(usize, usize)>)>>::new();

    for s_idx in 0..scanners.len() {
        'next: for other_s_idx in (0..scanners.len()).filter(|&i| i != s_idx) {
            let s_diffs = &scanner_diffs[s_idx][0];
            let other_s_diffs_rots = &scanner_diffs[other_s_idx];

            for diffs in s_diffs {
                for rot_idx in 0..other_s_diffs_rots.len() {
                    let other_s_diffs = &other_s_diffs_rots[rot_idx];
                    for other_diff in other_s_diffs {
                        let mut common = vec![];

                        for (i, diff) in diffs.iter().enumerate() {
                            if let Some((i2, _)) = other_diff
                                .iter()
                                .enumerate()
                                .find(|&(_, o_diff)| diff == o_diff)
                            {
                                common.push((i, i2));
                            }
                        }

                        if common.len() >= 12 {
                            common_by_idx
                                .entry(s_idx)
                                .or_default()
                                .insert(other_s_idx, (rot_idx, common));

                            continue 'next;
                        }
                    }
                }
            }
        }
    }

    let mut positions = HashMap::new();
    positions.insert(0, (Vec3::new(0, 0, 0), vec![]));

    let mut queue = VecDeque::new();
    for &i in common_by_idx[&0].keys() {
        queue.push_back(((0_usize, i), vec![]));
    }

    while let Some(((s_idx, o_idx), mut rotations)) = queue.pop_front() {
        if positions.contains_key(&o_idx) {
            continue;
        }

        let (rot, s_and_o) = &common_by_idx[&s_idx][&o_idx];
        let &(rx, ry, rz) = &rots[*rot];

        let (s0, s1) = s_and_o[0];

        let mut b0 = scanners[s_idx].beacons[s0];
        let mut b1 = scanners[o_idx].beacons[s1].rotate(rx, ry, rz);

        for &(rx, ry, rz) in rotations.iter().rev() {
            b0 = b0.rotate(rx, ry, rz);
            b1 = b1.rotate(rx, ry, rz);
        }

        rotations.push((rx, ry, rz));

        let pos = positions[&s_idx].0.translate(&b0.diff(&b1));
        positions.insert(o_idx, (pos, rotations.clone()));

        for &i in common_by_idx[&o_idx].keys() {
            queue.push_back(((o_idx, i), rotations.clone()));
        }
    }

    let mut beacons = BTreeSet::new();
    for (s_idx, scanner) in scanners.iter().enumerate() {
        let (s_pos, rots) = &positions[&s_idx];
        for &beacon in &scanner.beacons {
            let pos = rots
                .iter()
                .rev()
                .fold(beacon, |b, &(rx, ry, rz)| b.rotate(rx, ry, rz));

            let pos = s_pos.translate(&pos);
            beacons.insert(pos);
        }
    }

    let beacons = beacons.into_iter().collect::<Vec<_>>();

    let mut max_manh_dist = 0;

    for b1 in 0..scanners.len() {
        for b2 in 0..scanners.len() {
            let p1 = positions[&b1].0;
            let p2 = positions[&b2].0;
            let dist = p1.manh_dist(&p2);
            if dist > max_manh_dist {
                max_manh_dist = dist;
            }
        }
    }

    (beacons.len(), max_manh_dist)
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

impl FromStr for Vec3 {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(',');
        Ok(Vec3 {
            x: splits.next().unwrap().parse::<i32>()?,
            y: splits.next().unwrap().parse::<i32>()?,
            z: splits.next().unwrap().parse::<i32>()?,
        })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    let mut scanners = vec![];
    let mut lines = reader.lines();

    loop {
        let beacons = lines
            .by_ref()
            .skip(1)
            .map(|l| l.unwrap())
            .take_while(|l| !l.is_empty())
            .map(|l| l.parse::<Vec3>().unwrap())
            .collect::<Vec<_>>();

        if beacons.is_empty() {
            break;
        }
        scanners.push(Scanner { beacons });
    }

    Ok(scanners)
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
        --- scanner 0 ---
        404,-588,-901
        528,-643,409
        -838,591,734
        390,-675,-793
        -537,-823,-458
        -485,-357,347
        -345,-311,381
        -661,-816,-575
        -876,649,763
        -618,-824,-621
        553,345,-567
        474,580,667
        -447,-329,318
        -584,868,-557
        544,-627,-890
        564,392,-477
        455,729,728
        -892,524,684
        -689,845,-530
        423,-701,434
        7,-33,-71
        630,319,-379
        443,580,662
        -789,900,-551
        459,-707,401

        --- scanner 1 ---
        686,422,578
        605,423,415
        515,917,-361
        -336,658,858
        95,138,22
        -476,619,847
        -340,-569,-846
        567,-361,727
        -460,603,-452
        669,-402,600
        729,430,532
        -500,-761,534
        -322,571,750
        -466,-666,-811
        -429,-592,574
        -355,545,-477
        703,-491,-529
        -328,-685,520
        413,935,-424
        -391,539,-444
        586,-435,557
        -364,-763,-893
        807,-499,-711
        755,-354,-619
        553,889,-390

        --- scanner 2 ---
        649,640,665
        682,-795,504
        -784,533,-524
        -644,584,-595
        -588,-843,648
        -30,6,44
        -674,560,763
        500,723,-460
        609,671,-379
        -555,-800,653
        -675,-892,-343
        697,-426,-610
        578,704,681
        493,664,-388
        -671,-858,530
        -667,343,800
        571,-461,-707
        -138,-166,112
        -889,563,-600
        646,-828,498
        640,759,510
        -630,509,768
        -681,-892,-333
        673,-379,-804
        -742,-814,-386
        577,-820,562

        --- scanner 3 ---
        -589,542,597
        605,-692,669
        -500,565,-823
        -660,373,557
        -458,-679,-417
        -488,449,543
        -626,468,-788
        338,-750,-386
        528,-832,-391
        562,-778,733
        -938,-730,414
        543,643,-506
        -524,371,-870
        407,773,750
        -104,29,83
        378,-903,-323
        -778,-728,485
        426,699,580
        -438,-605,-362
        -469,-447,-387
        509,732,623
        647,635,-688
        -868,-804,481
        614,-800,639
        595,780,-596

        --- scanner 4 ---
        727,592,562
        -293,-554,779
        441,611,-461
        -714,465,-776
        -743,427,-804
        -660,-479,-426
        832,-632,460
        927,-485,-438
        408,393,-506
        466,436,-512
        110,16,151
        -258,-428,682
        -393,719,612
        -211,-452,876
        808,-476,-593
        -575,615,604
        -485,667,467
        -680,325,-822
        -627,-443,-432
        872,-547,-609
        833,512,582
        807,604,487
        839,-516,451
        891,-625,532
        -652,-548,-490
        30,-46,-14";

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
        assert_eq!(solve(&as_input(INPUT)?).0, 79);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(solve(&as_input(INPUT)?).1, 3621);
        Ok(())
    }
}
