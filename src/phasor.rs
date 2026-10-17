//! One Bi-Phase phasor channel, signal path in volts.
//!
//! ```text
//!  jack ─C1/R1─ Z8A ─┬────────────────────────── R31 12k ───────────┐
//!                    │                                               │
//!                    └─R4─ Z8B (−1) ─ 6 × all-pass ─C12─┬─ R26 ─ Z9B (−1) ─ PM1 ─ R123 ─ C14 ─ R29 ─┴─ Z9A ─ out
//!                          ▲ (+2)                       │
//!                          └── R6‖C5 ── R7 ── FEEDBACK pot (R133) ┘
//! ```
//!
//! * The dry path is always present: with the footswitch off, PM1 goes dark
//!   and the output is just the buffered input.
//! * The wet signal is re-inverted by Z9B and summed passively with the dry
//!   signal; R123 is the factory "notch depth" trim that matches the legs.
//! * C12 (into R133‖R26) and C14 (into the mixer) high-pass the wet leg, so
//!   the lowest octaves stay dry and solid instead of being cancelled.
//! * The FEEDBACK pot taps the wet signal at J1-1 and feeds it to the
//!   non-inverting input of Z8B with a gain of up to ~0.86, which adds the
//!   resonant peaks between the notches.
//!
//! The feedback loop contains no unit delay: the instantaneous response of
//! every stage is linear in its input, so the loop is solved exactly each
//! sample before the states are advanced.

use crate::allpass::AllpassStage;
use crate::circuit::*;
use crate::driver::LedDriver;
use crate::optocoupler::{PhotoPod, PhotoSwitch};
use truce::prelude::ParamEnum;

#[derive(ParamEnum)]
pub enum PhasorASweepSource {
    #[name = "Gen 1"]
    Gen1,
    #[name = "Ped"]
    Pedal,
}

#[derive(ParamEnum)]
pub enum PhasorBSweepSource {
    #[name = "Gen 1"]
    Gen1,
    #[name = "Gen 2"]
    Gen2,
    #[name = "Ped"]
    Pedal,
}

const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

pub struct PhasorChannel {
    pub pod: PhotoPod,
    pub switch: PhotoSwitch,
    driver: LedDriver,
    stages: [AllpassStage; 6],
    stage_cap: [f32; 6],
    stage_w_scale: [f32; 6],
    input_hp: OnePole,
    c12: OnePole,
    c5: OnePole,
    c13: OnePole,
    c14: OnePole,
    r123: f32,
    sample_rate: f32,
    g_input: f32,
    g12: f32,
    g13: f32,
    // Cached feedback network for the last pot position.
    fb_pos: f32,
    fb_k: f32,
    fb_g5: f32,
    // Cached mixer network for the last PM1 resistance.
    pm_r: f32,
    mix_k: f32,
    mix_g14: f32,
}

impl PhasorChannel {
    pub fn new(sample_rate: f32, is_b: bool) -> Self {
        let tol = if is_b { CAP_TOL_B } else { CAP_TOL_A };
        let mut stage_cap = [C_STAGE; 6];
        for (c, t) in stage_cap.iter_mut().zip(tol) {
            *c *= t;
        }
        let mut ch = Self {
            pod: PhotoPod::new(sample_rate, is_b),
            switch: PhotoSwitch::new(sample_rate),
            driver: LedDriver::new(),
            stages: [AllpassStage::default(); 6],
            stage_cap,
            stage_w_scale: [0.0; 6],
            input_hp: OnePole::default(),
            c12: OnePole::default(),
            c5: OnePole::default(),
            c13: OnePole::default(),
            c14: OnePole::default(),
            r123: if is_b { R123_B } else { R123_A },
            sample_rate,
            g_input: 0.0,
            g12: 0.0,
            g13: 0.0,
            fb_pos: -1.0,
            fb_k: 0.0,
            fb_g5: 0.0,
            pm_r: -1.0,
            mix_k: 0.0,
            mix_g14: 0.0,
        };
        ch.set_sample_rate(sample_rate);
        ch
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.pod.set_sample_rate(sample_rate);
        self.switch.set_sample_rate(sample_rate);
        for (w_scale, &cap) in self.stage_w_scale.iter_mut().zip(&self.stage_cap) {
            *w_scale = 1.0 / (TWO_PI * cap * sample_rate);
        }
        self.g_input = one_pole_g(1.0 / (TWO_PI * R_IN * C_IN), sample_rate);
        self.g12 = one_pole_g(1.0 / (TWO_PI * R_J1_LOAD * C12), sample_rate);
        self.g13 = one_pole_g(1.0 / (TWO_PI * R27 * C13), sample_rate);
        self.fb_pos = -1.0;
        self.pm_r = -1.0;
    }

