use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::{Context, Result};
use bitvec::prelude::*;

use utils::measure;

type Input = Vec<u8>;

#[derive(Debug, Eq, PartialEq)]
enum PacketType {
    Literal(u64),
    Operator(Operator, Vec<Packet>),
}

#[derive(Debug, Eq, PartialEq)]
enum Operator {
    Sum,
    Product,
    Minimum,
    Maximum,
    GreaterThan,
    LessThan,
    EqualTo,
}

impl Operator {
    fn from_type_id(type_id: u8) -> Self {
        use Operator::*;
        match type_id {
            0 => Sum,
            1 => Product,
            2 => Minimum,
            3 => Maximum,
            5 => GreaterThan,
            6 => LessThan,
            7 => EqualTo,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Packet {
    version: u8,
    packet_type: PacketType,
}

impl Packet {
    fn decode_bytes(input: &Vec<u8>) -> Packet {
        let bits = input.view_bits::<Msb0>();
        let (packet, _) = Packet::decode_bits(bits);
        packet
    }

    fn decode_bits(bits: &BitSlice<Msb0, u8>) -> (Packet, usize) {
        let version = bits[0..3].load_be::<u8>();
        let type_id = bits[3..6].load_be::<u8>();
        let mut consumed = 6;

        let packet_type = match type_id {
            4 => {
                let mut b_idx = 6;
                let mut lit_bits = bitvec![Msb0, u8;];
                loop {
                    consumed += 5;
                    let next = &bits[(b_idx + 1)..(b_idx + 5)];
                    lit_bits.extend_from_bitslice(next);

                    if !bits[b_idx] {
                        break;
                    }
                    b_idx += 5;
                }

                PacketType::Literal(lit_bits.load_be())
            }
            _ => {
                let mut packets = vec![];

                let mut b_idx = 7;
                let lenght_type_id = bits[6];
                consumed += 1;

                if lenght_type_id == false {
                    let subpackets_len = bits[(b_idx)..(b_idx + 15)].load_be::<usize>();
                    b_idx += 15;
                    consumed += 15;

                    let subpackets = &bits[(b_idx)..(b_idx + subpackets_len)];
                    let mut s_idx = 0;

                    while s_idx < subpackets_len - 1 {
                        let (packet, consumed) = Packet::decode_bits(&subpackets[(s_idx)..]);
                        s_idx += consumed;
                        packets.push(packet);
                    }

                    consumed += subpackets_len;
                } else {
                    let n_subpackets = bits[(b_idx)..(b_idx + 11)].load_be::<usize>();
                    b_idx += 11;
                    consumed += 11;

                    for _ in 0..n_subpackets {
                        let (packet, pconsumed) = Packet::decode_bits(&bits[(b_idx)..]);
                        b_idx += pconsumed;
                        consumed += pconsumed;
                        packets.push(packet);
                    }
                }

                PacketType::Operator(Operator::from_type_id(type_id), packets)
            }
        };

        (
            Packet {
                version,
                packet_type,
            },
            consumed,
        )
    }

    fn sum_version(&self) -> i32 {
        match &self.packet_type {
            PacketType::Literal(_) => self.version as i32,
            PacketType::Operator(_, packets) => {
                packets.iter().map(|p| p.sum_version()).sum::<i32>() + self.version as i32
            }
        }
    }

    fn calculate(&self) -> u64 {
        use Operator::*;
        match &self.packet_type {
            PacketType::Literal(value) => *value,
            PacketType::Operator(op, packets) => {
                let mut values = packets.iter().map(|p| p.calculate());
                match op {
                    Sum => values.sum(),
                    Product => values.product(),
                    Minimum => values.min().unwrap_or(0),
                    Maximum => values.max().unwrap_or(0),
                    GreaterThan => (values.next() > values.next()) as u64,
                    LessThan => (values.next() < values.next()) as u64,
                    EqualTo => (values.next() == values.next()) as u64,
                }
            }
        }
    }
}

fn part1(input: &Input) -> i32 {
    Packet::decode_bytes(input).sum_version()
}

fn part2(input: &Input) -> u64 {
    Packet::decode_bytes(input).calculate()
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
    const HEX_2_BIN: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    ];
    fn hex2bin(c: char) -> u8 {
        HEX_2_BIN
            .iter()
            .enumerate()
            .find(|&(_, hc)| hc == &c)
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }

    let chars = reader
        .lines()
        .next()
        .context("No input")??
        .chars()
        .collect::<Vec<_>>();
    let in_binary = chars
        .chunks(2)
        .map(|c| {
            let nibble1 = hex2bin(c[0]);
            let nibble2 = hex2bin(c[1]);
            (nibble1 << 4) | nibble2
        })
        .collect::<Vec<_>>();
    Ok(in_binary)
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
                .map(|s| s.trim())
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes(),
        ))
    }

    #[test]
    fn test_decode_packet() -> Result<()> {
        assert_eq!(
            Packet::decode_bytes(&as_input("D2FE28")?),
            Packet {
                version: 6,
                packet_type: PacketType::Literal(2021)
            }
        );

        assert_eq!(
            Packet::decode_bytes(&as_input("38006F45291200")?),
            Packet {
                version: 1,
                packet_type: PacketType::Operator(
                    Operator::LessThan,
                    vec![
                        Packet {
                            version: 6,
                            packet_type: PacketType::Literal(10)
                        },
                        Packet {
                            version: 2,
                            packet_type: PacketType::Literal(20)
                        }
                    ]
                )
            }
        );

        assert_eq!(
            Packet::decode_bytes(&as_input("EE00D40C823060")?),
            Packet {
                version: 7,
                packet_type: PacketType::Operator(
                    Operator::Maximum,
                    vec![
                        Packet {
                            version: 2,
                            packet_type: PacketType::Literal(1)
                        },
                        Packet {
                            version: 4,
                            packet_type: PacketType::Literal(2)
                        },
                        Packet {
                            version: 1,
                            packet_type: PacketType::Literal(3)
                        }
                    ]
                )
            }
        );

        Ok(())
    }

    #[test]
    fn test_part1() -> Result<()> {
        assert_eq!(part1(&as_input("8A004A801A8002F478")?), 16);
        assert_eq!(part1(&as_input("620080001611562C8802118E34")?), 12);
        assert_eq!(part1(&as_input("C0015000016115A2E0802F182340")?), 23);
        assert_eq!(part1(&as_input("A0016C880162017C3686B18A3D4780")?), 31);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        assert_eq!(part2(&as_input("C200B40A82")?), 3);
        assert_eq!(part2(&as_input("04005AC33890")?), 54);
        assert_eq!(part2(&as_input("880086C3E88112")?), 7);
        assert_eq!(part2(&as_input("CE00C43D881120")?), 9);
        assert_eq!(part2(&as_input("D8005AC2A8F0")?), 1);
        assert_eq!(part2(&as_input("F600BC2D8F")?), 0);
        assert_eq!(part2(&as_input("9C005AC2F8F0")?), 0);
        assert_eq!(part2(&as_input("9C0141080250320F1802104A08")?), 1);
        Ok(())
    }
}
