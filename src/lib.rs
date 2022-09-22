/*!
Granular synthesis with `#![no_std]` support!

This library implements a granular algorithm which can be easily placed on two
threads (i.e. audio callback and update).

# Limitations
Currently, only **mono** WAV files in f32le (32 bit floating point little endian) format
are being supported. It is planned to support a lot more PCM based formats and bit depths!

# Platform
Depending on which platform you are targeting, there are different implementation
styles.

##### Non-Embedded
When used with `std`, one can simply wrap the `Granulator` in an `Arc<Mutex<_>>`,
copy its reference counter, start two threads and lock it respectively.

This code snippet acts only as rough demonstration.

```no_run
use std::sync::{Arc, Mutex};
use std::time::Duration;

// only outputs sound if an audio buffer is provided
// which can be swapped out during playback as well
let granulator = granulator::Granulator::new(48_000); // provide a sample frequency

// Wrap it in a reference counter and a Mutex, then clone it
let audio_ref = Arc::new(Mutex::new(granulator));
let update_ref = audio_ref.clone();

// Audio callback (mono)
let buffer = | buffer: Vec<f32> | {
    // lock the granulator
    let mut gran = audio_ref.lock().unwrap();

    // calculate samples for the next buffer
    for mut sample in buffer {
        sample = gran.get_next_sample();
    }
};

// Update the granulator itself and its parameters
// Interval duration should be less than 20ms
let update = | interval_since_last: Duration | {
    // lock the granulator
    let mut gran = update_ref.lock().unwrap();

    gran.update_scheduler(interval_since_last);

    // set all other parameters of the algorithm
};
```

###### Embedded
For all embedded platforms with CAS (Compare and Swap) instruction, this library may be
used safely.
It follows the same principles as the example above, only that instead of a Mutex,
a `critical-section` is being used.

The use of the `rtic` (Real-Time Interrupt Concurrency) framework is highly recommended,
since it trivialises the implementation of the `critical-section`.

This code snippet acts only as rough demonstration.

```ignore
#[rtic::app(
    device = YOUR_DEVICE,
    peripherals = true,
)]
mod app {
    use granulator::Granulator;

    #[shared]
    struct Shared { granulator: Granulator }

    #[local]
    struct Local { buffer: &mut [f32; 64] }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // init system and buffer
        (
            Shared { granulator: Granulator::new(48_000) },
            Local { buffer },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    // Audio callback
    #[task(binds = DMA1_STR1, local = [buffer], shared = [granulator], priority = 8)]
    fn audio_handler(ctx: audio_handler::Context) {
        let mut buffer = *ctx.local.buffer;
        let mut granulator = *ctx.shared.granulator;

        for sample in buffer {
            let mut sample = 0.0;
            granulator.lock(|granulator| {
                sample = granulator.get_next_sample();
            });

            // push audio into stream
        }
    }

    #[task(binds = TIM2, shared = [granulator])]
    fn update_hanlder(ctx update_handler::Context) {
        let mut granulator = *ctx.shared.granulator;

        granulator.lock(|granulator| {
            granulator.update_schedular(YOUR_TIME_INTERVAL);

            // set all the parameters
        });
    }
}
```

*/

// configure as no_std as default since it is the default feature
#![cfg_attr(feature = "no_std", no_std)]
// #![warn(missing_docs)]

pub(crate) mod manager;

pub(crate) mod audio_tools;
pub(crate) mod grain;
pub(crate) mod grains_vector;
pub(crate) mod pointer_wrapper;
pub(crate) mod scheduler;
pub(crate) mod source;
pub(crate) mod window_function;

pub use crate::manager::Granulator;
pub use crate::manager::MAX_GRAINS;
