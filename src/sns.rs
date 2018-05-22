extern crate rusoto_sns;

use std::thread;
use region::TweetRegion;
use std::sync::mpsc;
use event::OutputEvent;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

// TODO: Refactor this.
pub enum Topic {
    UnknownRegion,
    SqsConnDropped,
}

pub fn notify(topic: Topic) {
    let message = match topic {
        Topic::UnknownRegion => "Unknown region.",
        Topic::SqsConnDropped => "Connection to SNS dropped.",
    };

    // TODO: Push to SNS.
    println!("{}", message);
}

/// Clones `std::sync::mpcs::Sender` to every item in given `Vec<TweetRegion>`
/// and spawns a new thread that listens to messages from this channel.
/// The message type that is sent across the channel has to be of `OutputEvent` type.
pub fn stream(regions: &mut Vec<TweetRegion>) {
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

/// Pushes a new message to AWS SNS.
fn push(event: OutputEvent) {
    // Before pushing the message, thread sleeps for certain amount of time.
    // This is to regulate the number of API calls we do to Twitter.
    thread::sleep(Duration::from_secs(event.delay.into()));

    // TODO: Push a new message to SNS.
    println!("Get since_id {} for {}.", event.since_id, event.params);
}
