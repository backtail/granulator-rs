/// Defines all configurable parameters
pub enum GranulatorParameter {
    MasterVolume,
    ActiveGrains,
    Offset,
    GrainSize,
    Pitch,
    Delay,
    Velocity,
    OffsetSpread,
    GrainSizeSpread,
    PitchSpread,
    VelocitySpread,
    DelaySpread,
    WindowParam,
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

    // window function parameters
    pub window_function: u8,
    pub window_param: f32,

    // musical pitch
    pub scale: u8,
    pub mode: u8,
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

            window_function: 0,
            window_param: 0.0,

            scale: 0,
            mode: 0,
        }
    }
}
