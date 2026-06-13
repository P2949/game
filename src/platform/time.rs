pub struct FixedTimestep {
    previous: std::time::Instant,
    accumulator: f64,
    dt: f64,
    max_frame_time: f64,
}

impl FixedTimestep {
    pub const MAX_STEPS_PER_FRAME: usize = 8;

    pub fn new(sim_hz: f64) -> Self {
        assert!(
            sim_hz.is_finite() && sim_hz > 0.0,
            "fixed timestep rate must be finite and positive"
        );

        Self {
            previous: std::time::Instant::now(),
            accumulator: 0.0,
            dt: 1.0 / sim_hz,
            max_frame_time: 0.25,
        }
    }

    pub fn begin_frame(&mut self) {
        let now = std::time::Instant::now();
        let mut frame_time = (now - self.previous).as_secs_f64();
        self.previous = now;

        frame_time = frame_time.min(self.max_frame_time);
        self.accumulator += frame_time;
    }

    pub fn step_ready(&self) -> bool {
        self.accumulator >= self.dt
    }

    pub fn consume_step(&mut self) -> f32 {
        self.accumulator -= self.dt;
        self.dt as f32
    }

    pub fn discard_lag(&mut self) {
        self.accumulator = self.accumulator.rem_euclid(self.dt);
    }

    pub fn alpha(&self) -> f32 {
        (self.accumulator / self.dt).clamp(0.0, 1.0) as f32
    }
}
