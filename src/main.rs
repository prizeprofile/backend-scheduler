extern crate rand;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use rand::distributions::{IndependentSample, Range};

struct InputEvent {
    region_id: u8,
    since_id: u64,
    tweets_count: u8,
    error: Option<u8>,
}

struct OutputEvent {
    delay: u8,
    region_id: u8,
    since_id: u64,
    params: String,
}

struct Region {
    id: u8,
    tick: u8,
    since_id: u64,
    params: String,
    channel: Option<Sender<OutputEvent>>,
}

impl Region {    
    pub fn new(id: u8, params: String) -> Region {
        Region {
            id: id,
            tick: 1,
            since_id: 0,
            channel: None,
            params: params,
        }
    }

    pub fn tick(&mut self, seconds: u8) -> &mut Region {
        self.tick = seconds;
        self
    }

    pub fn channel(&mut self, tx: Sender<OutputEvent>) -> &mut Region {
        self.channel = Some(tx);
        self
    }

    pub fn since_id(&mut self, since_id: u64) -> &mut Region {
        self.since_id = since_id;
        self
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        if event.since_id > self.since_id {
            self.since_id = event.since_id.clone();
        }

        let delay: u8 = match event.error {
            Some(wait_before_restart) => self.tick + wait_before_restart, 
            None => self.tick + 100 - event.tweets_count
        };

        let payload = OutputEvent {
            delay: delay,
            region_id: self.id,
            params: self.params,
            since_id: self.since_id,
        };

        // TODO: Propagate error.
        match self.channel {
            Some(channel) => channel.send(payload).unwrap(),
            None => println!("TODO: Propagate error."),
        }
    }
}

fn get_regions() -> Vec<Region> {
    let mut regions: Vec<Region> = Vec::new();

    let region = Region::new(1, String::from(":]"));
    region.since_id(1).tick(1);
    regions.push(region);

    let region = Region::new(2, String::from(":]"));
    region.since_id(2).tick(3);
    regions.push(region);

    regions
}

fn boot_output_channel(regions: &mut Vec<Region>) {
    let (tx, rx): (Sender<OutputEvent>, Receiver<OutputEvent>) = mpsc::channel();

    for region in regions {
        region.channel(mpsc::Sender::clone(&tx));
    }

    // TODO: Consider returning thread pointer.
    thread::spawn(move || {
        for event in rx {
            println!("Output!");
        }
    });
}

fn listen_to_sqs() -> Receiver<InputEvent> { 
    let (tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

    // TODO: Poll from AWS SQS.
    thread::spawn(move || mock_events(tx));

    rx
}

fn main() {
    let mut regions: Vec<Region> = get_regions();

    boot_output_channel(&mut regions);

    let rx: Receiver<InputEvent> = listen_to_sqs();

    for event in rx {
        let mut region = regions.iter_mut()
            .find(|region| region.id == event.region_id);
            
        match region {
            Some(ref mut region) => region.handle_event(event),
            // TODO: Error handling.
            None => println!("No region found for id {}.", event.region_id),
        };
    }
}

fn mock_events(tx: Sender<InputEvent>) {
    let mut rng = rand::thread_rng();
    let mut i: u32 = 0;

    loop {
        let tweets_count: u8 = 100 - Range::new(0, 4).ind_sample(&mut rng);
        let since_id: u64 = tweets_count as u64 + Range::new(10, 100).ind_sample(&mut rng) as u64;
        let region_id: u8 = Range::new(1, 3).ind_sample(&mut rng);
        let event: InputEvent = InputEvent {
            error: None::<u8>,
            since_id: since_id,
            region_id: region_id,
            tweets_count: tweets_count,
        };
        
        tx.send(event).unwrap();

        i += 1;
        thread::sleep(Duration::from_secs((3 + 100 - tweets_count).into()).into());
    }
}
