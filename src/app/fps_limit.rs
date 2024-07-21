use super::config::Config;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub struct FpsLimiter {
    frame_timer: Instant,
    frametime_smoothed: f64,
}

impl Default for FpsLimiter {
    fn default() -> Self {
        Self {
            frame_timer: Instant::now(),
            frametime_smoothed: 0.,
        }
    }
}

impl FpsLimiter {
    pub fn fps(&self) -> f64 {
        1. / self.frametime_smoothed
    }

    pub fn sleep(&mut self) {
        let before_wait = self.frame_timer.elapsed();

        let target_frametime = Duration::from_secs_f64(1. / Config::get().max_fps);
        if target_frametime > before_wait {
            sleep(target_frametime - before_wait);
        }

        let after_wait = self.frame_timer.elapsed();
        let frametime = after_wait.as_secs_f64();
        self.frametime_smoothed += (frametime - self.frametime_smoothed) * 0.1;

        self.frame_timer = Instant::now();
    }
}
