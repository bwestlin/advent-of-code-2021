use std::cmp::{self, Ordering};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::env;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};

use utils::measure;

type Input = BurrowState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AmphipodType {
    Amber,
    Bronze,
    Copper,
    Desert,
}

impl AmphipodType {
    fn ordinal(&self) -> usize {
        use AmphipodType::*;
        match self {
            Amber => 0,
            Bronze => 1,
            Copper => 2,
            Desert => 3,
        }
    }

    fn step_energy(&self) -> usize {
        use AmphipodType::*;
        match self {
            Amber => 1,
            Bronze => 10,
            Copper => 100,
            Desert => 1000,
        }
    }

    #[cfg(feature = "print")]
    fn as_char(&self) -> char {
        use AmphipodType::*;
        match self {
            Amber => 'A',
            Bronze => 'B',
            Copper => 'C',
            Desert => 'D',
        }
    }

    fn from_char(c: char) -> Self {
        use AmphipodType::*;
        match c {
            'A' => Amber,
            'B' => Bronze,
            'C' => Copper,
            'D' => Desert,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum AmphipodPosition {
    Room { typ: AmphipodType, idx: usize },
    Hallway { idx: usize },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct AmphipodState {
    typ: AmphipodType,
    position: AmphipodPosition,
    has_been_in_hallway: bool,
}

impl AmphipodState {
    fn is_in_desired_room(&self) -> bool {
        match self.position {
            AmphipodPosition::Room { typ, .. } if self.typ == typ => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct BurrowState {
    amphipods_state: Vec<AmphipodState>,
    energy: usize,
    room_size: usize,
}

const HALLWAY_POSITIONS: [usize; 7] = [0, 1, 3, 5, 7, 9, 10];
const ROOM_POSITIONS: [usize; 4] = [2, 4, 6, 8];
const ROOM_TYPES: [AmphipodType; 4] = [
    AmphipodType::Amber,
    AmphipodType::Bronze,
    AmphipodType::Copper,
    AmphipodType::Desert,
];

impl BurrowState {
    fn moves(&self) -> Vec<BurrowState> {
        let mut moves = vec![];

        for (idx, ams) in self.amphipods_state.iter().enumerate() {
            match ams.position {
                AmphipodPosition::Room {
                    typ: r_typ,
                    idx: r_idx,
                } => {
                    if ams.has_been_in_hallway {
                        continue;
                    }
                    for h_idx in self.possible_hallway_positions(r_typ, r_idx) {
                        let mut next = self.clone();

                        let room_pos = ROOM_POSITIONS[r_typ.ordinal()];
                        let energy = ((r_idx + 1)
                            + (h_idx as i32 - room_pos as i32).abs() as usize)
                            * ams.typ.step_energy();

                        next.amphipods_state[idx] = AmphipodState {
                            typ: ams.typ,
                            position: AmphipodPosition::Hallway { idx: h_idx },
                            has_been_in_hallway: true,
                        };
                        next.energy += energy;

                        moves.push(next);
                    }
                }
                AmphipodPosition::Hallway { idx: h_idx } => {
                    if let Some(r_idx) = self.possible_room_position(h_idx, ams.typ) {
                        let mut next = self.clone();

                        let room_pos = ROOM_POSITIONS[ams.typ.ordinal()];
                        let energy = ((r_idx + 1)
                            + (h_idx as i32 - room_pos as i32).abs() as usize)
                            * ams.typ.step_energy();

                        next.amphipods_state[idx] = AmphipodState {
                            typ: ams.typ,
                            has_been_in_hallway: true,
                            position: AmphipodPosition::Room {
                                typ: ams.typ,
                                idx: r_idx,
                            },
                        };
                        next.energy += energy;

                        moves.push(next);
                    }
                }
            }
        }
        moves
    }

    fn possible_room_position(&self, h_idx: usize, r_typ: AmphipodType) -> Option<usize> {
        // Check that all others in same room type is correct type
        if self.amphipods_state.iter().any(
            |AmphipodState {
                 typ: a_typ,
                 position,
                 ..
             }| match position {
                AmphipodPosition::Room { typ, .. } if r_typ == *typ => *a_typ != r_typ,
                _ => false,
            },
        ) {
            return None;
        }

        // Check if room is possible
        'outer: for r_idx in (0..self.room_size).rev() {
            // Check if room position is already taken
            // TODO Should be able to be replaced with e.g self.at_room(r_typ, r_idx).is_some()
            if self
                .amphipods_state
                .iter()
                .any(|AmphipodState { position, .. }| match position {
                    AmphipodPosition::Room { typ, idx } if r_typ == *typ && r_idx == *idx => true,
                    _ => false,
                })
            {
                continue;
            }

            let room_pos = ROOM_POSITIONS[r_typ.ordinal()];
            let taken_hallway_positions = self
                .amphipods_state
                .iter()
                .filter_map(|AmphipodState { position, .. }| match position {
                    AmphipodPosition::Hallway { idx } if *idx != h_idx => Some(*idx),
                    _ => None,
                })
                .collect::<HashSet<_>>();

            let start = cmp::min(room_pos, h_idx);
            let stop = cmp::max(room_pos, h_idx);

            for pos in start..=stop {
                if taken_hallway_positions.contains(&pos) {
                    continue 'outer;
                }
            }
            return Some(r_idx);
        }

        None
    }

    fn possible_hallway_positions(&self, r_typ: AmphipodType, r_idx: usize) -> Vec<usize> {
        // Check if entrance is blocked
        // TODO Should be able to be replaced with e.g self.at_room(r_typ, r_idx).is_some()
        if r_idx > 0 {
            for rc_idx in (0..r_idx).rev() {
                if self.amphipods_state.iter().any(
                    |AmphipodState { position, .. }| match position {
                        AmphipodPosition::Room { typ, idx } if r_typ == *typ && *idx == rc_idx => {
                            true
                        }
                        _ => false,
                    },
                ) {
                    return Vec::with_capacity(0);
                }
            }
        }

        let room_pos = ROOM_POSITIONS[r_typ.ordinal()];
        let taken_hallway_positions = self
            .amphipods_state
            .iter()
            .filter_map(|AmphipodState { position, .. }| match position {
                AmphipodPosition::Hallway { idx } => Some(*idx),
                _ => None,
            })
            .collect::<HashSet<_>>();

        let mut ret = vec![];
        for range in [
            (0..=room_pos).rev().collect::<Vec<_>>(),
            (room_pos..=10).collect(),
        ] {
            for pos in range {
                if !HALLWAY_POSITIONS.contains(&pos) {
                    continue;
                }
                if taken_hallway_positions.contains(&pos) {
                    break;
                }

                ret.push(pos);
            }
        }

        ret
    }

    fn reached_goal(&self) -> bool {
        self.amphipods_state
            .iter()
            .all(|ams| ams.is_in_desired_room())
    }

    fn organize_least_energy(&self) -> Option<Self> {
        #[derive(Clone, Debug)]
        struct Queued {
            burrow: BurrowState,
        }
        impl Ord for Queued {
            fn cmp(&self, other: &Queued) -> Ordering {
                other.burrow.energy.cmp(&self.burrow.energy)
            }
        }
        impl PartialOrd for Queued {
            fn partial_cmp(&self, other: &Queued) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl PartialEq for Queued {
            fn eq(&self, other: &Queued) -> bool {
                self.burrow.energy == other.burrow.energy
            }
        }
        impl Eq for Queued {}

        let mut queued_hash = HashMap::new();

        let mut heap = BinaryHeap::new();
        heap.push(Queued {
            burrow: self.clone(),
        });
        queued_hash.insert(self.amphipods_state_hash(), 0);

        while let Some(Queued { burrow }) = heap.pop() {
            if burrow.reached_goal() {
                return Some(burrow);
            }

            let hash = burrow.amphipods_state_hash();
            if let Some(queued_energy) = queued_hash.get(&hash) {
                if *queued_energy < burrow.energy {
                    continue;
                }
            }

            for next in burrow.moves() {
                let hash = next.amphipods_state_hash();
                if let Some(queued_energy) = queued_hash.get(&hash) {
                    if *queued_energy <= next.energy {
                        continue;
                    }
                }
                queued_hash.insert(hash, next.energy);
                heap.push(Queued { burrow: next });
            }
        }

        None
    }

    fn amphipods_state_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.amphipods_state.hash(&mut hasher);
        hasher.finish()
    }

    fn unfold(&self) -> Self {
        let mut unfolded = self.clone();

        for a_idx in 0..unfolded.amphipods_state.len() {
            match unfolded.amphipods_state[a_idx].position {
                AmphipodPosition::Room { typ, idx } if idx == 1 => {
                    unfolded.amphipods_state[a_idx].position =
                        AmphipodPosition::Room { typ, idx: idx + 2 };
                }
                _ => {}
            }
        }

        let add = [['D', 'C', 'B', 'A'], ['D', 'B', 'A', 'C']];

        for (i, add) in add.iter().enumerate() {
            for (j, c) in add.iter().enumerate() {
                let ams = AmphipodState {
                    typ: AmphipodType::from_char(*c),
                    position: AmphipodPosition::Room {
                        typ: ROOM_TYPES[j],
                        idx: 1 + i,
                    },
                    has_been_in_hallway: false,
                };
                unfolded.amphipods_state.push(ams);
            }
        }

        unfolded.room_size += 2;
        unfolded
    }

    #[cfg(feature = "print")]
    fn print(&self) {
        use AmphipodType::*;

        println!("#############");

        // Hallway
        print!("#");
        for idx in 0..11 {
            let amphipod = self.amphipods_state.iter().find(|ams| match ams {
                AmphipodState {
                    position: AmphipodPosition::Hallway { idx: a_idx },
                    ..
                } if idx == *a_idx => true,
                _ => false,
            });
            let c = amphipod.map(|ams| ams.typ.as_char()).unwrap_or('.');

            print!("{}", c);
        }
        println!("#");

        // Rooms
        let in_room = |at: AmphipodType, idx: usize| {
            let amphipod = self.amphipods_state.iter().find(|ams| match ams {
                AmphipodState {
                    position: AmphipodPosition::Room { typ, idx: a_idx },
                    ..
                } if idx == *a_idx && at == *typ => true,
                _ => false,
            });
            amphipod.map(|ams| ams.typ.as_char()).unwrap_or('.')
        };

        println!(
            "###{}#{}#{}#{}###",
            in_room(Amber, 0),
            in_room(Bronze, 0),
            in_room(Copper, 0),
            in_room(Desert, 0)
        );
        for r_idx in 1..self.room_size {
            println!(
                "  #{}#{}#{}#{}#",
                in_room(Amber, r_idx),
                in_room(Bronze, r_idx),
                in_room(Copper, r_idx),
                in_room(Desert, r_idx)
            );
        }
        println!("  #########");
    }
}

fn part1(input: &Input) -> usize {
    #[cfg(feature = "print")]
    input.print();
    input.organize_least_energy().map(|b| b.energy).unwrap_or(0)
}

fn part2(input: &Input) -> usize {
    let burrow = input.unfold();
    #[cfg(feature = "print")]
    burrow.print();
    burrow
        .organize_least_energy()
        .map(|b| b.energy)
        .unwrap_or(0)
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
    let mut lines = reader.lines().skip(2);
    let mut amphipods_state = vec![];

    for idx in [0, 1] {
        let line = lines.next().with_context(|| format!("Missing line"))??;
        let mut states = line
            .split('#')
            .filter(|s| !s.trim().is_empty())
            .enumerate()
            .map(|(i, s)| AmphipodState {
                typ: AmphipodType::from_char(s.chars().next().unwrap()),
                position: AmphipodPosition::Room {
                    typ: ROOM_TYPES[i],
                    idx,
                },
                has_been_in_hallway: false,
            })
            .collect::<Vec<_>>();
        amphipods_state.append(&mut states);
    }

    Ok(BurrowState {
        amphipods_state,
        room_size: 2,
        energy: 0,
    })
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
        |#############
        |#...........#
        |###B#C#B#D###
        |  #A#D#C#A#
        |  #########";

    fn as_input(s: &str) -> Result<Input> {
        read_input(BufReader::new(
            s.split('\n')
                .skip(1)
                .map(|s| s.trim().split('|').skip(1).next().unwrap())
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes(),
        ))
    }

    #[test]
    fn test_part1() -> Result<()> {
        assert_eq!(part1(&as_input(INPUT)?), 12521);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input(INPUT)?), 44169);
        Ok(())
    }
}
