//! Cadmium-sulphide photocell models.
//!
//! [`PhotoPod`] is one Musitronics phase pod (PM3 / PM4): a single LED in a
//! light-tight can surrounded by six CdS cells, one per all-pass stage.
//! [`PhotoSwitch`] is the PM1 / PM2 lamp+cell that the footswitch uses to fade
//! the wet signal in and out.
//!
//! The cell conductance follows the photoconductive law G ∝ I^γ. Its
//! dynamics are integrated in the log-conductance domain, which reproduces
//! the way CdS resistance changes multiplicatively: brightening is fast
//! (a few ms), while darkening slows down the darker the cell gets, because
//! recombination of the remaining carriers takes longer at low densities.
//! At high sweep rates this makes the cells "hang" on the bright side and
//! narrows the sweep, which is the behaviour the manual describes when it
//! suggests reducing depth at higher speeds.

use crate::circuit::*;

/// Floor for the light-driven conductance so the log state stays finite.
const G_FLOOR: f32 = 1.0 / 20e6;
const TAU_ON: f32 = 0.0035;
const TAU_OFF_REF: f32 = 0.025;
const TAU_OFF_MAX: f32 = 1.5;
/// How strongly the darkening time constant grows as the cell gets darker.
const TAU_OFF_SLOPE: f32 = 0.42;
/// Fixed parallel shunt conductance of each stage (5 MΩ dark cell ‖ 220 kΩ shunt).
const G_STAGE_FLOOR: f32 = 1.0 / LDR_R_DARK + 1.0 / R_SHUNT;

#[derive(Clone, Copy)]
struct CdsCell {
    /// ln of the light-driven conductance (siemens).
    ln_g: f32,
    /// ln of the conductance at 1 mA, the reference for the decay slope.
    ln_g_ref: f32,
    inv_r_at_1ma: f32,
    dt: f32,
}

impl CdsCell {
    fn new(r_at_1ma: f32, coupling: f32, sample_rate: f32) -> Self {
        Self {
            ln_g: G_FLOOR.ln(),
            ln_g_ref: (1.0 / r_at_1ma).ln(),
            inv_r_at_1ma: coupling / r_at_1ma,
            dt: 1.0 / sample_rate,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.dt = 1.0 / sample_rate;
    }

    /// Steady-state conductance for an LED current in mA.
    #[inline]
    fn target_ln_g(&self, current_ma: f32) -> f32 {
        let g = if current_ma > 1e-6 {
            self.inv_r_at_1ma * current_ma.powf(LDR_GAMMA)
        } else {
            0.0
        };
        (g + G_FLOOR).ln()
    }

    #[inline]
    fn advance(&mut self, target_ln_g: f32) {
        let diff = target_ln_g - self.ln_g;
        if diff >= 0.0 {
            let x = self.dt / TAU_ON;
            self.ln_g += diff * (x / (1.0 + x));
        } else {
            // Recombination slows down as carrier density drops below the 1 mA
            // reference; when brighter than 1 mA, tau starts at TAU_OFF_REF
            // rather than shrinking below TAU_ON. Clamping the log driving
            // step prevents the dark floor asymptote from artificially
            // accelerating initial decay when the LED snaps off.
            let darkness = (self.ln_g_ref - self.ln_g).max(0.0);
            let tau = (TAU_OFF_REF * (TAU_OFF_SLOPE * darkness).exp())
                .clamp(TAU_OFF_REF, TAU_OFF_MAX);
            let x = self.dt / tau;
            self.ln_g += diff.max(-2.5) * (x / (1.0 + x));
        }
    }

    #[inline]
    fn conductance(&self) -> f32 {
        self.ln_g.exp()
    }

    fn settle(&mut self, current_ma: f32) {
        self.ln_g = self.target_ln_g(current_ma);
    }
}

/// One six-cell phase pod (PM3 / PM4), modeling all six CdS cells individually.
#[derive(Clone, Copy)]
pub struct PhotoPod {
    cells: [CdsCell; 6],
    current_ma: f32,
}

impl PhotoPod {
    pub fn new(sample_rate: f32, is_b: bool) -> Self {
        let r1 = if is_b { LDR_R_AT_1MA_B } else { LDR_R_AT_1MA_A };
        let coupling = if is_b { LIGHT_COUPLING_B } else { LIGHT_COUPLING_A };
        let cells = std::array::from_fn(|i| CdsCell::new(r1, coupling[i], sample_rate));
        Self {
            cells,
            current_ma: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for cell in &mut self.cells {
            cell.set_sample_rate(sample_rate);
        }
    }

    /// Jump straight to the steady state for `current_ma` (used on reset so
    /// the plugin does not start with a slow fade-in from darkness).
    pub fn settle(&mut self, current_ma: f32) {
        self.current_ma = current_ma;
        for cell in &mut self.cells {
            cell.settle(current_ma);
        }
    }

    #[inline]
    pub fn tick(&mut self, current_ma: f32) {
        self.current_ma = current_ma;
        let i_pow = if current_ma > 1e-6 {
            current_ma.powf(LDR_GAMMA)
        } else {
            0.0
        };
        for cell in &mut self.cells {
            let t = (cell.inv_r_at_1ma * i_pow + G_FLOOR).ln();
            cell.advance(t);
        }
    }

    /// Total conductance to ground of stage `i`: photocell in parallel with
    /// its 5 MΩ dark resistance and the 220 kΩ shunt (R10, R13, ...).
    #[inline]
    pub fn stage_conductance(&self, i: usize) -> f32 {
        self.cells[i].conductance() + G_STAGE_FLOOR
    }

    /// Resistance to ground of stage `i`: photocell in parallel with the
    /// 220 kΩ shunt (R10, R13, ...).
    #[inline]
    pub fn stage_resistance(&self, i: usize) -> f32 {
        1.0 / self.stage_conductance(i)
    }

    /// 0..1 display value for the pod glow (log of the light level).
    pub fn glow(&self) -> f32 {
        (self.current_ma / 16.0).clamp(0.0, 1.0).sqrt()
    }

    /// Representative cutoff (Hz) of the middle stages, for display.
    pub fn center_hz(&self) -> f32 {
        self.stage_conductance(2) / (2.0 * std::f32::consts::PI * C_STAGE)
    }
}

/// PM1 / PM2 optical bypass element.
#[derive(Clone, Copy)]
pub struct PhotoSwitch {
    ln_r: f32,
    dt: f32,
}

impl PhotoSwitch {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            ln_r: R_PM_ON.ln(),
            dt: 1.0 / sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.dt = 1.0 / sample_rate;
    }

