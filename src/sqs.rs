use std::thread;
use std::sync::mpsc;
use event::InputEvent;
use event::OutputEvent;
use rand::{thread_rng};
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use rand::distributions::{IndependentSample, Range};

pub fn push(event: OutputEvent) {
    thread::sleep(Duration::from_secs(event.delay.into()));

    // TODO: Push a new message to SQS.
    println!("Get since_id {} for {}.", event.since_id, event.params);
}

pub fn listen() -> Receiver<InputEvent> { 
    let (tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

    // TODO: Poll from AWS SQS.
    thread::spawn(move || mock_events(tx));

    rx
}

// For debug purposes.
fn mock_events(tx: Sender<InputEvent>) {
    let mut rng = thread_rng();
    let mut i: u64 = 0;

    loop {
        let event: InputEvent = InputEvent {
            error: None::<u8>,
            since_id: i,
            region_id: Range::new(1, 3).ind_sample(&mut rng),
            tweets_count: 100,
        };
        
        tx.send(event).unwrap();
        i += Range::new(1, 3).ind_sample(&mut rng);
        thread::sleep(Duration::from_secs(1));
    }
}
