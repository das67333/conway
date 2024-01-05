use macroquad::prelude::*;

pub struct InteractionManager {
    frame_x: f64,
    frame_y: f64,
    zoom: f64,
    zoom_step: f64,
    mouse_pos_prev: Option<(f64, f64)>,
}

impl InteractionManager {
    pub fn new(zoom_step: f64) -> Self {
        Self {
            frame_x: 0.,
            frame_y: 0.,
            zoom: 1.,
            zoom_step,
            mouse_pos_prev: None,
        }
    }

    pub fn update(&mut self) {
        let zoom_change = self.zoom_step.powf(mouse_wheel().1 as f64);

        let (x, y) = mouse_position();
        let (x, y) = (x as f64, y as f64);
        // capture
        if is_mouse_button_down(MouseButton::Left) {
            if let Some((x_prev, y_prev)) = self.mouse_pos_prev {
                self.frame_x += x - x_prev;
                self.frame_y += y - y_prev;
            }
        }
        // zoom
        self.frame_x = self.frame_x * zoom_change + x * (1. - zoom_change);
        self.frame_y = self.frame_y * zoom_change + y * (1. - zoom_change);
        self.zoom *= zoom_change;

        self.mouse_pos_prev = Some((x, y));
    }

    pub fn get_x_y_zoom(&self) -> (f32, f32, f32) {
        (self.frame_x as f32, self.frame_y as f32, self.zoom as f32)
    }
}
