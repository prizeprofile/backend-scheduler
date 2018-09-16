//! TweetRegion struct represents a parameter collection for Tweet search.

use sqs;
use sns;
use event::InputEvent;
use event::OutputEvent;
use std::sync::mpsc::Sender;

/// A single region struct.
pub struct TweetRegion {
    /// Private integer representing the minumum delay between API calls in seconds.
    tick: u64,

    /// An identifier.
    pub id: u64,

    /// BigInteger with the highest Tweet ID in a region.
    pub since_id: u64,

    /// JSON string of parameters that define a region.
    pub params: String,

    /// SNS Topic that region should be pushed to.
    pub topic: String,

    /// A `std::sync::mpsc::Sender` which streams `event::OutputEvent` to
    /// `sqs::stream` and pushes them to an AWS SQS.
    pub channel: Option<Sender<OutputEvent>>,
}

impl TweetRegion {
    /// Builds a new default `TweetRegion`.
    pub fn new(id: u64, topic: String, params: String) -> TweetRegion {
        TweetRegion {
            id: id,
            tick: 1,
            since_id: 0,
            channel: None,
            topic: topic,
            params: params,
        }
    }

    /// Setter for `TweetRegion.tick`.
    pub fn tick(&mut self, seconds: u64) -> &mut TweetRegion {
        self.tick = seconds;
        self
    }

    /// Setter for `TweetRegion.channel`.
    pub fn channel(&mut self, tx: Sender<OutputEvent>) -> &mut TweetRegion {
        self.channel = Some(tx);
        self
    }

    /// Setter for `TweetRegion.since_id`.
    pub fn since_id(&mut self, since_id: u64) -> &mut TweetRegion {
        self.since_id = since_id;
        self
    }

    /// Handles an incoming SQS message and generates an outcoming one.
    pub fn handle_event(&mut self, event: InputEvent) {
        // Only update since_id if max_id of incoming event is higher.
        if event.max_id <= self.since_id {
            return
        }

        self.since_id = event.max_id.clone();

        // Decides how many seconds should the scheduler wait before rescheduling.
        let delay: u64 = match event.error {
            // If there is an error, it will have an information about how long
            // should we wait.
            Some(wait_before_restart) => self.tick + wait_before_restart,

            // If there is no error, wait for `TweetRegion.tick` seconds plus
            // an extra second for every empty slot in the last call.
            None => self.tick + 100 - event.tweets_count,
        };

        self.fire_in(delay);
    }

    pub fn fire_in(&mut self, delay: u64) {
        // Create a payload.
        let payload = OutputEvent {
            delay: delay,
            region_id: self.id,
            params: self.params.clone(),
            since_id: self.since_id,
            topic: self.topic.clone()
        };

        // Stream the payload down the channel.
        match self.channel {
            Some(ref mut channel) => channel.send(payload).unwrap(),
            None => println!("TODO: Propagate error."),
        };
    }
}

pub fn route(mut regions: Vec<TweetRegion>) {
    // Blocks thread and polls messages from SQS.
    for event in sqs::listen() {
        // Finds the region via the identifier.
        let mut region = regions.iter_mut()
            .find(|region| region.id == event.region_id);

        match region {
            // Let the region handle the event.
            Some(ref mut region) => region.handle_event(event),
            // TODO: Refactor SNS.
            None => sns::notify(sns::Topic::UnknownRegion),
        };
    }
}

pub fn fire_all(regions: &mut Vec<TweetRegion>) {
    for region in regions.iter_mut() {
        region.fire_in(0);
    }
}
