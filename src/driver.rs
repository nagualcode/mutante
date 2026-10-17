//! LED current driver (Z7A + Q1 for pod PM3, Z7B + Q2 for pod PM4).
//!
//! The sweep CV and the R130 offset are summed by R97/R102/R103 at the
//! op-amp's non-inverting input. The op-amp forces the voltage at its
//! inverting input (R98 from the current-sense node, R100 to ground, and
//! R99 + CR9 to ground) to follow. While CR9 is off the stage has a gain of
//! 1 + R98/R100 ≈ 4.2; once the diode starts conducting the gain climbs
//! towards 1 + R98/(R100‖R99) ≈ 26. The result is an expanding V→I curve
//! that, combined with the photocells' I^-γ law, spreads the sweep evenly
//! over the musical range. Q1's emitter tops out a little below the rail,
//! which caps the LED current.
//!
//! The transfer curve is solved once (Shockley diode, bisection) into a
//! fixed table and interpolated at audio rate.

use crate::circuit::*;

const TABLE_LEN: usize = 512;
const V_MIN: f32 = 0.0;
const V_MAX: f32 = 1.2;

#[derive(Clone)]
pub struct LedDriver {
    /// LED current in mA for V+ in [V_MIN, V_MAX].
    table: [f32; TABLE_LEN],
    /// Gain of the R97/R102/R103 summing divider.
    k_in: f32,
}

impl LedDriver {
    pub fn new() -> Self {
        let mut table = [0.0f32; TABLE_LEN];
        for (i, slot) in table.iter_mut().enumerate() {
            let v = V_MIN as f64 + (V_MAX - V_MIN) as f64 * i as f64 / (TABLE_LEN - 1) as f64;
            *slot = solve_led_current_ma(v) as f32;
        }
        let g_sum = 1.0 / R97 + 1.0 / R102 + 1.0 / R103;
        Self {
            table,
            k_in: (1.0 / R97) / g_sum,
        }
    }

    /// LED current in mA for a sweep control voltage (volts at J1-5/J1-6).
    #[inline]
    pub fn current_ma(&self, cv: f32) -> f32 {
        let v_plus = (cv + V_LED_OFFSET) * self.k_in;
        if v_plus <= V_MIN {
            return 0.0;
        }
        let pos = (v_plus - V_MIN) * ((TABLE_LEN - 1) as f32 / (V_MAX - V_MIN));
        if pos >= (TABLE_LEN - 1) as f32 {
            return self.table[TABLE_LEN - 1];
        }
        let i = pos as usize;
        let frac = pos - i as f32;
        self.table[i] + (self.table[i + 1] - self.table[i]) * frac
    }
}

impl Default for LedDriver {
    fn default() -> Self {
        Self::new()
    }
}

/// Current through R99 + CR9 for a given voltage at Z7A pin 2.
fn diode_branch_current(v: f64) -> f64 {
    if v <= 0.0 {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0f64, v / R99 as f64);
    for _ in 0..80 {
        let i = 0.5 * (lo + hi);
        let vd = DIODE_NVT * (i / DIODE_IS + 1.0).ln();
        if i * R99 as f64 + vd > v {
            hi = i;
        } else {
            lo = i;
        }
    }
    0.5 * (lo + hi)
}

fn solve_led_current_ma(v_plus: f64) -> f64 {
    if v_plus <= 0.0 {
        return 0.0;
    }
    let i_fb = v_plus / R100 as f64 + diode_branch_current(v_plus);
    let v_sense = (v_plus + R98 as f64 * i_fb).min(V_SENSE_MAX as f64);
    1000.0 * v_sense / R_SENSE as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_is_monotonic_and_expanding() {
        let d = LedDriver::new();
        let mut prev = -1.0;
        for i in 0..100 {
            let cv = -6.0 + 0.12 * i as f32;
            let c = d.current_ma(cv);
            assert!(c >= prev);
            prev = c;
        }
        // Full negative sweep turns the LED off, full positive reaches ~16 mA.
        assert_eq!(d.current_ma(-5.0), 0.0);
        let top = d.current_ma(5.0);
        assert!(top > 14.0 && top < 18.0, "top = {top}");
        // Expanding: the upper half of the CV range adds more current.
        let mid = d.current_ma(0.0);
        assert!(top - mid > mid);
    }
}
