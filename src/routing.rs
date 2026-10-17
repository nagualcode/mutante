//! "Phasor B Input" switch (S1).
//!
//! * `In A`  – Input A feeds both phasors (one instrument, two outputs:
//!   stereo phasing with Sweep Sync on Rev). Output A → left, B → right.
//! * `Out A` – the complete Phasor A output (dry + wet) feeds Phasor B: two
//!   phasors in series, the "12-stage" double-depth setting. Output B is
//!   sent to both channels, as in the manual's one-amp setup.
//! * `In B`  – independent inputs: left → Phasor A, right → Phasor B.
//!
//! In the plugin, "Input A" for `In A` / `Out A` is the mono sum of the
//! incoming stereo pair.

use truce::prelude::ParamEnum;

#[derive(ParamEnum)]
pub enum PhasorBInput {
    #[name = "In A"]
    InA,
    #[name = "Out A"]
    OutA,
    #[name = "In B"]
    InB,
}
