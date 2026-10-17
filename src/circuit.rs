//! Component values transcribed from the ARP/Musitronics service schematics
//! (drawing 77204 rev 2 "BD 1 Bi-Phasor" audio I/O + sweep generators and the
//! "BD 2" phase-shift / LED-driver board). Reference designators are kept in
//! the comments so every number can be traced back to the drawings in
//! `schematics1/` and `schematics5/`.
//!
//! Signal levels are modelled in volts. The plugin maps 0 dBFS to
//! [`VOLTS_PER_FS`] at the input jack, which matches the "2 V input" reference
//! used in the owner's manual noise specification.

/// 0 dBFS at the input jack corresponds to 2 V peak.
pub const VOLTS_PER_FS: f32 = 2.0;

/// RCA 4558 output swing on the ±15 V rails (Z1–Z16).
pub const OPAMP_SWING: f32 = 13.5;

// --- Input buffer (Z8A / Z10A) -------------------------------------------------
/// C1 / C17 coupling capacitor.
pub const C_IN: f32 = 0.047e-6;
/// R1 / R33 input bias resistor (sets the 390 kΩ input impedance).
pub const R_IN: f32 = 390e3;

// --- Feedback summing amplifier (Z8B / Z10B) -----------------------------------
/// R6 / R38: lower leg of the feedback divider at the non-inverting input.
pub const R6: f32 = 4.7e3;
/// R7 / R39: series resistor from the FEEDBACK pot wiper.
pub const R7: f32 = 6.2e3;
/// C5 / C21 across R6: rolls the feedback off above ~19 kHz.
pub const C5: f32 = 0.0033e-6;
/// R133 / R134 FEEDBACK pot (10 kΩ, linear).
pub const R_FB_POT: f32 = 10e3;

// --- Phase-shift ladder (Z1–Z3 / Z4–Z6) -----------------------------------------
/// R10, R13, ... R57: fixed 220 kΩ in parallel with each photocell.
pub const R_SHUNT: f32 = 220e3;
/// C6–C11 / C22–C27: 0.0068 µF 10 % polyester.
pub const C_STAGE: f32 = 0.0068e-6;

// --- Wet coupling and output mixer ----------------------------------------------
/// C12 / C28: 0.47 µF coupling cap from the last stage to J1-1 / J1-13.
pub const C12: f32 = 0.47e-6;
/// R26 / R59: 4.7 kΩ input resistor of the Z9B / Z11B inverting wet amplifier.
pub const R26: f32 = 4.7e3;
/// Load seen by C12 at feedback = 0: R133 (10 kΩ to ground) in parallel with
/// R26 (4.7 kΩ into the Z9B virtual ground).
pub const R_J1_LOAD: f32 = R_FB_POT * R26 / (R_FB_POT + R26);
/// C13 across R27 (Z9B): 33.9 kHz roll-off of the re-inverted wet signal.
pub const C13: f32 = 0.001e-6;
pub const R27: f32 = 4.7e3;
/// C14 / C30 wet coupling cap into the passive mixer.
pub const C14: f32 = 0.15e-6;
/// R29 / R62 wet mixing resistor.
pub const R29: f32 = 4.7e3;
/// R31 / R63 dry mixing resistor (from the Z8A buffer).
pub const R31: f32 = 12e3;
/// R123 / R124 "NOTCH DEPTH" trimmer, set at the factory so that the wet leg
/// matches R31. Real units are never perfectly trimmed; the small error here
/// keeps the notches deep but finite (about -36 dB).
pub const R123_A: f32 = 6.95e3;
pub const R123_B: f32 = 6.62e3;

// --- Optical bypass PM1 / PM2 (PBT5-12 photocell + lamp) ------------------------
pub const R_PM_ON: f32 = 500.0;
pub const R_PM_OFF: f32 = 8e6;

// --- LED driver (Z7A/Q1 for PM3, Z7B/Q2 for PM4) ---------------------------------
/// R97 / R102 into R103: CV and offset summing divider at Z7A pin 3.
pub const R97: f32 = 47e3;
pub const R102: f32 = 47e3;
pub const R103: f32 = 3.9e3;
/// R98: feedback from the LED cathode / sense node to Z7A pin 2.
pub const R98: f32 = 220e3;
/// R100: low-level gain-setting resistor from pin 2 to ground.
pub const R100: f32 = 68e3;
/// R99 + CR9 (1N4148): kicks in above ~0.35 V and steepens the V→I curve.
pub const R99: f32 = 10e3;
/// LED current sense: R101 (220 Ω) plus the R129 1 kΩ trimmer.
pub const R_SENSE: f32 = 220.0 + 330.0;
/// Highest voltage Q1's emitter can reach across the sense resistor
/// (op-amp swing minus Vbe minus the LED forward drop).
pub const V_SENSE_MAX: f32 = 10.8;
/// R130 offset trimmer as seen through R102 (volts at the wiper).
pub const V_LED_OFFSET: f32 = 4.73;
/// 1N4148 SPICE parameters.
pub const DIODE_IS: f64 = 2.52e-9;
pub const DIODE_NVT: f64 = 1.752 * 0.025_85;

// --- Sweep generators -----------------------------------------------------------
/// Calibrated triangle amplitude at the generator outputs (R128/R127 trims).
pub const GEN_TRI_PEAK: f32 = 5.0;
/// Gen 1 "square": Z13A comparator through R76 120 kΩ into C36 0.47 µF, loaded
/// by R77 + R128. Amplitude relative to the triangle and the RC time constant.
pub const GEN1_SQUARE_PEAK: f32 = 4.3;
pub const GEN1_SQUARE_TAU: f32 = 0.0186;
/// Gen 2 "square": Z14A through R92 10 kΩ into C35 2.2 µF, loaded by R127 and
/// the depth pot. It comes out hotter than its triangle and drives the LED
/// into both extremes, which is why the Gen 2 square sounds harder.
pub const GEN2_SQUARE_PEAK: f32 = 8.5;
pub const GEN2_SQUARE_TAU: f32 = 0.0144;

