pub struct InputEvent {
    pub region_id: u8,
    pub since_id: u64,
    pub tweets_count: u8,
    pub error: Option<u8>,
}

pub struct OutputEvent {
    pub delay: u8,
    pub region_id: u8,
    pub since_id: u64,
    pub params: String,
}

