use std::env;
use std::thread;
use std::sync::mpsc;
use event::OutputEvent;
use rusoto_core::Region;
use std::time::Duration;
use region::TweetRegion;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use rusoto_sns::{Sns, SnsClient, PublishInput, MessageAttributeValue};

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
    thread::sleep(Duration::from_millis(event.delay.into()));
    
    let client: SnsClient = SnsClient::simple(Region::EuWest1);
    
    // TODO: Handle this.
    let topic = env::var("SNS_OUTPUT_TOPIC").unwrap();

    let mut message: PublishInput = Default::default();
    {
        let mut attributes: HashMap<String, MessageAttributeValue> = HashMap::new();

        let mut attr: MessageAttributeValue = Default::default();
        attr.string_value = Some(event.since_id.to_string());
        attributes.insert("since_id".to_string(), attr);

        let mut attr: MessageAttributeValue = Default::default();
        attr.string_value = Some(event.region_id.to_string());
        attributes.insert("region_id".to_string(), attr);

        message.topic_arn = Some(topic);
        message.message_structure = Some("json".to_string());
        message.message = event.params;
        message.message_attributes = Some(attributes);
    }

    match client.publish(&message).sync() {
        Ok(res) => println!("SNS Message successfully added with id {}", res.message_id.unwrap_or("None".to_string())),
        Err(_e) => println!("Couldn't publish the message. TODO: SNS notify admin."),
    };
}
