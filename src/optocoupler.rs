//! Cadmium Sulfide (CdS) Optocoupler / Vactrol Physics Model.
//!
//! Emulates the physical behavior of the Musitronics optical pods (PM3 and PM4),
//! each consisting of a central light source illuminating 6 matched photoresistors.
//!
//! Physics modeled:
//! 1. Non-linear photoconductive power-law: R(L) = R_light + (R_dark - R_light) / (1 + k*L)^gamma
//! 2. Asymmetric dual-time-constant kinetics (fast rise ~10ms, two-stage decay ~45ms/200ms)
//! 3. Parallel fixed bias resistor (220 kΩ) across each LDR per schematic
//! 4. 6.8 nF timing capacitors and subtle organic inter-stage component dispersion

const R_DARK: f32 = 2_500_000.0; // 2.5 MΩ dark resistance
const R_LIGHT: f32 = 350.0;      // 350 Ω minimum illuminated resistance
const R_BIAS: f32 = 220_000.0;   // 220 kΩ schematic shunt resistor
const CAP_FARADS: f32 = 6.8e-9;  // 6.8 nF (0.0068 µF) schematic capacitor
const GAMMA: f32 = 0.82;         // CdS response exponent

/// Per-stage component tolerance offsets for Phasor A (organic dispersion)
const DISPERSION_A: [f32; 6] = [-0.025, 0.018, -0.012, 0.031, -0.019, 0.022];
/// Per-stage component tolerance offsets for Phasor B (organic dispersion)
const DISPERSION_B: [f32; 6] = [0.021, -0.015, 0.028, -0.024, 0.014, -0.018];

#[derive(Clone, Copy)]
pub struct Optocoupler {
    // Current optical states
    light_fast: f32,
    light_slow: f32,
    effective_light: f32,

    // Filter coefficients for ballistics
    coeff_rise: f32,
    coeff_fall_fast: f32,
    coeff_fall_slow: f32,

    sample_rate: f32,
    is_phasor_b: bool,
}

impl Optocoupler {
    pub fn new(sample_rate: f32, is_phasor_b: bool) -> Self {
        let mut op = Self {
            light_fast: 0.0,
            light_slow: 0.0,
            effective_light: 0.0,
            coeff_rise: 0.0,
            coeff_fall_fast: 0.0,
            coeff_fall_slow: 0.0,
            sample_rate,
            is_phasor_b,
        };
        op.recalc_coefficients();
        op
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.recalc_coefficients();
    }

    pub fn reset(&mut self) {
        self.light_fast = 0.0;
        self.light_slow = 0.0;
        self.effective_light = 0.0;
    }

    fn recalc_coefficients(&mut self) {
        // Physical CdS time constants:
        // Attack (light on): tau ~ 10 ms
        let tau_rise = 0.010;
        // Release (light off): dual-stage decay
        // Fast component (70%): tau ~ 45 ms
        let tau_fall_fast = 0.045;
        // Slow persistent photoconductivity component (30%): tau ~ 200 ms
        let tau_fall_slow = 0.200;

        self.coeff_rise = 1.0 - (-1.0 / (self.sample_rate * tau_rise)).exp();
        self.coeff_fall_fast = 1.0 - (-1.0 / (self.sample_rate * tau_fall_fast)).exp();
        self.coeff_fall_slow = 1.0 - (-1.0 / (self.sample_rate * tau_fall_slow)).exp();
    }

    /// Advances the optical dynamics by one sample given an instantaneous drive voltage [0..1].
    #[inline(always)]
    pub fn tick(&mut self, drive_normalized: f32) {
        // Non-linear LED/driver current response: soft threshold knee
        let target_light = (drive_normalized.max(0.0) * 1.1).min(1.0);

        if target_light > self.effective_light {
            // Rising light: dominated by rapid photoexcitation
            self.light_fast += (target_light - self.light_fast) * self.coeff_rise;
            self.light_slow += (target_light - self.light_slow) * self.coeff_rise;
            self.effective_light = self.light_fast;
        } else {
            // Decaying light: two-phase carrier recombination and crystal trap release
            self.light_fast += (target_light - self.light_fast) * self.coeff_fall_fast;
            self.light_slow += (target_light - self.light_slow) * self.coeff_fall_slow;
            self.effective_light = 0.70 * self.light_fast + 0.30 * self.light_slow;
        }
    }

    /// Computes the cutoff frequency (Hz) for stage `stage_idx` (0..5).
    #[inline(always)]
    pub fn stage_cutoff_hz(&self, stage_idx: usize) -> f32 {
        let dispersion = if self.is_phasor_b {
            DISPERSION_B[stage_idx % 6]
        } else {
            DISPERSION_A[stage_idx % 6]
        };

        // Photoconductive power law
        let ldr_scale = (1.0 + 80.0 * self.effective_light).powf(GAMMA);
        let mut r_ldr = R_LIGHT + (R_DARK - R_LIGHT) / ldr_scale;

        // Apply physical component dispersion
        r_ldr *= 1.0 + dispersion;

        // Effective resistance with R_BIAS (220 kΩ) in parallel
        let r_eff = (r_ldr * R_BIAS) / (r_ldr + R_BIAS);

        // Allpass break frequency fc = 1 / (2 * pi * R * C)
        let fc = 1.0 / (2.0 * std::f32::consts::PI * r_eff * CAP_FARADS);

        // Clamp safely below Nyquist
        let max_fc = (0.45 * self.sample_rate).min(20_000.0);
        fc.clamp(40.0, max_fc)
    }

    /// Returns current effective optical brightness [0..1] for GUI/meters.
    pub fn brightness(&self) -> f32 {
        self.effective_light.clamp(0.0, 1.0)
    }
}
