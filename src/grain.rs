use super::source::Source;
use super::window_function::WindowFunction;

pub const DEFAULT_GRAIN: Grain = Grain {
    window: None,
    window_parameter: None,
    envelope_position: 0,
    envelope_value: 0.0,

    source: None,
    source_length: None,
    source_offset: None,
    relative_position: 0,
    source_position: 0,

    grain_size: None,
    finished: true,
    id: 0,
};

#[derive(Clone, Copy)]
pub struct Grain {
    // envelope variables
    pub window: Option<WindowFunction>,
    pub window_parameter: Option<f32>,
    pub envelope_position: u32, // in samples between 0..grain_size
    pub envelope_value: f32,    // between 0..1

    // source variables
    pub source: Option<Source>,
    pub source_length: Option<usize>, // in samples
    pub source_offset: Option<usize>, // in samples
    pub relative_position: usize,     // between 0..grain_size
    pub source_position: usize,       // between source_offset..source_length

    // grain variables
    pub grain_size: Option<f32>, // in samples
    pub finished: bool,

    // misc
    pub id: usize,
}

impl Grain {
    pub fn new() -> Self {
        Grain {
            window: None,
            window_parameter: None,
            envelope_position: 0,
            envelope_value: 0.0,

            source: None,
            source_length: None,
            source_offset: None,
            relative_position: 0,
            source_position: 0,

            grain_size: None,
            finished: true,

            id: 0,
        }
    }

    pub fn activate(&mut self) {
        self.finished = false; // start grain
    }

    pub fn is_finished(&mut self) -> bool {
        self.finished
    }

    pub fn reactivate(&mut self) {
        self.envelope_position = 0;
        self.envelope_value = 0.0;
        self.relative_position = 0;
        self.source_position = self.source_offset.unwrap();

        self.finished = false;
    }

    pub fn update_envelope(&mut self) {
        if !self.finished {
            let current_position = self.envelope_position as f32;
            self.envelope_value = self.window.as_ref().unwrap().get_envelope_value(
                current_position,
                self.grain_size.unwrap(),
                self.window_parameter,
            );
            if current_position < self.grain_size.unwrap() {
                self.envelope_position += 1;
            } else {
                self.finished = true;
                self.envelope_value = 0.0;
            }
        }
    }

    // TODO
    // many parameters may be calculated before creating the grain

    // if offset + size > length { doesn't fitclear -> find new position }
    pub fn update_source_sample(&mut self) {
        if !self.finished {
            self.relative_position += 4;
            self.source_position = self
                .source
                .as_ref()
                .unwrap()
                .get_source_sample(self.relative_position, self.source_offset.unwrap());
            if self.source_position
                > self.source_length.unwrap() - self.grain_size.unwrap() as usize
            {
                self.source_position = self.source_offset.unwrap();
                self.finished = true;
            }
        }
    }

    pub fn reset(&mut self) {
        self.window = None;
        self.window_parameter = None;
        self.envelope_position = 0;
        self.envelope_value = 0.0;

        self.source = None;
        self.source_length = None;
        self.source_offset = None;
        self.relative_position = 0;
        self.source_position = 0;

        self.grain_size = Some(0.0);
        self.finished = true;
    }
}
