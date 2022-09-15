// use assert2::*;
use granulator::Granulator;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, Builder};
use std::time::Duration;

#[test]
fn multi_threading() {
    let schedule_timer_interval = Duration::from_millis(10);
    let audio_callback_interval = Duration::from_nanos(1190);

    let audio_ptr = Arc::new(Mutex::new(Granulator::new()));
    let scheduler_ptr = audio_ptr.clone();

    let audio_callback = Builder::new()
        .name("Audio Callback".to_string())
        .spawn(move || {
            for _ in 0..100 {
                // lock
                {
                    let mut audio = audio_ptr.lock().unwrap();
                    audio.get_next_sample();
                }

                sleep(audio_callback_interval);
            }
        });

    let scheduler_thread = Builder::new().name("Scheduler".to_string()).spawn(move || {
        for _ in 0..10 {
            // lock
            {
                let mut scheduler = scheduler_ptr.lock().unwrap();
                scheduler.update_scheduler(schedule_timer_interval);
            }

            sleep(schedule_timer_interval);
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
