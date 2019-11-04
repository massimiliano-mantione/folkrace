use hal::{ProtocolBuffer,new_protocol_buffer};
use crate::map::*;
use crate::protocol::*;
use crate::V3;

fn buffer_from_str(s: &str) -> ProtocolBuffer {
    let mut buffer = new_protocol_buffer();
    for (i, c) in s.chars().enumerate() {
        buffer[i] = c as u8;
        buffer[i + 1] = '\n' as u8;
    }
    buffer
}

static SECTIONS: [&str; 7] = [
    "MAP-SECTION:0:STRAIGHT:1000:800:800",
    "MAP-SECTION:1:LEFT:180:800:800:500:500",
    "MAP-SECTION:2:RIGHT:90:800:800:500:500",
    "MAP-SECTION:3:LEFT:180:800:800:500:500",
    "MAP-SECTION:4:UP:500:300:800:800",
    "MAP-SECTION:5:DOWN:500:300:800:800",
    "MAP-SECTION:6:LEFT:90:800:800:500:500",
];

fn new_map(sections: &[&str]) -> Map {
    let mut map = Map::new();
    map.reset();
    for s in sections.iter() {
        let b = buffer_from_str(s);
        let cmd = BotCommand::parse(&b).unwrap();
        if let BotCommand::MapSection(section) = cmd {
            let index = section.index;
            let section = MapSection::from_protocol_data(&section.data);
            map.configure_section(index, &section);
        }
    }
    map.complete_configuration();
    map
}

fn check_relative_eq(v1: V3, v2: V3) {
    let result = (v1 - v2).magnitude() < 0.001;
    if !result {
        println!("check_relative_eq failed: {} != {}", v1, v2);
    }
    assert!(result);
}

#[test]
fn parses_map() {
    let map = new_map(&SECTIONS);
    assert_eq!(map.length, 7);

    check_relative_eq(map[0].start, V3::new(0.0, 0.0, 0.0));
    check_relative_eq(map[0].center, V3::new(0.0, 0.0, 0.5));
    check_relative_eq(map[0].end, V3::new(0.0, 0.0, 1.0));

    check_relative_eq(map[1].start, V3::new(0.0, 0.0, 1.0));
    check_relative_eq(map[1].center, V3::new(0.5, 0.0, 1.0));
    check_relative_eq(map[1].end, V3::new(1.0, 0.0, 1.0));

    check_relative_eq(map[2].start, V3::new(1.0, 0.0, 1.0));
    check_relative_eq(map[2].center, V3::new(1.5, 0.0, 1.0));
    check_relative_eq(map[2].end, V3::new(1.5, 0.0, 0.5));

    check_relative_eq(map[3].start, V3::new(1.5, 0.0, 0.5));
    check_relative_eq(map[3].center, V3::new(1.5, 0.0, 0.0));
    check_relative_eq(map[3].end, V3::new(1.5, 0.0, -0.5));

    check_relative_eq(map[4].start, V3::new(1.5, 0.0, -0.5));
    check_relative_eq(map[4].center, V3::new(1.25, 0.15, -0.5));
    check_relative_eq(map[4].end, V3::new(1.0, 0.3, -0.5));

    check_relative_eq(map[5].start, V3::new(1.0, 0.3, -0.5));
    check_relative_eq(map[5].center, V3::new(0.75, 0.15, -0.5));
    check_relative_eq(map[5].end, V3::new(0.5, 0.0, -0.5));

    check_relative_eq(map[6].start, V3::new(0.5, 0.0, -0.5));
    check_relative_eq(map[6].center, V3::new(0.5, 0.0, 0.0));
    check_relative_eq(map[6].end, V3::new(0.0, 0.0, 0.0));
}
