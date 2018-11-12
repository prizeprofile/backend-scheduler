//! ResourceRegion struct represents a parameter collection for resource search.

use sqs;
use sns;
use event::InputEvent;
use event::OutputEvent;
use std::sync::mpsc::Sender;

/// A single region struct.
pub struct ResourceRegion {
    /// Private integer representing the minumum delay between API calls in seconds.
    tick: u64,

    /// An identifier.
    pub id: u64,

    /// BigInteger with the highest resource id in a region.
    pub since_id: u64,

    /// Bool that blocks events with max_id lower or equal to since_id if true. 
    pub compare_since_id: bool,

    /// JSON string of parameters that define a region.
    pub params: String,

    /// SNS Topic that region should be pushed to.
    pub topic: String,

    /// A `std::sync::mpsc::Sender` which streams `event::OutputEvent` to
    /// `sqs::stream` and pushes them to an AWS SQS.
    pub channel: Option<Sender<OutputEvent>>,
}

impl ResourceRegion {
    /// Builds a new default `ResourceRegion`.
    pub fn new(id: u64, topic: String, params: String) -> ResourceRegion {
        ResourceRegion {
            id: id,
            tick: 1,
            since_id: 0,
            topic: topic,
            channel: None,
            params: params,
            compare_since_id: true
        }
    }

    /// Setter for `ResourceRegion.tick`.
    pub fn tick(&mut self, seconds: u64) -> &mut ResourceRegion {
        self.tick = seconds;
        self
    }

    /// Setter for `ResourceRegion.channel`.
    pub fn channel(&mut self, tx: Sender<OutputEvent>) -> &mut ResourceRegion {
        self.channel = Some(tx);
        self
    }

    /// Setter for `ResourceRegion.since_id`.
    pub fn since_id(&mut self, since_id: u64) -> &mut ResourceRegion {
        self.since_id = since_id;
        self
    }

    /// Setter for `ResourceRegion.compare_since_id`.
    pub fn compare_since_id(&mut self, compare_since_id: bool) -> &mut ResourceRegion {
        self.compare_since_id = compare_since_id;
        self
    }

    /// Handles an incoming SQS message and generates an outcoming one.
    pub fn handle_event(&mut self, event: InputEvent) {
        // Only update since_id if max_id of incoming event is higher.
        if self.compare_since_id && event.max_id <= self.since_id {
            println!("[Discarted] region: {}, since_id: {}, max_id: {}", self.id, self.since_id, event.max_id);
            return
        }

        self.since_id = event.max_id.clone();

        // Decides how many seconds should the scheduler wait before rescheduling.
        let delay: u64 = match event.error {
            // If there is an error, it will have an information about how long
            // should we wait.
            Some(wait_before_restart) => self.tick + wait_before_restart,

            // If there is no error, wait for `ResourceRegion.tick` seconds plus
            // an extra second for every empty slot in the last call.
            None => self.tick + (100 - event.resources_count) * 1000,
        };

        self.fire_in(delay);
    }

    pub fn fire_in(&mut self, delay: u64) {
        println!("[ Fired ] region: {}, since_id: {}, delay: {}", self.id, self.since_id, delay.clone());

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

pub fn route(mut regions: Vec<ResourceRegion>) {
    // Blocks thread and polls messages from SQS.
    for event in sqs::listen() {
        // Finds the region via the identifier.
        let mut region = regions.iter_mut()
            .find(|region| region.id == event.region_id);
        
        println!("[Received] region: {}, max_id: {}, resources: {}", event.region_id, event.max_id, event.resources_count);

        match region {
            // Let the region handle the event.
            Some(ref mut region) => region.handle_event(event),
            // TODO: Refactor SNS.
            None => sns::notify(sns::Topic::UnknownRegion),
        };
    }
}

pub fn fire_all(regions: &mut Vec<ResourceRegion>) {
    for region in regions.iter_mut() {
        region.fire_in(0);
    }
}
