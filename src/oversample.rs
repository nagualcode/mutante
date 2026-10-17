//! 2× oversampling for the phasor channels.
//!
//! The RCA 4558 rail limiters sit inside the Z8B feedback loop, so they can't
//! be anti-aliased in isolation; instead both phasors (and the dry/wet mix,
//! so the two legs see the same filtering) run at twice the host rate.
//!
//! The half-band filters are the classic polyphase IIR pair: two chains of
//! first-order all-passes in z⁻², with elliptic coefficients computed from
//! the order and transition band (A. Valenzuela / L. de Soras design). They
//! are minimum-phase-like (no latency to report) and cost a few multiplies
//! per sample.

use std::f64::consts::PI;

/// Coefficients per filter (order 2·N + 1).
const NBR_COEFS: usize = 10;
/// Transition band as a fraction of the oversampled rate: the pass band ends
/// at (0.25 − TRANSITION)·2fs, i.e. 20.1 kHz for a 44.1 kHz host.
const TRANSITION: f64 = 0.022;

/// Elliptic half-band all-pass coefficients for `NBR_COEFS` and `TRANSITION`.
fn halfband_coefs() -> [f32; NBR_COEFS] {
    let mut k = ((1.0 - TRANSITION * 2.0) * PI / 4.0).tan();
    k *= k;
    let kksqrt = (1.0 - k * k).powf(0.25);
    let e = 0.5 * (1.0 - kksqrt) / (1.0 + kksqrt);
    let e4 = e.powi(4);
    let q = e * (1.0 + e4 * (2.0 + e4 * (15.0 + 150.0 * e4)));
    let order = (NBR_COEFS * 2 + 1) as f64;

    let mut coefs = [0.0f32; NBR_COEFS];
    for (index, coef) in coefs.iter_mut().enumerate() {
        let c = (index + 1) as f64;
        let (mut num, mut den) = (0.0f64, 0.0f64);
        for i in 0..32 {
            let fi = i as f64;
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            num += sign * q.powf(fi * (fi + 1.0)) * ((fi * 2.0 + 1.0) * c * PI / order).sin();
            if i > 0 {
                den += sign * q.powf(fi * fi) * (fi * 2.0 * c * PI / order).cos();
            }
        }
        let ww = num * q.powf(0.25) / (den + 0.5);
        let wwsq = ww * ww;
        let x = ((1.0 - wwsq * k) * (1.0 - wwsq / k)).sqrt() / (1.0 + wwsq);
        *coef = ((1.0 - x) / (1.0 + x)) as f32;
    }
    coefs
}

/// Two interleaved all-pass chains; even coefficients on path 0, odd on path 1.
#[derive(Clone, Copy)]
struct PolyphasePair {
    coefs: [f32; NBR_COEFS],
    x: [f32; NBR_COEFS],
    y: [f32; NBR_COEFS],
}

impl PolyphasePair {
    fn new() -> Self {
        Self {
            coefs: halfband_coefs(),
            x: [0.0; NBR_COEFS],
            y: [0.0; NBR_COEFS],
        }
    }

    fn reset(&mut self) {
        self.x = [0.0; NBR_COEFS];
        self.y = [0.0; NBR_COEFS];
    }

    #[inline]
    fn run(&mut self, mut path_0: f32, mut path_1: f32) -> (f32, f32) {
        for i in (0..NBR_COEFS).step_by(2) {
            let y0 = (path_0 - self.y[i]) * self.coefs[i] + self.x[i];
            self.x[i] = path_0;
            self.y[i] = y0;
            path_0 = y0;
            let j = i + 1;
            let y1 = (path_1 - self.y[j]) * self.coefs[j] + self.x[j];
            self.x[j] = path_1;
            self.y[j] = y1;
            path_1 = y1;
        }
        (path_0, path_1)
    }
}

/// Host-rate sample in, two oversampled samples out.
#[derive(Clone, Copy)]
pub struct Upsampler2x(PolyphasePair);

impl Upsampler2x {
    pub fn new() -> Self {
        Self(PolyphasePair::new())
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> [f32; 2] {
        let (a, b) = self.0.run(x, x);
        [a, b]
    }
}

/// Two oversampled samples in, one host-rate sample out.
#[derive(Clone, Copy)]
pub struct Downsampler2x(PolyphasePair);

impl Downsampler2x {
    pub fn new() -> Self {
        Self(PolyphasePair::new())
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }

    #[inline]
    pub fn process(&mut self, x: [f32; 2]) -> f32 {
        let (a, b) = self.0.run(x[1], x[0]);
        0.5 * (a + b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RMS level (dB re. a unit sine) of `sig` correlated against `freq`.
    fn tone_db(sig: &[f32], freq: f64, sr: f64) -> f64 {
        let (mut re, mut im) = (0.0f64, 0.0f64);
        for (n, &v) in sig.iter().enumerate() {
            let ph = 2.0 * PI * freq * n as f64 / sr;
            re += f64::from(v) * ph.cos();
            im += f64::from(v) * ph.sin();
        }
        let amp = 2.0 * (re * re + im * im).sqrt() / sig.len() as f64;
        20.0 * amp.log10()
    }

    #[test]
    fn passband_is_flat_and_images_are_rejected() {
        let sr = 48_000.0f64;
        for f in [100.0, 1_000.0, 10_000.0, 19_000.0] {
            let mut up = Upsampler2x::new();
            let mut down = Downsampler2x::new();
            let n = 48_000;
            let mut ups = Vec::with_capacity(2 * n);
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                let x = (2.0 * PI * f * i as f64 / sr).sin() as f32;
                let u = up.process(x);
                ups.extend_from_slice(&u);
                out.push(down.process(u));
            }
            let tail = &out[n / 2..];
            let pass = tone_db(tail, f, sr);
            assert!(pass.abs() < 0.05, "{f} Hz passband: {pass} dB");
            let image = tone_db(&ups[n..], sr - f, 2.0 * sr);
            if f < 18_000.0 {
                assert!(image < -90.0, "{f} Hz image: {image} dB");
            }
        }
    }

    #[test]
    fn downsampler_rejects_aliases() {
        let sr2 = 96_000.0f64;
        let mut down = Downsampler2x::new();
        let n = 96_000;
        let mut out = Vec::with_capacity(n / 2);
        let f = 30_000.0; // would alias to 18 kHz
        for i in (0..n).step_by(2) {
            let a = (2.0 * PI * f * i as f64 / sr2).sin() as f32;
            let b = (2.0 * PI * f * (i + 1) as f64 / sr2).sin() as f32;
            out.push(down.process([a, b]));
        }
        let alias = tone_db(&out[n / 4..], 18_000.0, sr2 / 2.0);
        assert!(alias < -90.0, "alias: {alias} dB");
    }
}
