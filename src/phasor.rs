//! Musitronics 6-Stage Optical Phasor Channel (Phasor A / Phasor B).
//!
//! Emulates:
//! 1. Allpass filter cascade (6 stages default, selectable 2..12 poles)
//! 2. Inverting summing amplifier feedback loop with op-amp rail soft saturation
//! 3. CdS optocoupler module driving per-stage photoresistors
//! 4. Accurate wet/dry mixing and deep phase cancellation nulls

use crate::allpass::{calc_allpass_coeff, AllpassStage};
use crate::optocoupler::Optocoupler;
use truce::prelude::ParamEnum;

#[derive(ParamEnum)]
pub enum PolesCount {
    #[name = "6 Poles (Classic)"]
    Poles6,
    #[name = "2 Poles"]
    Poles2,
    #[name = "4 Poles"]
    Poles4,
    #[name = "8 Poles"]
    Poles8,
    #[name = "10 Poles"]
    Poles10,
    #[name = "12 Poles (Series)"]
    Poles12,
}

impl PolesCount {
    pub fn num_stages(self) -> usize {
        match self {
            PolesCount::Poles6 => 6,
            PolesCount::Poles2 => 2,
            PolesCount::Poles4 => 4,
            PolesCount::Poles8 => 8,
            PolesCount::Poles10 => 10,
            PolesCount::Poles12 => 12,
        }
    }
}

#[derive(ParamEnum)]
pub enum PhasorASweepSource {
    #[name = "Gen 1"]
    Gen1,
    #[name = "Pedal"]
    Pedal,
}

#[derive(ParamEnum)]
pub enum PhasorBSweepSource {
    #[name = "Gen 1"]
    Gen1,
    #[name = "Gen 2"]
    Gen2,
    #[name = "Pedal"]
    Pedal,
}

pub struct PhasorChannel {
    pub optocoupler: Optocoupler,
    stages: [AllpassStage; 12],
    feedback_state: f32,
    sample_rate: f32,
}

impl PhasorChannel {
    pub fn new(sample_rate: f32, is_phasor_b: bool) -> Self {
        Self {
            optocoupler: Optocoupler::new(sample_rate, is_phasor_b),
            stages: [AllpassStage::new(); 12],
            feedback_state: 0.0,
            sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.optocoupler.set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.optocoupler.reset();
        for st in &mut self.stages {
            st.reset();
        }
        self.feedback_state = 0.0;
    }

    /// Processes one sample through this phaser channel.
    /// Returns (processed_sample, wet_only_sample).
    #[inline(always)]
    pub fn process_sample(
        &mut self,
        input: f32,
        sweep_cv: f32,
        depth: f32,       // 0.0 to 1.0 (potentiometer range 0 to 10)
        feedback: f32,    // 0.0 to 1.0 (potentiometer range 0 to 10)
        poles: PolesCount,
        bypass: bool,
    ) -> (f32, f32) {
        if bypass {
            return (input, input);
        }

        // Depth scales the optical drive swing around a nominal center point
        let center = 0.35;
        let drive = (center + (sweep_cv - center) * depth).clamp(0.0, 1.0);

        // Advance CdS optical dynamics
        self.optocoupler.tick(drive);

        // Feedback loop with analog op-amp rail saturation
        let fb_gain = feedback.clamp(0.0, 1.0) * 0.94;
        let fb_signal = self.feedback_state * fb_gain;

        // Sum input with inverted feedback per schematic (Z8B inverting summing node)
        let in_sum = input - fb_signal;

        // Cascade through allpass stages
        let num_stages = poles.num_stages();
        let mut y = in_sum;

        for i in 0..num_stages {
            let fc = self.optocoupler.stage_cutoff_hz(i);
            let a = calc_allpass_coeff(fc, self.sample_rate);
            y = self.stages[i].process(y, a);
        }

        // Op-amp feedback non-linear soft saturation (Z9B/Z8B rail limits)
        let fb_saturated = (y * 1.2).tanh();
        self.feedback_state = fb_saturated;

        // Summing node (Z9A / Z11A)
        let wet_out = y;
        let mixed = 0.5 * input + 0.5 * wet_out;

        (mixed, wet_out)
    }
}
