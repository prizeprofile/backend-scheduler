//! Region struct represents a parameter collection for Tweet search.

use event::InputEvent;
use event::OutputEvent;
use std::sync::mpsc::Sender;

/// A single region struct.
pub struct Region {
    /// Private integer representing the minumum delay between API calls in seconds.
    tick: u8,

    /// An identifier.
    pub id: u8,

    /// BigInteger with the highest Tweet ID in a region.
    pub since_id: u64,

    /// JSON string of parameters that define a region.
    pub params: String,

    /// A `std::sync::mpsc::Sender` which streams `event::OutputEvent` to
    /// `sqs::stream` and pushes them to an AWS SQS.
    pub channel: Option<Sender<OutputEvent>>,
}

impl Region {
    /// Builds a new default `Region`.
    /// TODO: Implement `Default` for `Region`.
    pub fn new(id: u8, params: String) -> Region {
        Region {
            id: id,
            tick: 1,
            since_id: 0,
            channel: None,
            params: params,
        }
    }

    /// Setter for `Region.tick`.
    pub fn tick(&mut self, seconds: u8) -> &mut Region {
        self.tick = seconds;
        self
    }

    /// Setter for `Region.channel`.
    pub fn channel(&mut self, tx: Sender<OutputEvent>) -> &mut Region {
        self.channel = Some(tx);
        self
    }

    /// Setter for `Region.since_id`.
    pub fn since_id(&mut self, since_id: u64) -> &mut Region {
        self.since_id = since_id;
        self
    }

    /// Handles an incoming SQS message and generates an outcoming one.
    /// TODO: Implement error propagation.
    pub fn handle_event(&mut self, event: InputEvent) {
        // Only update since_id if max_id of incoming event is higher.
        if event.max_id > self.since_id {
            self.since_id = event.max_id.clone() + 1;
        }

        // Decides how many seconds should the scheduler wait before rescheduling.
        let delay: u8 = match event.error {
            // If there is an error, it will have an information about how long
            // should we wait.
            Some(wait_before_restart) => self.tick + wait_before_restart, 
            
            // If there is no error, wait for `Region.tick` seconds plus
            // an extra second for every empty slot in the last call.
            None => self.tick + 100 - event.tweets_count,
        };

        // Create a payload.
        let payload = OutputEvent {
            delay: delay,
            region_id: self.id,
            params: self.params.clone(),
            since_id: self.since_id,
        };

        // Stream the payload down the channel.
        match self.channel {
            Some(ref mut channel) => channel.send(payload).unwrap(),
            None => println!("TODO: Propagate error."),
        }
    }
}

