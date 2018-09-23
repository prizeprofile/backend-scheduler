use std::env;
use std::thread;
use std::sync::mpsc;
use event::InputEvent;
use std::time::Duration;
use rusoto_core::Region;
use serde_json::{Value, from_str};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use rusoto_sqs::{Sqs, SqsClient, ReceiveMessageRequest, Message, DeleteMessageRequest};

/// Starts polling messages from AWS SQS and returns the receiver object.
pub fn listen() -> Receiver<InputEvent> {
    // Boots a new channel.
    let (tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();
    
    thread::spawn(move || {
        let client: SqsClient = SqsClient::simple(Region::EuWest1);
        
        // Polls for messages every second.
        loop {
            match poll_messages(&client) {
                Some(messages) => {
                    println!("Received: {}", messages.len());
                    // TODO: Handle error.
                    for message in messages.into_iter() {
                        tx.send(message).unwrap();
                    }
                },
                None => (),
            }

            thread::sleep(Duration::from_secs(1));
        }
    });

    // Returns the single receiver.
    rx
}

fn poll_messages(client: &SqsClient) -> Option<Vec<InputEvent>> {
    let queue_url = env::var("SQS_RESULT_QUEUE").unwrap();
    
    let mut req: ReceiveMessageRequest = Default::default();
    {
        req.queue_url = queue_url.clone();
        req.max_number_of_messages = Some(10);
    }

    let messages: Option<Vec<Message>> = match client.receive_message(&req).sync() {
        Ok(res) => res.messages,
        Err(e) => {
            println!("SQS polling error: {}", e);
            None
        },
    };

    let messages: Vec<InputEvent> = messages?.into_iter().filter_map(|message| {
        let mut entry: DeleteMessageRequest = Default::default();
        {
            entry.queue_url = queue_url.clone();
            entry.receipt_handle = message.receipt_handle?;
        }

        // TODO: Handle this.
        let json: Value = from_str(&message.body?).unwrap(); 
        let region_id: u64 = json["region_id"].as_u64()?;
        let max_id: u64 = json["max_id"].as_u64()?;
        let tweets_count: u64 = json["tweets_count"].as_u64()?;
        let error: Option<u64> = json["error_delay"].as_u64();
        let event: InputEvent = InputEvent {
            region_id: region_id,
            max_id: max_id,
            tweets_count: tweets_count,
            error: error
        };

        match client.delete_message(&entry).sync() {
            Ok(_) => Some(event),
            Err(e) => {
                println!("Couldnt delete message {}", e);
                None
            },
        }
    })
    .collect::<Vec<InputEvent>>();
    
    Some(messages)
}
