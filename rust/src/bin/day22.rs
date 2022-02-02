/// CAUTION This solution is not right for the problem, see part2 below
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::ParseIntError;
use std::ops::RangeInclusive;
use std::str::FromStr;

use anyhow::{Context, Result};

use utils::measure;

type Input = Vec<RebootStep>;

#[derive(Debug, Clone)]
struct RebootStep {
    on: bool,
    xr: RangeInclusive<i32>,
    yr: RangeInclusive<i32>,
    zr: RangeInclusive<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Clone)]
struct Partition {
    on: bool,
    xr: RangeInclusive<i32>,
    yr: RangeInclusive<i32>,
    zr: RangeInclusive<i32>,
    sub_partitions: Option<Vec<Option<Partition>>>,
}

impl Partition {
    fn from_bounds(
        xr: RangeInclusive<i32>,
        yr: RangeInclusive<i32>,
        zr: RangeInclusive<i32>,
    ) -> Self {
        Self {
            on: false,
            xr,
            yr,
            zr,
            sub_partitions: None,
        }
    }

    fn subdivide(&self) -> Vec<Option<Partition>> {
        let Partition { xr, yr, zr, on, .. } = self;
        let mut ret = vec![];

        let xm = xr.start() + (xr.end() - xr.start()) / 2;
        let xr1 = *xr.start()..=(xm);
        let xr2 = (xm + 1)..=*xr.end();

        let ym = yr.start() + (yr.end() - yr.start()) / 2;
        let yr1 = *yr.start()..=(ym);
        let yr2 = (ym + 1)..=*yr.end();

        let zm = zr.start() + (zr.end() - zr.start()) / 2;
        let zr1 = *zr.start()..=(zm);
        let zr2 = (zm + 1)..=*zr.end();

        fn valid(p: Partition) -> Option<Partition> {
            if p.xr.start() > p.xr.end() || p.yr.start() > p.yr.end() || p.zr.start() > p.zr.end() {
                None
            } else {
                Some(p)
            }
        }

        for (xr, yr, zr) in [
            (&xr1, &yr1, &zr1),
            (&xr2, &yr1, &zr1),
            (&xr2, &yr1, &zr2),
            (&xr1, &yr1, &zr2),
            (&xr1, &yr2, &zr1),
            (&xr2, &yr2, &zr1),
            (&xr2, &yr2, &zr2),
            (&xr1, &yr2, &zr2),
        ] {
            ret.push(valid(Partition {
                on: *on,
                xr: xr.clone(),
                yr: yr.clone(),
                zr: zr.clone(),
                sub_partitions: None,
            }));
        }

        ret
    }

    fn set_on(
        &mut self,
        on: bool,
        xr: &RangeInclusive<i32>,
        yr: &RangeInclusive<i32>,
        zr: &RangeInclusive<i32>,
    ) -> Option<bool> {
        // Partition fully inside on range
        if xr.contains(self.xr.start())
            && xr.contains(self.xr.end())
            && yr.contains(self.yr.start())
            && yr.contains(self.yr.end())
            && zr.contains(self.zr.start())
            && zr.contains(self.zr.end())
        {
            self.on = on;
            self.sub_partitions = None;
            Some(self.on)
        }
        // Range partially inside of partition
        else if (self.xr.contains(xr.start())
            || self.xr.contains(xr.end())
            || xr.contains(self.xr.start())
            || xr.contains(self.xr.end()))
            && (self.yr.contains(yr.start())
                || self.yr.contains(yr.end())
                || yr.contains(self.yr.start())
                || yr.contains(self.yr.end()))
            && (self.zr.contains(zr.start())
                || self.zr.contains(zr.end())
                || zr.contains(self.zr.start())
                || zr.contains(self.zr.end()))
        {
            if self.sub_partitions.is_none() {
                let sub_partitions = self.subdivide();
                self.sub_partitions = Some(sub_partitions);
            }

            let mut sub_ons = vec![];

            if let Some(sub_partitions) = &mut self.sub_partitions {
                for sub_partition in sub_partitions {
                    if let Some(sub_partition) = sub_partition {
                        let sub_on = sub_partition.set_on(on, xr, yr, zr);
                        sub_ons.push(sub_on);
                    }
                }
            }

            if sub_ons.iter().all(|&so| so == Some(self.on)) {
                // self.sub_partitions = None;
                Some(self.on)
            } else {
                None
            }
        } else {
            Some(self.on)
        }
    }

    fn count_on(&self) -> usize {
        let mut n_on = 0;

        if let Some(sub_partitions) = &self.sub_partitions {
            for sub_partition in sub_partitions {
                if let Some(sub_partition) = sub_partition {
                    n_on += sub_partition.count_on();
                }
            }
        } else {
            if self.on {
                n_on += ((self.xr.end() - self.xr.start() + 1)
                    * (self.yr.end() - self.yr.start() + 1)
                    * (self.zr.end() - self.zr.start() + 1)) as usize;
            }
        }

        n_on
    }
}

