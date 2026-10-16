//! Active 1st-Order All-Pass Filter Stage (Musitronics Bi-Phase topology).
//!
//! Continuous-time transfer function:
//! H(s) = (s*R*C - 1) / (s*R*C + 1)
//!
//! Discrete-time implementation using Bilinear Transform (TPT / TDF-II):
//! H(z) = (a - z^-1) / (1 - a*z^-1)
//! where g = tan(pi * fc / fs), a = (1 - g) / (1 + g).
//!
//! Magnitude is identically 1.0 across all frequencies.
//! Phase is -180 deg at DC, -90 deg at fc, and 0 deg at Nyquist.

#[derive(Clone, Copy, Default)]
pub struct AllpassStage {
    state: f32,
}

impl AllpassStage {
    pub fn new() -> Self {
        Self { state: 0.0 }
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    /// Processes one sample through the allpass filter.
    /// `a` is the precomputed coefficient (1 - g) / (1 + g).
    #[inline(always)]
    pub fn process(&mut self, x: f32, a: f32) -> f32 {
        let y = a * x + self.state;
        self.state = a * y - x;
        y
    }
}

/// Helper function to calculate the allpass coefficient `a` from frequency `fc` and sample rate `fs`.
#[inline(always)]
pub fn calc_allpass_coeff(fc: f32, sample_rate: f32) -> f32 {
    let norm_f = (fc / sample_rate).clamp(0.0001, 0.499);
    let g = (std::f32::consts::PI * norm_f).tan();
    (1.0 - g) / (1.0 + g)
}
