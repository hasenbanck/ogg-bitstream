use std::fs::File;

use ogg_ng::{BitStreamReader, Packet};

#[test]
pub fn parse_bitstream() {
    let mut file = File::open("tests/data/02 - ANANT-GARDE EYES - theme of SSS.ogg").unwrap();
    let mut br = BitStreamReader::default();

    let mut packet = Packet::default();
    (0..10000).for_each(|_| {
        br.read_packet(&mut file, &mut packet).unwrap();
    });
}