fn bounds(
    steps: &Vec<RebootStep>,
) -> (
    RangeInclusive<i32>,
    RangeInclusive<i32>,
    RangeInclusive<i32>,
) {
    let (min_x, max_x, min_y, max_y, min_z, max_z) = steps.iter().fold(
        (0, 0, 0, 0, 0, 0),
        |(min_x, max_x, min_y, max_y, min_z, max_z), step| {
            (
                std::cmp::min(min_x, *step.xr.start()),
                std::cmp::max(max_x, *step.xr.end()),
                std::cmp::min(min_y, *step.yr.start()),
                std::cmp::max(max_y, *step.yr.end()),
                std::cmp::min(min_z, *step.zr.start()),
                std::cmp::max(max_z, *step.zr.end()),
            )
        },
    );
    (min_x..=max_x, min_y..=max_y, min_z..=max_z)
}

fn part1(input: &Input) -> usize {
    let bounds = -50..=50;

    let mut root = Partition::from_bounds(bounds.clone(), bounds.clone(), bounds.clone());

    for RebootStep { on, xr, yr, zr } in input.iter().cloned() {
        if ![
            xr.start(),
            xr.end(),
            yr.start(),
            yr.end(),
            zr.start(),
            zr.end(),
        ]
        .iter()
        .all(|&&n| (-50..=50).contains(&n))
        {
            continue;
        }

        root.set_on(on, &xr, &yr, &zr);
    }

    root.count_on()
}

/*
Very inefficient solution, I solved this by bruteforcing..
For some reason I went down the line of a https://en.wikipedia.org/wiki/Octree partitioning approach but
it didn't pan out memory usage wise. I then adapted it to conserve memory such that it operates on slices on the z axis.
For that a https://en.wikipedia.org/wiki/Quadtree partitioning would have been enough but I doubth it would have made
a significant enough difference since the whole approach was wrong.

Saved output:
z=-12344 from -97454..=95416 tot_on_cubes=535852590421475
z=35518 from -97454..=95416 tot_on_cubes=959313333288009
z=95416 from -97454..=95416 tot_on_cubes=1263946820845866
Part2: 1263946820845866
It took: 156156421.618617ms

Thats about 43+ hours, poor computer who had to go through that ;)
*/

fn part2(input: &Input) -> usize {
    let (xr, yr, zr) = bounds(input);
    let mut tot_on_cubes = 0;

    for z in zr.clone() {
        let curr_zr = z..=z;
        println!("z={} from {:?} tot_on_cubes={}", z, zr, tot_on_cubes);

        let mut root = Partition::from_bounds(xr.clone(), yr.clone(), curr_zr.clone());

        for RebootStep { on, xr, yr, zr } in input.iter().cloned() {
            if !zr.contains(&z) {
                continue;
            }
            root.set_on(on, &xr, &yr, &curr_zr);
        }

        tot_on_cubes += root.count_on();
    }

    tot_on_cubes
}

fn main() -> Result<()> {
    measure(|| {
        let input = input()?;
        println!("Part1: {}", part1(&input));
        println!("Part2: {}", part2(&input));
        Ok(())
    })
}

impl FromStr for RebootStep {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(' ');
        let on = split.next() == Some("on");

        fn parse_range(s: &str) -> Result<RangeInclusive<i32>, ParseIntError> {
            let mut split = s.split("..");
            let start = split.next().unwrap().parse::<i32>()?;
            let end = split.next().unwrap().parse::<i32>()?;
            Ok(start..=end)
        }

        let s = split.next().unwrap();
        let mut split = s.split(',');
        let xr = parse_range(split.next().unwrap().split('=').skip(1).next().unwrap())?;
        let yr = parse_range(split.next().unwrap().split('=').skip(1).next().unwrap())?;
        let zr = parse_range(split.next().unwrap().split('=').skip(1).next().unwrap())?;

        Ok(RebootStep { on, xr, yr, zr })
    }
}

fn read_input<R: Read>(reader: BufReader<R>) -> Result<Input> {
    reader
        .lines()
        .map(|line| Ok(line?.parse::<RebootStep>()?))
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
        on x=-20..26,y=-36..17,z=-47..7
        on x=-20..33,y=-21..23,z=-26..28
        on x=-22..28,y=-29..23,z=-38..16
        on x=-46..7,y=-6..46,z=-50..-1
        on x=-49..1,y=-3..46,z=-24..28
        on x=2..47,y=-22..22,z=-23..27
        on x=-27..23,y=-28..26,z=-21..29
        on x=-39..5,y=-6..47,z=-3..44
        on x=-30..21,y=-8..43,z=-13..34
        on x=-22..26,y=-27..20,z=-29..19
        off x=-48..-32,y=26..41,z=-47..-37
        on x=-12..35,y=6..50,z=-50..-2
        off x=-48..-32,y=-32..-16,z=-15..-5
        on x=-18..26,y=-33..15,z=-7..46
        off x=-40..-22,y=-38..-28,z=23..41
        on x=-16..35,y=-41..10,z=-47..6
        off x=-32..-23,y=11..30,z=-14..3
        on x=-49..-5,y=-3..45,z=-29..18
        off x=18..30,y=-20..-8,z=-3..13
        on x=-41..9,y=-7..43,z=-33..15
        on x=-54112..-39298,y=-85059..-49293,z=-27449..7877
        on x=967..23432,y=45373..81175,z=27513..53682";

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
        assert_eq!(part1(&as_input(INPUT)?), 590784);
        Ok(())
    }
}