/// DEPTH pot (R131 / R138, 10 kΩ) sits on top of R96 / R105 (560 Ω), so the
/// minimum setting still passes ~5 % of the sweep.
pub const R_DEPTH_POT: f32 = 10e3;
pub const R_DEPTH_FLOOR: f32 = 560.0;

// --- Photocell pods PM3 / PM4 ---------------------------------------------------
/// Photocell resistance at 1 mA of LED current (each pod shares one LED
/// between six cells, so the cells see far less light than a vactrol).
pub const LDR_R_AT_1MA_A: f32 = 55e3;
pub const LDR_R_AT_1MA_B: f32 = 58e3;
/// Photoconductive exponent: R ∝ I^-γ.
pub const LDR_GAMMA: f32 = 0.9;
/// Dark resistance of a CdS cell.
pub const LDR_R_DARK: f32 = 5e6;

/// Per-unit tolerances of the 10 % polyester stage capacitors.
pub const CAP_TOL_A: [f32; 6] = [1.04, 0.97, 1.06, 0.99, 0.95, 1.02];
pub const CAP_TOL_B: [f32; 6] = [0.98, 1.05, 0.96, 1.03, 1.01, 0.94];
/// How much of the pod's light reaches each of the six cells.
pub const LIGHT_COUPLING_A: [f32; 6] = [1.00, 0.88, 1.10, 0.95, 1.07, 0.92];
pub const LIGHT_COUPLING_B: [f32; 6] = [0.93, 1.06, 0.97, 1.12, 0.90, 1.02];

/// Thevenin model of a pot used as a voltage divider: returns
/// (division ratio, source resistance) for wiper position `p` in [0, 1].
#[inline]
pub fn pot_divider(p: f32, r_total: f32) -> (f32, f32) {
    let p = p.clamp(0.0, 1.0);
    (p, p * (1.0 - p) * r_total)
}

/// RCA 4558 output stage: clean until close to the rails, then a firm knee.
#[inline]
pub fn opamp_clip(x: f32) -> f32 {
    let r = x * (1.0 / OPAMP_SWING);
    let r2 = r * r;
    x / (1.0 + r2 * r2).sqrt().sqrt()
}

/// Effective load resistance at J1-1 (seen by coupling capacitor C12 = 0.47 µF)
/// as a function of the FEEDBACK pot wiper position `p` in [0, 1].
///
/// At p = 0 the wiper is grounded and R133 presents its full 10 kΩ in parallel
/// with R26 (4.7 kΩ) → 3197 Ω (fc ≈ 105.8 Hz). As feedback increases toward
/// p = 1, the wiper load R7 + R6 (6.2 kΩ + 4.7 kΩ = 10.9 kΩ) pulls the R133
/// branch down to 10 kΩ ‖ 10.9 kΩ = 5215 Ω, reducing the J1-1 load to 2472 Ω
/// (fc ≈ 136.8 Hz).
#[inline]
pub fn j1_load_resistance(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    let r_wiper_load = R7 + R6;
    let r_bot = (p * R_FB_POT * r_wiper_load) / (p * R_FB_POT + r_wiper_load);
    let r_133_eff = (1.0 - p) * R_FB_POT + r_bot;
    (r_133_eff * R26) / (r_133_eff + R26)
}

/// TPT one-pole coefficient G = g / (1 + g) for normalized cutoff `w_norm = fc / fs`.
///
/// Uses a Padé [3/2] rational approximation of `tan(π · w_norm)` for `w_norm <= 0.25`
/// (relative error < 2e-5 across the entire Bi-Phase sweep range) and falls back
/// to exact `tan` near Nyquist.
#[inline]
pub fn one_pole_g_norm(w_norm: f32) -> f32 {
    let w = w_norm.clamp(2.0e-8, 0.49);
    let theta = std::f32::consts::PI * w;
    let g = if w <= 0.25 {
        let t2 = theta * theta;
        theta * (15.0 - t2) / (15.0 - 6.0 * t2)
    } else {
        theta.tan()
    };
    g / (1.0 + g)
}

/// TPT one-pole coefficient G = g / (1 + g) for cutoff `fc` at sample rate `fs`.
#[inline]
pub fn one_pole_g(fc: f32, fs: f32) -> f32 {
    one_pole_g_norm(fc / fs)
}

/// Topology-preserving-transform one-pole low-pass, also used for the
/// high-pass (x - lp) responses of the coupling networks.
#[derive(Clone, Copy, Default)]
pub struct OnePole {
    s: f32,
}

impl OnePole {
    #[inline]
    pub fn reset(&mut self) {
        self.s = 0.0;
    }

    #[inline]
    pub fn lp(&mut self, x: f32, big_g: f32) -> f32 {
        let v = (x - self.s) * big_g;
        let y = v + self.s;
        self.s = y + v;
        y
    }

    #[inline]
    pub fn hp(&mut self, x: f32, big_g: f32) -> f32 {
        x - self.lp(x, big_g)
    }

    /// Instantaneous response of the low-pass: lp = G·x + (1 − G)·s.
    #[inline]
    pub fn lp_offset(&self, big_g: f32) -> f32 {
        (1.0 - big_g) * self.s
    }
}
