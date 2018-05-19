use event::InputEvent;
use event::OutputEvent;
use std::sync::mpsc::Sender;

pub struct Region {
    tick: u8,
    pub id: u8,
    pub since_id: u64,
    pub params: String,
    pub channel: Option<Sender<OutputEvent>>,
}

impl Region {    
    pub fn new(id: u8, params: String) -> Region {
        Region {
            id: id,
            tick: 1,
            since_id: 0,
            channel: None,
            params: params,
        }
    }

    pub fn tick(&mut self, seconds: u8) -> &mut Region {
        self.tick = seconds;
        self
    }

    pub fn channel(&mut self, tx: Sender<OutputEvent>) -> &mut Region {
        self.channel = Some(tx);
        self
    }

    pub fn since_id(&mut self, since_id: u64) -> &mut Region {
        self.since_id = since_id;
        self
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        if event.since_id > self.since_id {
            self.since_id = event.since_id.clone();
        }

        let delay: u8 = match event.error {
            Some(wait_before_restart) => self.tick + wait_before_restart, 
            None => self.tick + 100 - event.tweets_count
        };

        let payload = OutputEvent {
            delay: delay,
            region_id: self.id,
            params: self.params.clone(),
            since_id: self.since_id,
        };

        // TODO: Propagate error.
        match self.channel {
            Some(ref mut channel) => channel.send(payload).unwrap(),
            None => println!("TODO: Propagate error."),
        }
    }
}
