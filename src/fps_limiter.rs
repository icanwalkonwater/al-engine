use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct FpsLimiter {
    last_frame_time: Instant,
    target_frame_duration: Duration,
    target_frame_duration_micros: u128,
    delta_frame: u32,
}

impl FpsLimiter {
    pub fn new(target_fps: f32) -> Self {
        let target_frame_duration = Duration::from_secs_f32(1. / target_fps);

        Self {
            last_frame_time: Instant::now(),
            target_frame_duration,
            target_frame_duration_micros: target_frame_duration.as_micros(),
            delta_frame: 0,
        }
    }

    pub fn tick(&mut self) {
        let real_frame_duration = self.last_frame_time.elapsed();

        // Don't sleep for anything under 1ms
        if real_frame_duration.as_micros() < self.target_frame_duration_micros + 1000 {
            sleep(self.target_frame_duration - real_frame_duration);
        }

        // We can't reuse the previous values because we may have slept a bit
        self.delta_frame = self.last_frame_time.elapsed().subsec_micros();
        self.last_frame_time = Instant::now();
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_frame as f32 / 1_000_000.
    }
}
