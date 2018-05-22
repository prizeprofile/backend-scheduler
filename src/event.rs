//! There are 2 event types that can happen in the app.
//! Events represent messages received from or sent to outer AWS architecture.

/// Incoming SQS message type. Is generated in the `sqs::listen` method and sent
/// across the channel to a receiver that routes it to relevant region.
pub struct InputEvent {
    /// Integer ID of region that the event belongs to.
    pub region_id: u64,

    /// BigInteger of the highest Tweet ID that was parsed in the region.
    pub max_id: u64,

    /// Integer in an interval <0, 100>.
    /// How many tweets were received in the last Twitter scrape.
    pub tweets_count: u64,

    /// If there was an error in scraping Twitter, this field would hold
    /// an integer indicating how many seconds should the scheduler wait
    /// before rescheduling the call.
    pub error: Option<u64>,
}

/// Outcoming SQS message type. Is generated in `region::TweetRegion::handle_event`
/// and sent across the channel to `sqs::stream` that pushes it to the AWS SQS.
pub struct OutputEvent {
    /// Integer of how many seconds should the scheduler wait
    /// before pushing the message.
    pub delay: u64,

    /// Integer that is used to backtrack the message to a region.
    pub region_id: u64,

    /// BigInteger with the minimum id a Tweet oughts to have.
    pub since_id: u64,

    /// JSON string of parameters that define a region.
    pub params: String,
}
