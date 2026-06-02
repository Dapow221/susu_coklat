use shared::events::EngineEvent;

#[derive(Debug, Default)]
pub struct EngineState {
    pub paused: bool,
    pub recent_events: Vec<EngineEvent>,
}

impl EngineState {
    pub fn push_event(&mut self, event: EngineEvent) {
        self.recent_events.push(event);
        if self.recent_events.len() > 100 {
            self.recent_events.remove(0);
        }
    }
}
