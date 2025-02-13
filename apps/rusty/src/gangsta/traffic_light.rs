use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrafficLightState {
    Green,
    Yellow,
    Red,
}

#[derive(Debug)]
pub struct TrafficLight {
    pub state: TrafficLightState,
    last_changed: Instant,
    cycle_duration: Duration, // Duration of each cycle
}

impl TrafficLight {
    pub fn new() -> Self {
        Self {
            state: TrafficLightState::Green,
            last_changed: Instant::now(),
            cycle_duration: Duration::from_secs(10), // Change every 10 seconds
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.last_changed.elapsed();
        match self.state {
            TrafficLightState::Green if elapsed >= self.cycle_duration => {
                self.state = TrafficLightState::Yellow;
                self.last_changed = Instant::now();
            }
            TrafficLightState::Yellow if elapsed >= Duration::from_secs(3) => {
                self.state = TrafficLightState::Red;
                self.last_changed = Instant::now();
            }
            TrafficLightState::Red if elapsed >= self.cycle_duration => {
                self.state = TrafficLightState::Green;
                self.last_changed = Instant::now();
            }
            _ => {}
        }
    }
}