    /// Clear the filters and settle the photocells for the given sweep CV
    /// and footswitch state.
    pub fn reset(&mut self, cv: f32, on: bool) {
        for s in &mut self.stages {
            s.reset();
        }
        self.input_hp.reset();
        self.c12.reset();
        self.c5.reset();
        self.c13.reset();
        self.c14.reset();
        self.pod.settle(self.driver.current_ma(cv));
        self.switch.settle(on);
        self.fb_pos = -1.0;
        self.pm_r = -1.0;
    }

    #[inline]
    fn update_feedback_network(&mut self, pos: f32) {
        if (pos - self.fb_pos).abs() < 1e-5 {
            return;
        }
        self.fb_pos = pos;
        let (ratio, r_th) = pot_divider(pos, R_FB_POT);
        let r_src = R7 + r_th;
        self.fb_k = ratio * R6 / (R6 + r_src);
        let r_par = R6 * r_src / (R6 + r_src);
        self.fb_g5 = one_pole_g(1.0 / (TWO_PI * r_par * C5), self.sample_rate);
        let r_j1 = j1_load_resistance(pos);
        self.g12 = one_pole_g(1.0 / (TWO_PI * r_j1 * C12), self.sample_rate);
    }

    #[inline]
    fn update_mixer_network(&mut self, r_pm: f32) {
        if (r_pm - self.pm_r).abs() < 1e-3 * self.pm_r.abs() {
            return;
        }
        self.pm_r = r_pm;
        let r_wet = r_pm + self.r123 + R29;
        self.mix_k = R31 / (R31 + r_wet);
        self.mix_g14 = one_pole_g(1.0 / (TWO_PI * (R31 + r_wet) * C14), self.sample_rate);
    }

