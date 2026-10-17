//! First-order op-amp all-pass stage of the Bi-Phase ladder (Z1A ... Z6B).
//!
//! Each stage has equal 4.7 kΩ input/feedback resistors on the inverting
//! input and a series capacitor into the non-inverting input with the
//! photocell (‖ 220 kΩ) to ground:
//!
//! H(s) = (sRC − 1) / (sRC + 1)
//!
//! i.e. −180° at DC, −90° at fc = 1/(2πRC), 0° at high frequencies. It equals
//! x − 2·LP(x), so it is built on a topology-preserving one-pole low-pass,
//! which keeps the state continuous while R sweeps over three decades and
//! lets the feedback loop be solved without a unit delay.

#[derive(Clone, Copy, Default)]
pub struct AllpassStage {
    s: f32,
}

impl AllpassStage {
    #[inline]
    pub fn reset(&mut self) {
        self.s = 0.0;
    }

    /// Instantaneous response y = a·x + b for the current state, where
    /// `big_g` = g / (1 + g).
    #[inline]
    pub fn response(&self, big_g: f32) -> (f32, f32) {
        (1.0 - 2.0 * big_g, -2.0 * (1.0 - big_g) * self.s)
    }

    #[inline]
    pub fn process(&mut self, x: f32, big_g: f32) -> f32 {
        let v = (x - self.s) * big_g;
        let lp = v + self.s;
        self.s = lp + v;
        x - 2.0 * lp
    }
}
