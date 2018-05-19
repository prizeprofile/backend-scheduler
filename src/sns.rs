pub enum Priority {
    SEVERE,
    // STANDARD,
}

pub fn notify(message: String, _priority: Priority) {
    // TODO: Push to SNS.
    println!("{}", message);
}

