//! Sweep generators 1 and 2.
//!
//! Both are classic integrator + Schmitt-trigger oscillators: Gen 1 is
//! Z12A/Z12B/Z13A/Z13B with a CR2–CR5 diode bridge steering the integrator
//! current so its rate can be voltage controlled (RATE knob or pedal);
//! Gen 2 is Z14A/Z14B with the RATE pot dividing the comparator output into
//! the integrator.
//!
//! What the panel calls "sine" is the integrator's triangle, taken straight
//! from Z13B through R75 (Gen 1) or Z14B (Gen 2). The rounded, sine-like
//! motion people hear comes later, from the LED driver curve and the
//! photocells. "Square" is the comparator output smoothed by an RC network
//! (R76/C36 for Gen 1, R92/C35 for Gen 2), so it jumps between the sweep
//! extremes with a short exponential glide instead of clicking.
//!
//! Outputs are in volts, centred on 0 V.

use crate::circuit::*;
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
    #[name = "Man"]
    Manual,
    #[name = "Ped"]
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
    /// Number of quarter-note beats per sweep cycle.
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

/// Which generator circuit this is (they differ in their square networks).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GenCircuit {
    Gen1,
    Gen2,
}

#[derive(Clone, Copy)]
pub struct SweepGenerator {
    phase: f32,
    square_rc: f32,
    sample_rate: f32,
    square_peak: f32,
    square_tau: f32,
    square_coeff: f32,
}

impl SweepGenerator {
    pub fn new(sample_rate: f32, circuit: GenCircuit) -> Self {
        let (square_peak, square_tau) = match circuit {
            GenCircuit::Gen1 => (GEN1_SQUARE_PEAK, GEN1_SQUARE_TAU),
            GenCircuit::Gen2 => (GEN2_SQUARE_PEAK, GEN2_SQUARE_TAU),
        };
        let mut g = Self {
            phase: 0.0,
            square_rc: 0.0,
            sample_rate,
            square_peak,
            square_tau,
            square_coeff: 0.0,
        };
        g.set_sample_rate(sample_rate);
        g
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.square_coeff = 1.0 - (-1.0 / (sample_rate * self.square_tau)).exp();
    }

    pub fn reset(&mut self) {
        // Start at the bottom of the sweep, the way the integrator comes up
        // after power-on.
        self.phase = 0.0;
        self.square_rc = -self.square_peak;
    }

    /// Advance one sample and return the generator output in volts.
    #[inline]
    pub fn tick(&mut self, freq_hz: f32, shape: LfoShape) -> f32 {
        self.phase += freq_hz / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
        }

        // Triangle: −1 at phase 0, +1 at phase 0.5.
        let tri = 1.0 - 4.0 * (self.phase - 0.5).abs();
        // Comparator output is high while the integrator ramps up.
        let comparator = if self.phase < 0.5 { 1.0 } else { -1.0 };
        self.square_rc += (comparator * self.square_peak - self.square_rc) * self.square_coeff;

        match shape {
            LfoShape::Sine => tri * GEN_TRI_PEAK,
            LfoShape::Square => self.square_rc,
        }
    }
}
