//! C-100 control pedal and envelope follower.
//!
//! The C-100 is optical too, but it uses VTT9313 phototransistors behind a
//! shutter (with a servo loop holding its LED steady), so it responds
//! instantly; the only lag modelled is the pedal's own travel smoothing.
//! The envelope mode is a modern addition that "plays" the pedal from the
//! input level, in the spirit of Mu-tron's envelope-controlled filters.

use truce::prelude::ParamEnum;

#[derive(ParamEnum)]
pub enum PedalMode {
    #[name = "Manual"]
    Manual,
    #[name = "Envelope"]
    Envelope,
}

/// Pedal CV span at the sweep switches: heel to toe covers the same ±5 V
/// as a full-depth sweep generator.
pub const PEDAL_CV_SPAN: f32 = 10.0;

#[derive(Clone, Copy)]
pub struct PedalFollower {
    env: f32,
    position: f32,
    coeff_attack: f32,
    coeff_release: f32,
    coeff_travel: f32,
}

impl PedalFollower {
    pub fn new(sample_rate: f32) -> Self {
        let mut pf = Self {
            env: 0.0,
            position: 0.5,
            coeff_attack: 0.0,
            coeff_release: 0.0,
            coeff_travel: 0.0,
        };
        pf.set_sample_rate(sample_rate);
        pf
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let c = |tau: f32| 1.0 - (-1.0 / (sample_rate * tau)).exp();
        self.coeff_attack = c(0.010);
        self.coeff_release = c(0.180);
        self.coeff_travel = c(0.004);
    }

    pub fn reset(&mut self, position: f32) {
        self.env = 0.0;
        self.position = position.clamp(0.0, 1.0);
    }

    /// Returns the pedal position (0 = heel, 1 = toe).
    #[inline]
    pub fn process(&mut self, input: f32, mode: PedalMode, manual: f32, sensitivity: f32) -> f32 {
        let target = match mode {
            PedalMode::Manual => manual,
            PedalMode::Envelope => {
                let rect = input.abs() * sensitivity;
                let coeff = if rect > self.env {
                    self.coeff_attack
                } else {
                    self.coeff_release
                };
                self.env += (rect - self.env) * coeff;
                // Gentle compression so the pedal travel feels like a foot.
                1.0 - (-2.5 * self.env).exp()
            }
        };
        self.position += (target.clamp(0.0, 1.0) - self.position) * self.coeff_travel;
        self.position
    }

    /// Pedal position as a control voltage, centred on 0 V.
    #[inline]
    pub fn cv(position: f32) -> f32 {
        (position - 0.5) * PEDAL_CV_SPAN
    }
}
