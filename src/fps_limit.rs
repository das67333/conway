use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub struct FpsLimiter {
    target_frametime: Duration,
    frame_timer: Instant,
    frametime_smoothed: f64,
}

impl FpsLimiter {
    pub fn new(max_fps: f64) -> Self {
        Self {
            target_frametime: Duration::from_secs_f64(1.0 / max_fps),
            frame_timer: Instant::now(),
            frametime_smoothed: 1.0 / max_fps,
        }
    }

    pub fn fps(&self) -> f64 {
        1. / self.frametime_smoothed
    }

    pub fn delay(&mut self) {
        let before_wait = self.frame_timer.elapsed();

        if self.target_frametime > before_wait {
            sleep(self.target_frametime - before_wait);
        }

        let after_wait = self.frame_timer.elapsed();
        let frametime = after_wait.as_secs_f64();
        self.frametime_smoothed += (frametime - self.frametime_smoothed) * 0.1;

        self.frame_timer = Instant::now();
    }
}
