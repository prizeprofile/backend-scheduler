//! Scheduler
//! Schedules API calls to Twitter that are picked up from AWS SNS.
//! Also listens to another SQS that informs about a result of a call
//! and then reschedules the call based on that result.

extern crate rusoto_core;
extern crate serde_json;
extern crate rusoto_sns;
extern crate rusoto_s3;
extern crate futures;
extern crate dotenv;

mod sns;
mod sqs;
mod boot;
mod event;
mod region;

use dotenv::dotenv;

fn main() {
    // Loads .env config file.
    dotenv().ok();

    // Loads all available regions.
    match boot::get_regions() {
        Ok(mut regions) => {
            // Sets up a comm channel where the regions stream their output messages.
            sns::stream(&mut regions);

            region::fire_all(&mut regions);

            // Routes the SQS events to a relevant region.
            // TODO: Change the file structure.
            region::route(regions);

            sns::notify(sns::Topic::SqsConnDropped);
        },
        Err(error) => println!("TODO: Report in sns{}", error),
    };
}

