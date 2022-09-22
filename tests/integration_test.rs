use granulator::{Granulator, MAX_GRAINS};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, Builder};
use std::time::Duration;

const BUFFER_LENGTH: usize = 64;

#[test]
#[ignore]
fn multi_threading() {
    // setup the variables
    let fs = 48000.0;
    let mut granulator = Granulator::new(fs as usize);
    let mut mock_audio_buffer = vec![];
    for _ in 0..10_000 {
        mock_audio_buffer.push(1.0);
    }

    let audio_callback_interval = ((1_000_000.0 * BUFFER_LENGTH as f32) / fs) as u64; //ns
    let schedule_timer_interval = 20; //ms

    const UPDATE_COUNT: u64 = 100;
    let test_duration = (UPDATE_COUNT * schedule_timer_interval) as usize; //ms
    let audio_callback_repition = (test_duration * 200) / audio_callback_interval as usize;

    let audio_callback_duration = Duration::from_nanos(audio_callback_interval);
    let schedule_timer_duration = Duration::from_millis(schedule_timer_interval);

    // setup granulator
    granulator.set_audio_buffer(mock_audio_buffer.as_slice());
    granulator.set_active_grains(MAX_GRAINS);
    granulator.set_grain_size(10.0);
    granulator.set_master_volume(1.0);

    // create reference counters for two threads
    let audio_ptr = Arc::new(Mutex::new(granulator));
    let scheduler_ptr = audio_ptr.clone();

    let audio_callback = Builder::new()
        .name("Audio Callback".to_string())
        .spawn(move || {
            for _ in 0..audio_callback_repition {
                for _ in 0..BUFFER_LENGTH {
                    // lock
                    {
                        let mut audio = audio_ptr.lock().unwrap();
                        let _sample = audio.get_next_sample();
                    }
                }
                sleep(audio_callback_duration);
            }
        });

    let scheduler_thread = Builder::new().name("Scheduler".to_string()).spawn(move || {
        for _ in 0..UPDATE_COUNT {
            // lock
            {
                let mut scheduler = scheduler_ptr.lock().unwrap();
                scheduler.update_scheduler(schedule_timer_duration);
            }

            sleep(schedule_timer_duration);
        }
    });

    audio_callback
        .unwrap()
        .join()
        .expect("Audio callback didn't finish!");
    scheduler_thread
        .unwrap()
        .join()
        .expect("Scheduler didn't finish!");
}
