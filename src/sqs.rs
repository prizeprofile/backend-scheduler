extern crate rusoto_sqs;

// use std::thread;
use std::sync::mpsc;
use event::InputEvent;
// use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

/// Starts polling messages from AWS SQS and returns the receiver object.
pub fn listen() -> Receiver<InputEvent> {
    // Boots a new channel.
    let (_tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

    // TODO: Poll from AWS SQS.
    // thread::spawn(move || mock_events(tx));

    // Returns the single receiver.
    rx
}

