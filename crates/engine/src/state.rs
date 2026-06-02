use shared::events::EngineEvent;

#[derive(Debug, Default)]
pub struct EngineState {
    pub paused: bool,
    pub dry_run: bool,
    pub recent_events: Vec<EngineEvent>,
    pub event_counter: u64,
}

impl EngineState {
    pub fn push_event(&mut self, mut event: EngineEvent) {
        event.seq = self.event_counter;
        self.event_counter += 1;
        self.recent_events.push(event);
        if self.recent_events.len() > 100 {
            self.recent_events.remove(0);
        }
    }

    pub fn events_since(&self, cursor: u64) -> Vec<EngineEvent> {
        if cursor >= self.event_counter {
            return Vec::new();
        }
        let idx = self.recent_events.partition_point(|e| e.seq <= cursor);
        self.recent_events[idx..].to_vec()
    }
}
