#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEvent {
    pub kind: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EventLog {
    pub events: Vec<LogEvent>,
}

impl EventLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn record(&mut self, kind: &str, message: &str) {
        self.events.push(LogEvent {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }

    pub fn contains_kind(&self, kind: &str) -> bool {
        self.events.iter().any(|e| e.kind == kind)
    }

    pub fn messages(&self) -> Vec<String> {
        self.events.iter().map(|e| e.message.clone()).collect()
    }
}
