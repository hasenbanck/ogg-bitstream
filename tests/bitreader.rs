use std::fs::File;

use ogg_ng::{BitStreamFileReader, Packet, ReadStatus};

// TODO write dynamic test cases once the writer is working.

#[test]
pub fn parse_bitstream() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut br = BitStreamFileReader::new(file);

    let mut packet = Packet::default();
    (0..10000).for_each(|_| {
        br.read_packet(&mut packet).unwrap();
    });
}

#[test]
pub fn seek_bitstream_direct() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut br = BitStreamFileReader::new(file);

    let mut packet = Packet::default();
    br.read_packet(&mut packet).unwrap();

    br.seek(packet.bitstream_serial_number(), 4032960).unwrap();

    br.read_packet(&mut packet).unwrap();

    assert_eq!(packet.granule_position(), 4032960);
}

#[test]
pub fn seek_bitstream_min() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut br = BitStreamFileReader::new(file);

    let mut packet = Packet::default();
    br.read_packet(&mut packet).unwrap();

    br.seek(packet.bitstream_serial_number(), 0).unwrap();

    br.read_packet(&mut packet).unwrap();

    assert_eq!(packet.granule_position(), 0);
}

#[test]
pub fn seek_bitstream_close_to_min() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut br = BitStreamFileReader::new(file);

    let mut packet = Packet::default();
    br.read_packet(&mut packet).unwrap();

    br.seek(packet.bitstream_serial_number(), 50).unwrap();

    br.read_packet(&mut packet).unwrap();

    assert_eq!(packet.granule_position(), 26880);
}

#[test]
pub fn seek_bitstream_outside() {
    let file = File::open("tests/data/Made in Abyss.opus").unwrap();
    let mut br = BitStreamFileReader::new(file);

    let mut packet = Packet::default();
    br.read_packet(&mut packet).unwrap();

    br.seek(packet.bitstream_serial_number(), u64::MAX - 1)
        .unwrap();

    let ret = br.read_packet(&mut packet).unwrap();
    assert_eq!(ret, ReadStatus::Ok);

    let granule_position = packet.granule_position();

    // Make sure that we only have packets from the same page left.
    while ReadStatus::Ok == br.read_packet(&mut packet).unwrap() {
        assert_eq!(packet.granule_position(), granule_position);
    }
}
