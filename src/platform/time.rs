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

    pub fn reset_after_pause(&mut self) {
        self.previous = std::time::Instant::now();
        self.accumulator = 0.0;
    }

    pub fn alpha(&self) -> f32 {
        (self.accumulator / self.dt).clamp(0.0, 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::FixedTimestep;

    fn step(sim_hz: f64, accumulator: f64) -> FixedTimestep {
        // Build a timestep and inject an explicit accumulator so the pure
        // step/alpha/discard logic can be tested without real elapsed time.
        let mut ts = FixedTimestep::new(sim_hz);
        ts.accumulator = accumulator;
        ts
    }

    #[test]
    #[should_panic(expected = "finite and positive")]
    fn new_panics_on_zero_sim_hz() {
        FixedTimestep::new(0.0);
    }

    #[test]
    #[should_panic(expected = "finite and positive")]
    fn new_panics_on_negative_sim_hz() {
        FixedTimestep::new(-120.0);
    }

    #[test]
    #[should_panic(expected = "finite and positive")]
    fn new_panics_on_non_finite_sim_hz() {
        FixedTimestep::new(f64::NAN);
    }

    #[test]
    fn reset_after_pause_clears_accumulator() {
        let mut ts = step(120.0, 5.0);
        assert!(ts.step_ready());
        ts.reset_after_pause();
        assert_eq!(ts.accumulator, 0.0);
        assert!(!ts.step_ready());
    }

    #[test]
    fn accumulator_produces_expected_number_of_steps() {
        let dt = 1.0 / 120.0;
        let mut ts = step(120.0, dt * 3.5);

        let mut steps = 0;
        while ts.step_ready() {
            let consumed = ts.consume_step();
            assert!((consumed - dt as f32).abs() < 1e-6);
            steps += 1;
        }

        assert_eq!(steps, 3);
        // Just under one step of lag remains.
        assert!(ts.accumulator < dt);
        assert!((ts.accumulator - dt * 0.5).abs() < 1e-9);
    }

    #[test]
    fn discard_lag_leaves_less_than_one_step() {
        let dt = 1.0 / 120.0;
        let mut ts = step(120.0, dt * 5.5);
        ts.discard_lag();
        assert!(ts.accumulator < dt);
        assert!((ts.accumulator - dt * 0.5).abs() < 1e-9);
    }

    #[test]
    fn alpha_clamps_to_unit_range() {
        let dt = 1.0 / 120.0;
        assert_eq!(step(120.0, 0.0).alpha(), 0.0);
        assert!((step(120.0, dt * 0.5).alpha() - 0.5).abs() < 1e-6);
        // More than a full step of accumulated time still reports alpha 1.0.
        assert_eq!(step(120.0, dt * 2.0).alpha(), 1.0);
    }
}