    /// Process one sample.
    ///
    /// * `x` – voltage at the input jack
    /// * `cv` – sweep control voltage after the DEPTH pot (J1-5 / J1-6)
    /// * `feedback` – FEEDBACK pot rotation, 0..1
    /// * `on` – footswitch state
    ///
    /// Returns the voltage at the phasor output jack.
    #[inline]
    pub fn process(&mut self, x: f32, cv: f32, feedback: f32, on: bool) -> f32 {
        // Z8A input buffer.
        let dry = opamp_clip(self.input_hp.hp(x, self.g_input));

        // LED driver → photocells → per-stage cutoff.
        self.pod.tick(self.driver.current_ma(cv));
        let mut g = [0.0f32; 6];
        for (i, gi) in g.iter_mut().enumerate() {
            *gi = one_pole_g_norm(self.pod.stage_conductance(i) * self.stage_w_scale[i]);
        }

        self.update_feedback_network(feedback);
        let (fb_k, g5, g12) = (self.fb_k, self.fb_g5, self.g12);

        // Solve the loop: u = 2·V+ − dry, V+ = LP5(k·w), w = HP12(AP6(u)).
        let (mut a, mut b) = (1.0f32, 0.0f32);
        for (st, &gi) in self.stages.iter().zip(&g) {
            let (sa, sb) = st.response(gi);
            a *= sa;
            b = sa * b + sb;
        }
        let c = 1.0 - g12;
        let d = -self.c12.lp_offset(g12);
        let e = g5 * fb_k;
        let f = self.c5.lp_offset(g5);
        let u_lin = (2.0 * e * (c * b + d) + 2.0 * f - dry) / (1.0 - 2.0 * e * c * a);
        let u = opamp_clip(u_lin);

        // Advance the states with the solved input.
        let mut y = u;
        for (st, &gi) in self.stages.iter_mut().zip(&g) {
            y = st.process(y, gi);
        }
        let w = self.c12.hp(y, g12);
        self.c5.lp(fb_k * w, g5);

        // Z9B re-inverts the wet signal (with 4558 rail limiting).
        let wet = opamp_clip(-self.c13.lp(w, self.g13));

        // PM1 optical switch and the passive R31/R29 mixer into Z9A.
        let r_pm = self.switch.tick(on);
        self.update_mixer_network(r_pm);
        let out = dry + self.mix_k * self.c14.hp(wet - dry, self.mix_g14);
        opamp_clip(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sweep::{GenCircuit, LfoShape, SweepGenerator};

    /// Steady-state gain (dB) of a sine through the channel.
    fn gain_db(freq: f32, cv: f32, fb: f32) -> f32 {
        let sr = 48_000.0;
        let mut ch = PhasorChannel::new(sr, false);
        ch.reset(cv, true);
        let n = 24_000;
        let (mut acc_in, mut acc_out) = (0.0f64, 0.0f64);
        for i in 0..n {
            let x = (2.0 * std::f32::consts::PI * freq * i as f32 / sr).sin();
            let y = ch.process(x, cv, fb, true);
            if i > n / 2 {
                acc_in += f64::from(x * x);
                acc_out += f64::from(y * y);
            }
        }
        10.0 * (acc_out / acc_in).log10() as f32
    }

    fn notches(cv: f32, fb: f32) -> Vec<(f32, f32)> {
        let mut freqs = Vec::new();
        let mut f = 30.0f32;
        while f < 20_000.0 {
            freqs.push(f);
            f *= 1.06;
        }
        let g: Vec<f32> = freqs.iter().map(|&f| gain_db(f, cv, fb)).collect();
        (1..g.len() - 1)
            .filter(|&i| g[i] < g[i - 1] && g[i] < g[i + 1] && g[i] < -6.0)
            .map(|i| (freqs[i], g[i]))
            .collect()
    }

    #[test]
    fn six_stages_give_three_deep_notches() {
        let n = notches(0.0, 0.0);
        assert_eq!(n.len(), 3, "{n:?}");
        assert!(n.iter().all(|&(_, db)| db < -15.0), "{n:?}");
        // Mid-sweep, the first notch sits in the low mids.
        assert!(n[0].0 > 250.0 && n[0].0 < 700.0, "{n:?}");
    }

    #[test]
    fn feedback_adds_peaks_between_notches() {
        let peak = (300..3000)
            .step_by(50)
            .map(|f| gain_db(f as f32, 0.0, 1.0))
            .fold(f32::MIN, f32::max);
        assert!(peak > 8.0 && peak < 16.0, "peak = {peak}");
    }

    #[test]
    fn footswitch_off_passes_dry_signal() {
        let sr = 48_000.0;
        let mut ch = PhasorChannel::new(sr, false);
        ch.reset(0.0, false);
        let (mut e_in, mut e_out) = (0.0f64, 0.0f64);
        for i in 0..24_000 {
            let x = (2.0 * std::f32::consts::PI * 1500.0 * i as f32 / sr).sin();
            let y = ch.process(x, 0.0, 0.0, false);
            if i > 12_000 {
                e_in += f64::from(x * x);
                e_out += f64::from(y * y);
            }
        }
        let db = 10.0 * (e_out / e_in).log10();
        assert!(db.abs() < 0.2, "{db}");
    }

    /// Range of the middle-stage cutoff over a few sweep cycles.
    fn sweep_range(rate: f32, shape: LfoShape) -> (f32, f32) {
        let sr = 48_000.0;
        let mut ch = PhasorChannel::new(sr, false);
        let mut sg = SweepGenerator::new(sr, GenCircuit::Gen1);
        sg.reset();
        ch.reset(-5.0, true);
        let n = (sr * 3.0 / rate) as usize;
        let (mut lo, mut hi) = (f32::MAX, 0.0f32);
        for i in 0..n {
            let cv = sg.tick(rate, shape);
            ch.process(0.0, cv, 0.0, true);
            if i > n * 2 / 3 {
                let f = ch.pod.center_hz();
                lo = lo.min(f);
                hi = hi.max(f);
            }
        }
        (lo, hi)
    }

    #[test]
    fn photocells_compress_fast_sweeps() {
        let (slow_lo, slow_hi) = sweep_range(0.5, LfoShape::Sine);
        let (fast_lo, fast_hi) = sweep_range(8.0, LfoShape::Sine);
        // A slow full-depth sweep spans more than five octaves...
        assert!(slow_hi / slow_lo > 32.0, "{slow_lo}..{slow_hi}");
        // ...while the cells cannot fully darken between fast cycles.
        assert!(fast_lo > 1.5 * slow_lo, "{fast_lo} vs {slow_lo}");
        assert!(fast_hi > 0.8 * slow_hi);
    }

    #[test]
    fn feedback_pot_loads_c12_highpass() {
        let r0 = j1_load_resistance(0.0);
        let r1 = j1_load_resistance(1.0);
        assert!((r0 - R_J1_LOAD).abs() < 1.0, "r0 = {r0}");
        assert!(r1 < r0 * 0.80 && r1 > 2400.0, "r1 = {r1}");
    }
}
