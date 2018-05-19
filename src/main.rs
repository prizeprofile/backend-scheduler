extern crate rand;

mod region;
mod event;
mod sqs;
mod boot;
mod sns;

use std::thread;
use region::Region;
use std::sync::mpsc;
use event::InputEvent;
use event::OutputEvent;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

fn start_output_channel(regions: &mut Vec<Region>) {
    let (tx, rx): (Sender<OutputEvent>, Receiver<OutputEvent>) = mpsc::channel();

    for region in regions {
        region.channel(mpsc::Sender::clone(&tx));
    }

    thread::spawn(move || {
        for event in rx {
            thread::spawn(move || sqs::push(event));
        }
    });
}

fn main() {
    let mut regions: Vec<Region> = boot::get_regions();

    start_output_channel(&mut regions);

    let rx: Receiver<InputEvent> = sqs::listen();

    for event in rx {
        let mut region = regions.iter_mut()
            .find(|region| region.id == event.region_id);
            
        match region {
            Some(ref mut region) => region.handle_event(event),
            None => sns::notify(String::from("Unknown region ID"), sns::Priority::SEVERE),
        };
    }
}

