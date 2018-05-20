use std::thread;
use std::sync::mpsc;
use event::InputEvent;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
// TODO: Remove.
use rand::{thread_rng};
use rand::distributions::{IndependentSample, Range};

/// Starts polling messages from AWS SQS and returns the receiver object.
pub fn listen() -> Receiver<InputEvent> {
    // Boots a new channel.
    let (tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

    // TODO: Poll from AWS SQS.
    thread::spawn(move || mock_events(tx));

    // Returns the single receiver.
    rx
}

// For debug purposes.
fn mock_events(tx: Sender<InputEvent>) {
    let mut rng = thread_rng();
    let mut i: u64 = 0;

    loop {
        let event: InputEvent = InputEvent {
            error: None::<u8>,
            max_id: i,
            region_id: Range::new(1, 3).ind_sample(&mut rng),
            tweets_count: 100,
        };
        
        tx.send(event).unwrap();
        i += Range::new(1, 3).ind_sample(&mut rng);
        thread::sleep(Duration::from_secs(1));
    }
}
