//! Scheduler
//! Schedules API calls to Twitter that are picked up from AWS SNS.
//! Also listens to another SQS that informs about a result of a call
//! and then reschedules the call based on that result.

extern crate rand;

mod region;
mod event;
mod sqs;
mod boot;
mod sns;

use region::Region;

fn main() {
    // Loads all available regions.
    let mut regions: Vec<Region> = boot::get_regions();

    // TODO: Consider putting following into an infinit loop.

    // Sets up a comm channel where the regions stream their output messages.
    sns::stream(&mut regions);

    // Routes the SQS events to a relevant region.
    for event in sqs::listen() {
        // Finds the region via the identifier.
        let mut region = regions.iter_mut()
            .find(|region| region.id == event.region_id);
            
        match region {
            // Let's the region handle the event.
            Some(ref mut region) => region.handle_event(event),
            // TODO: Give more detail about the region id.
            None => sns::notify(sns::Topic::UnknownRegion),
        };
    }

    sns::notify(sns::Topic::SqsConnDropped);
}

