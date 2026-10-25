//! Spike S1 on Windows and Linux (T105): logs MIDI ports and messages as the engine sees them
//! through `midir`, to measure disconnect detection, reconnect, latency and jitter.
//!
//! `cargo run -p legato-engine --example midi_probe [-- <part of the port name>]`
//!
//! Without a name only port changes are logged. With a name the matching port is opened
//! (again after it returns); every message shows its arrival time, the driver timestamp and
//! the offset between both relative to the first message, whose spread is the jitter.

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn main() {
    eprintln!("midi_probe runs on Windows and Linux only");
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn main() {
    use std::collections::BTreeMap;
    use std::time::{Duration, Instant};

    use midir::{Ignore, MidiInput};

    let filter = std::env::args().nth(1).map(|s| s.to_lowercase());
    let start = Instant::now();
    let ms = move || start.elapsed().as_millis();
    let midi = MidiInput::new("Legato probe").expect("MIDI not available");
    let mut known: BTreeMap<String, String> = BTreeMap::new();
    let mut open: Option<(String, midir::MidiInputConnection<()>)> = None;

    loop {
        let current: BTreeMap<String, String> = midi
            .ports()
            .iter()
            .filter_map(|p| Some((p.id(), midi.port_name(p).ok()?)))
            .collect();
        for (id, name) in &known {
            if !current.contains_key(id) {
                println!("{:>8} ms  - {name}  [{id}]", ms());
                if open.as_ref().is_some_and(|(open_id, _)| open_id == id) {
                    open.take().unwrap().1.close();
                }
            }
        }
        for (id, name) in &current {
            if !known.contains_key(id) {
                println!("{:>8} ms  + {name}  [{id}]", ms());
            }
        }
        known = current;

        if open.is_none()
            && let Some(filter) = &filter
            && let Some((id, name)) = known
                .iter()
                .find(|(_, name)| name.to_lowercase().contains(filter))
        {
            let mut input = MidiInput::new("Legato probe").expect("MIDI not available");
            input.ignore(Ignore::All);
            let port = input.find_port_by_id(id).expect("port just listed");
            let mut first: Option<i128> = None;
            // WinRT may block here for Bluetooth devices that are paired but not reachable.
            println!("{:>8} ms  connecting to {name} ...", ms());
            match input.connect(
                &port,
                "Legato probe",
                move |driver_us, data, _| {
                    let arrival_us = start.elapsed().as_micros() as i128;
                    let offset = arrival_us - driver_us as i128;
                    let jitter_us = offset - *first.get_or_insert(offset);
                    println!(
                        "{:>8} ms  driver {driver_us:>12} us  jitter {jitter_us:>+7} us  {data:02X?}",
                        arrival_us / 1000
                    );
                },
                (),
            ) {
                Ok(connection) => {
                    println!("{:>8} ms  connected to {name}", ms());
                    open = Some((id.clone(), connection));
                }
                Err(e) => println!("{:>8} ms  cannot connect to {name}: {e}", ms()),
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}
