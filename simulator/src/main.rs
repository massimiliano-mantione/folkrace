use hal::{new_protocol_buffer, ProtocolBuffer};
use map::*;
use protocol::map::{Map, MapSection};
use protocol::protocol::BotCommand;

static SECTIONS: [&str; 7] = [
    "MAP-SECTION:0:STRAIGHT:1000:800:800",
    "MAP-SECTION:1:LEFT:180:800:800:500:500",
    "MAP-SECTION:2:RIGHT:90:800:800:500:500",
    "MAP-SECTION:3:LEFT:180:800:800:500:500",
    "MAP-SECTION:4:UP:500:300:800:800",
    "MAP-SECTION:5:DOWN:500:300:800:800",
    "MAP-SECTION:6:LEFT:90:800:800:500:500",
];
fn buffer_from_str(s: &str) -> ProtocolBuffer {
    let mut buffer = new_protocol_buffer();
    for (i, c) in s.chars().enumerate() {
        buffer[i] = c as u8;
        buffer[i + 1] = '\n' as u8;
    }
    buffer
}

fn setup_map() -> Map {
    let mut map = Map::new();
    map.reset();
    for s in SECTIONS.iter() {
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

#[allow(dead_code)]
fn main_testbed() {
    let map = setup_map();
    let mut world = simulation::SimulatedWorld::new();
    world.setup_map(&map);
    world.set_motor_power(0.9, 0.9, 0.9, 0.9);
    world.apply_power();
    world.run_testbed();
}

fn main_full() {
    let car = Car::new();
    let map = setup_map();

    let mut visual_world = display::VisualizedWorld::new(&car);
    visual_world.setup_map(&map);
    let mut simulated_world = simulation::SimulatedWorld::new();
    simulated_world.setup_map(&map);

    // simulated_world.set_motor_power(0.4, 0.4, 0.4, 0.4);

    while visual_world.render() {
        simulated_world.step();
        simulated_world.step();

        let pos = simulated_world.body_position();

        visual_world.set_car_position(NaV3::new(
            pos.translation.x,
            pos.translation.y,
            pos.translation.z,
        ));
        visual_world.set_car_rotation(pos.rotation);
        visual_world.set_wheel_angles(
            simulated_world.wheel_rotation_bl(),
            simulated_world.wheel_rotation_br(),
            simulated_world.wheel_rotation_fl(),
            simulated_world.wheel_rotation_fr(),
        );

        match visual_world.ui().power {
            Some(power) => {
                let (power_bl, power_br, power_fl, power_lr) = power.power();
                println!("power {} {} {} {}", power_bl, power_br, power_fl, power_lr);
                simulated_world.set_motor_power(power_bl, power_br, power_fl, power_lr);
            }
            None => {
                simulated_world.set_motor_power(0.0, 0.0, 0.0, 0.0);
            }
        }

        /*
        println!(
            "w_vel {} {} {} {}",
            simulated_world.wheel_velocity_bl().to_degrees(),
            simulated_world.wheel_velocity_br().to_degrees(),
            simulated_world.wheel_velocity_fl().to_degrees(),
            simulated_world.wheel_velocity_fr().to_degrees(),
        );
        */
    }
}

fn main() {
    main_full();
}
