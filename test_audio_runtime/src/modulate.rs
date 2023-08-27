use crate::util::ease_cubic_in_out;

struct TransitionDef {
    // easing
    start_time: f32,
    end_time: f32,
    target: f32,
}

struct Transition {
    start_time: f32,
    end_time: f32,
    source: f32,
    target: f32,
}

pub struct Modulation {
    time: f32,
    param: String,
    value: f32,
    transition: Option<Transition>,
    pending_transitions: Vec<TransitionDef>,
}

impl Modulation {
    pub fn new(param: String, value: f32) -> Self {
        Self {
            time: 0.0,
            param,
            value,
            transition: None,
            pending_transitions: vec![],
        }
    }

    pub fn schedule_transition(&mut self, start_time: f32, duration: f32, target: f32) {
        self.pending_transitions.push(TransitionDef {
            start_time,
            end_time: start_time + duration,
            target,
        });

        self.pending_transitions
            .sort_by(|a, b| b.start_time.partial_cmp(&a.start_time).unwrap());
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time;

        // start a new transition
        if self
            .pending_transitions
            .last()
            .map(|t| t.start_time <= self.time)
            .unwrap_or(false)
        {
            let t = self.pending_transitions.pop().unwrap();
            self.transition = Some(Transition {
                start_time: t.start_time,
                end_time: t.end_time,
                source: self.value,
                target: t.target,
            });
        }

        // end a transition
        if self
            .transition
            .as_ref()
            .map(|t| t.end_time <= self.time)
            .unwrap_or(false)
        {
            self.transition = None;
        }

        // interpolate the potential current transition
        if let Some(t) = &self.transition {
            let x = (self.time - t.start_time) / (t.end_time - t.start_time);
            let y = ease_cubic_in_out(x);
            self.value = y * (t.target - t.source) + t.source;
        }
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn get_message(&mut self, time: f32) -> (String, f32) {
        self.set_time(time);
        (self.param.clone(), self.get_value())
    }
}
