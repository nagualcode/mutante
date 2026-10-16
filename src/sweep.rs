//! Mu-Tron Bi-Phase Sweep Generators (LFOs).
//!
//! Features:
//! 1. Hardware-calibrated rate range: 0.1 Hz to 18.0 Hz.
//! 2. Diode-bridge shaped pseudo-sine and bipolar square waveforms.
//! 3. Host tempo synchronization support with musical subdivisions.
//! 4. Continuous phase accumulation and anti-pop phase preservation.

use truce::prelude::ParamEnum;

#[derive(ParamEnum)]
pub enum LfoShape {
    #[name = "Sine"]
    Sine,
    #[name = "Square"]
    Square,
}

#[derive(ParamEnum)]
pub enum RateControl {
    #[name = "Manual"]
    Manual,
    #[name = "Pedal"]
    Pedal,
}

#[derive(ParamEnum)]
pub enum SweepSync {
    #[name = "Norm"]
    Norm,
    #[name = "Rev"]
    Rev,
}

#[derive(ParamEnum)]
pub enum SyncRate {
    #[name = "4 Bars"]
    Bars4,
    #[name = "2 Bars"]
    Bars2,
    #[name = "1 Bar"]
    Bar1,
    #[name = "1/2"]
    Half,
    #[name = "1/2 D"]
    HalfDotted,
    #[name = "1/2 T"]
    HalfTriplet,
    #[name = "1/4"]
    Quarter,
    #[name = "1/4 D"]
    QuarterDotted,
    #[name = "1/4 T"]
    QuarterTriplet,
    #[name = "1/8"]
    Eighth,
    #[name = "1/8 D"]
    EighthDotted,
    #[name = "1/8 T"]
    EighthTriplet,
    #[name = "1/16"]
    Sixteenth,
}

impl SyncRate {
    /// Returns the number of quarter-note beats for this sync subdivision.
    pub fn beats(self) -> f32 {
        match self {
            SyncRate::Bars4 => 16.0,
            SyncRate::Bars2 => 8.0,
            SyncRate::Bar1 => 4.0,
            SyncRate::Half => 2.0,
            SyncRate::HalfDotted => 3.0,
            SyncRate::HalfTriplet => 4.0 / 3.0,
            SyncRate::Quarter => 1.0,
            SyncRate::QuarterDotted => 1.5,
            SyncRate::QuarterTriplet => 2.0 / 3.0,
            SyncRate::Eighth => 0.5,
            SyncRate::EighthDotted => 0.75,
            SyncRate::EighthTriplet => 1.0 / 3.0,
            SyncRate::Sixteenth => 0.25,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SweepGenerator {
    phase: f32,
    sample_rate: f32,
}

impl SweepGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Advances the LFO phase and returns the normalized sweep value [0.0, 1.0].
    /// `effective_freq` is in Hz.
    #[inline(always)]
    pub fn tick(&mut self, effective_freq: f32, shape: LfoShape) -> f32 {
        let phase_inc = effective_freq / self.sample_rate;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
        }

        match shape {
            LfoShape::Sine => {
                // Analog diode-bridge waveshaper emulation:
                // Base triangle wave in [-1.0, 1.0]
                let tri = if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                };

                // CR2-CR5 diode bridge soft-saturation curve
                let rounded = (tri * 1.5).tanh() * 1.15;
                // Normalize from [-1, +1] to [0.0, 1.0]
                (rounded * 0.5 + 0.5).clamp(0.0, 1.0)
            }
            LfoShape::Square => {
                // Bipolar square wave [0.0, 1.0]
                if self.phase < 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}
