use std::fs::File;
use std::io::BufReader;

use ogg_ng::{BitStreamReader, Packet, ReadStatus};

#[test]
pub fn parse_bitstream() {
    let mut file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut br = BitStreamReader::default();

    let mut packet = Packet::default();
    (0..10000).for_each(|_| {
        br.read_packet(&mut file, &mut packet).unwrap();
    });
}

#[test]
pub fn seek_bitstream_direct() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut buffered_file = BufReader::new(file);
    let mut br = BitStreamReader::default();

    br.seek(&mut buffered_file, 4032960).unwrap();

    let mut packet = Packet::default();
    br.read_packet(&mut buffered_file, &mut packet).unwrap();

    assert_eq!(packet.granule_position(), 4032960);
}

#[test]
pub fn seek_bitstream_min() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut buffered_file = BufReader::new(file);
    let mut br = BitStreamReader::default();

    br.seek(&mut buffered_file, 0).unwrap();

    let mut packet = Packet::default();
    br.read_packet(&mut buffered_file, &mut packet).unwrap();

    assert_eq!(packet.granule_position(), 0);
}

#[test]
pub fn seek_bitstream_close_to_min() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut buffered_file = BufReader::new(file);
    let mut br = BitStreamReader::default();

    br.seek(&mut buffered_file, 50).unwrap();

    let mut packet = Packet::default();
    br.read_packet(&mut buffered_file, &mut packet).unwrap();

    assert_eq!(packet.granule_position(), 26880);
}

#[test]
pub fn seek_bitstream_outside() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut buffered_file = BufReader::new(file);
    let mut br = BitStreamReader::default();

    br.seek(&mut buffered_file, u64::MAX - 1).unwrap();

    let mut packet = Packet::default();
    let ret = br.read_packet(&mut buffered_file, &mut packet).unwrap();

    assert_eq!(ret, ReadStatus::Eof);
}