    pub fn settle(&mut self, on: bool) {
        self.ln_r = if on { R_PM_ON } else { R_PM_OFF }.ln();
    }

    /// Advance and return the cell resistance in ohms.
    #[inline]
    pub fn tick(&mut self, on: bool) -> f32 {
        let (target, tau) = if on {
            (R_PM_ON.ln(), 0.006)
        } else {
            // Lamp off: the cell darkens slowly, faster at first.
            let darkness = (self.ln_r - R_PM_ON.ln()).max(0.0);
            (R_PM_OFF.ln(), 0.030 * (0.45 * darkness).exp())
        };
        let x = self.dt / tau;
        self.ln_r += (target - self.ln_r) * (x / (1.0 + x));
        self.ln_r.exp()
    }

    pub fn is_lit(&self) -> bool {
        self.ln_r < (4.0 * R_PM_ON).ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn darkening_is_slower_than_brightening() {
        let sr = 48_000.0;
        let mut pod = PhotoPod::new(sr, false);
        pod.settle(10.0);
        let r_bright = pod.stage_resistance(0);
        // Step LED off for 3.5 ms (one TAU_ON): resistance must not jump more than 1.5x.
        for _ in 0..(0.0035 * sr) as usize {
            pod.tick(0.0);
        }
        let r_after_3_5ms = pod.stage_resistance(0);
        assert!(
            r_after_3_5ms < 1.5 * r_bright,
            "darkened too fast: {r_bright} -> {r_after_3_5ms}"
        );
    }

    #[test]
    fn six_cells_disperse_dynamically_during_decay() {
        let sr = 48_000.0;
        let mut pod = PhotoPod::new(sr, false);
        pod.settle(2.0);
        let ratio_static = pod.cells[2].conductance() / pod.cells[1].conductance();
        // Let the pod decay in darkness for 60 ms: dimmer cell (stage 1, coupling 0.88)
        // has a longer tau_off than brighter cell (stage 2, coupling 1.10), so their
        // conductance ratio evolves dynamically during the sweep.
        for _ in 0..(0.060 * sr) as usize {
            pod.tick(0.0);
        }
        let ratio_dynamic = pod.cells[2].conductance() / pod.cells[1].conductance();
        assert!(
            (ratio_dynamic - ratio_static).abs() > 0.02,
            "expected dynamic dispersion between cells: static={ratio_static}, dynamic={ratio_dynamic}"
        );
    }
}
