use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub struct FpsLimiter {
    target_frametime: Duration,
    frame_timer: Instant,
    frametime_smoothed: f64,
}

impl Default for FpsLimiter {
    fn default() -> Self {
        Self {
            target_frametime: Duration::ZERO,
            frame_timer: Instant::now(),
            frametime_smoothed: 0.,
        }
    }
}

impl FpsLimiter {
    pub fn fps(&self) -> f64 {
        1. / self.frametime_smoothed
    }

    pub fn set_max_fps(&mut self, max_fps: f64) {
        self.target_frametime = Duration::from_secs_f64(1. / max_fps);
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
