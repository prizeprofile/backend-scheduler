use std::thread;
use region::Region;
use std::sync::mpsc;
use event::InputEvent;
use event::OutputEvent;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
// TODO: Remove.
use rand::{thread_rng};
use rand::distributions::{IndependentSample, Range};

/// Clones `std::sync::mpcs::Sender` to every item in given `Vec<Region>`
/// and spawns a new thread that listens to messages from this channel.
/// The message type that is sent across the channel has to be of `OutputEvent` type.
pub fn stream(regions: &mut Vec<Region>) {
    // Boots a new channel.
    let (tx, rx): (Sender<OutputEvent>, Receiver<OutputEvent>) = mpsc::channel();

    for region in regions {
        // Clones the transmiter to a region.
        region.channel(mpsc::Sender::clone(&tx));
    }

    // Spawns a new thread and moves the ownership of the transmiter.
    thread::spawn(move || {
        for event in rx {
            // Every time a message comes down the stream, spawns a new thread
            // and calls sqs::push. This is important so that pushing the event
            // doesn't block other regions.
            thread::spawn(move || push(event));
        }
    });
}

/// Starts listening to messages from AWS SQS and returns the receiver object.
pub fn listen() -> Receiver<InputEvent> {
    // Boots a new channel.
    let (tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

    // TODO: Poll from AWS SQS.
    thread::spawn(move || mock_events(tx));

    // Returns the single receiver.
    rx
}

/// Pushes a new message to AWS SQS.
fn push(event: OutputEvent) {
    // Before pushing the message, thread sleeps for certain amount of time.
    // This is to regulate the number of API calls we do to Twitter.
    thread::sleep(Duration::from_secs(event.delay.into()));

    // TODO: Push a new message to SQS.
    println!("Get since_id {} for {}.", event.since_id, event.params);
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
