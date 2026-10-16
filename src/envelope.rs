//! C-100 Optical Foot Pedal & Dynamic Envelope Follower.
//!
//! Models both the Musitronics C-100 expression controller and the envelope-follower
//! sweep mode, enabling dynamic auto-wah / auto-phase sweeps from instrument dynamics.

use truce::prelude::ParamEnum;

#[derive(ParamEnum)]
pub enum PedalMode {
    #[name = "Manual"]
    Manual,
    #[name = "Envelope"]
    Envelope,
}

#[derive(Clone, Copy)]
pub struct PedalFollower {
    env_state: f32,
    sample_rate: f32,
    coeff_attack: f32,
    coeff_release: f32,
}

impl PedalFollower {
    pub fn new(sample_rate: f32) -> Self {
        let mut pf = Self {
            env_state: 0.0,
            sample_rate,
            coeff_attack: 0.0,
            coeff_release: 0.0,
        };
        pf.recalc_coefficients();
        pf
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.recalc_coefficients();
    }

    pub fn reset(&mut self) {
        self.env_state = 0.0;
    }

    fn recalc_coefficients(&mut self) {
        let tau_att = 0.008; // 8 ms attack
        let tau_rel = 0.120; // 120 ms release
        self.coeff_attack = 1.0 - (-1.0 / (self.sample_rate * tau_att)).exp();
        self.coeff_release = 1.0 - (-1.0 / (self.sample_rate * tau_rel)).exp();
    }

    /// Processes an input audio sample and returns the current pedal CV value [0.0, 1.0].
    #[inline(always)]
    pub fn process(
        &mut self,
        input_sample: f32,
        mode: PedalMode,
        manual_val: f32,
        sensitivity: f32,
    ) -> f32 {
        match mode {
            PedalMode::Manual => manual_val.clamp(0.0, 1.0),
            PedalMode::Envelope => {
                let rect = input_sample.abs() * sensitivity;
                if rect > self.env_state {
                    self.env_state += (rect - self.env_state) * self.coeff_attack;
                } else {
                    self.env_state += (rect - self.env_state) * self.coeff_release;
                }
                self.env_state.clamp(0.0, 1.0)
            }
        }
    }

    pub fn envelope_value(&self) -> f32 {
        self.env_state.clamp(0.0, 1.0)
    }
}
