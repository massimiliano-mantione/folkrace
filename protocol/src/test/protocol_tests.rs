use hal::{ProtocolBuffer,new_protocol_buffer};
use crate::protocol::*;

fn buffer_from_str(s: &str) -> ProtocolBuffer {
    let mut buffer = new_protocol_buffer();
    for (i, c) in s.chars().enumerate() {
        buffer[i] = c as u8;
        buffer[i + 1] = '\n' as u8;
    }
    buffer
}

fn buffer_to_string(b: &ProtocolBuffer) -> String {
    let mut s = String::new();
    for c in b.iter() {
        if *c == '\n' as u8 {
            break;
        }
        s.push(*c as char);
    }
    s
}

static COMMANDS: [&str; 11] = [
    "MAP-START:5",
    "MAP-SECTION:0:STRAIGHT:1000:800:800",
    "MAP-SECTION:1:LEFT:90:800:800:500:500",
    "MAP-SECTION:2:RIGHT:90:800:800:500:500",
    "MAP-SECTION:3:UP:1000:30:800:800",
    "MAP-SECTION:4:DOWN:1000:30:800:800",
    "MAP-END",
    "RESET",
    "PAUSE",
    "RESTART",
    "DIRECT:100:-100:0:50",
];

static EVENTS: [&str; 11] = [
    "STATUS:INVALID-MAP",
    "STATUS:DEVICE-ERROR",
    "STATUS:STOPPED",
    "STATUS:WAITING:1000:300",
    "STATUS:RACING:2:0:100:-100:100",
    "STATUS:RACING:2:20:70:-90:-20",
    "STATUS:RACING:2:60:100:20:100",
    "LASERS:101:102:103:104:105:106:107:108:109:110:111:112:113:114:115:116:117:118:119:120",
    "IMU:0:0:45:0:0:0:0:0:-1",
    "IMU:2:-5:-45:12:23:4:1:-1:-5",
    "LOG:This is a lovely log message",
];

#[test]
fn it_converts_buffers() {
    let b = buffer_from_str("my buffer");
    let s = buffer_to_string(&b);
    assert_eq!(s, "my buffer");
}

#[test]
fn it_handles_commands() {
    for s in COMMANDS.iter() {
        let sb = buffer_from_str(s);
        match BotCommand::parse(&sb) {
            Ok(cmd) => {
                let mut rb = new_protocol_buffer();
                cmd.write(&mut rb);
                let rs = buffer_to_string(&rb);
                assert_eq!(*s, rs);
            }
            Err(index) => panic!(format!("error parsing {} at index {}", s, index)),
        }
    }
}

#[test]
fn it_handles_events() {
    for s in EVENTS.iter() {
        let sb = buffer_from_str(s);
        match BotEvent::parse(&sb) {
            Ok(cmd) => {
                let mut rb = new_protocol_buffer();
                cmd.write(&mut rb);
                let rs = buffer_to_string(&rb);
                assert_eq!(*s, rs);
            }
            Err(index) => panic!(format!("error parsing {} at index {}", s, index)),
        }
    }
}
