/// Defines all configurable parameters
pub enum GranulatorParameter {
    /**
    Software master volume. Since granular synthesis experiences high dynamic ranges, the user
    needs to have the possibility to change the output volume. It softclips if the gain is too high.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::MasterVolume, 0.0); // master_volume = 0.0
    g.set_parameter(GranulatorParameter::MasterVolume, 1.0); // master_volume = 1.0
    ```
    */
    MasterVolume,
    /**
    Number of active grains. Currently upper limited to a const `MAX_GRAINS`. Different API coming in the future.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    // MAX_GRAINS not configurable
    g.set_parameter(GranulatorParameter::ActiveGrains, 0.0); // grain_amount = 0
    g.set_parameter(GranulatorParameter::ActiveGrains, 1.0); // grain_amount = MAX_GRAINS
    ```
    */
    ActiveGrains,

    /**
    Offset a grain has relative to the start of the audio source.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Offset, 0.0); // offset = 0
    g.set_parameter(GranulatorParameter::Offset, 1.0); // offset = source_length
    ```
    */
    Offset,
    /**
    The grain's size. May be as big as the audio sources length. Currently lower bounded to 1ms. If a grain has a
    `source_length - offset < grain_size`, its size is gonna be reduced to `source_length - offset` to prevent
    out of bounds stepping.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::GrainSize, 0.0); // grain_size = 1ms
    g.set_parameter(GranulatorParameter::GrainSize, 1.0); // grain_size = source_length - offset
    ```
    */
    GrainSize,
    /**
    The grain's playback speed. For `pitch < 1`, the playback speed is gonna slower than original. Likewise, for
    `pitch > 1` the playback speed is gonna be faster than original.

    ### Example

    Internal exponentiation by power of 10.

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Pitch, 0.0); // pitch = 0.1
    g.set_parameter(GranulatorParameter::Pitch, 0.5); // pitch = 1.0
    g.set_parameter(GranulatorParameter::Pitch, 1.0); // pitch = 10.0
    ```
    */
    Pitch,
    /**
    The grain's delay. A grain can be fired with delay. Currently limited to 1s.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Delay, 0.0); // delay = 0s
    g.set_parameter(GranulatorParameter::Delay, 1.0); // delay = 1s
    ```
    */
    Delay,
    /**
    The grain's velocity. A per grain specific gain, reminiscent of MIDI velocity.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Velocity, 0.0); // grain_volume = 0.0
    g.set_parameter(GranulatorParameter::Velocity, 1.0); // grain_volume = 1.0
    ```
    */
    Velocity,

    /**
    The range in which a grain can randomly spawn with a different offset on top of the inital `offset` value.
    This is bipolar, so the random offset can be either smaller or greater than the global `offset` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::OffsetSpread, 0.0); // no random offset added
    g.set_parameter(GranulatorParameter::OffsetSpread, 1.0); // full random offset in range [0, source_length] bipolarly added
    ```
    */
    OffsetSpread,
    /**
    The range in which a grain can randomly spawn with a different grain size on top of the inital `grain_size` value.
    This is bipolar, so the random grain size can be either smaller or greater than the global `grain_size` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::GrainSizeSpread, 0.0); // no random grain size added
    g.set_parameter(GranulatorParameter::GrainSizeSpread, 1.0); // full random grain size in range [1ms, source_length - offset] bipolarly added
    ```
    */
    GrainSizeSpread,
    /**
    The range in which a grain can randomly spawn with a different pitch on top of the inital `pitch` value.
    This is bipolar, so the random pitch can be either smaller or greater than the global `pitch` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::PitchSpread, 0.0); // no random pitch added
    g.set_parameter(GranulatorParameter::PitchSpread, 1.0); // full random pitch in range [0.1, 10.0] bipolarly added
    ```
    */
    PitchSpread,
    /**
    The range in which a grain can randomly spawn with a different velocity on top of the inital `velocity` value.
    This is bipolar, so the random velocity can be either smaller or greater than the global `velocity` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::VelocitySpread, 0.0); // no random velocity added
    g.set_parameter(GranulatorParameter::VelocitySpread, 1.0); // full random velocity in range [0.0, 1.0] bipolarly added
    ```
    */
    VelocitySpread,
    /**
    The range in which a grain can randomly spawn with a different delay on top of the inital `delay` value.
    This is unipolar, so the random delay can only be greater than the global `delay` value.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::DelaySpread, 0.0); // no random delay added
    g.set_parameter(GranulatorParameter::DelaySpread, 1.0); // random delay in range [0s, 1s] added
    ```
    */
    DelaySpread,
}

#[derive(Debug, Clone, Copy)]
pub struct UserSettings {
    // parameters
    pub master_volume: f32,
    pub active_grains: f32,
    pub offset: f32,
    pub grain_size: f32,
    pub pitch: f32,
    pub delay: f32,
    pub velocity: f32,

    // spread parameters
    pub sp_offset: f32,
    pub sp_grain_size: f32,
    pub sp_pitch: f32,
    pub sp_delay: f32,
    pub sp_velocity: f32,
}

impl UserSettings {
    pub const fn new_empty() -> UserSettings {
        UserSettings {
            master_volume: 0.5,
            active_grains: 0.5,
            offset: 0.0,
            grain_size: 0.5,
            pitch: 0.5,
            delay: 0.0,
            velocity: 1.0,

            sp_offset: 0.0,
            sp_grain_size: 0.0,
            sp_pitch: 0.0,
            sp_delay: 0.0,
            sp_velocity: 0.0,
        }
    }
}
