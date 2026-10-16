//! Musitronics Bi-Phase Routing Matrix.
//!
//! Models the authentic 3-position "Phasor B Input" hardware rotary switch (S1):
//! 1. `InB`: True Stereo / Dual Mono. Inputs A and B are processed independently.
//! 2. `InA`: Dual Parallel (Mono to Stereo). Input A feeds both Phasor A and Phasor B.
//! 3. `OutA`: 12-Stage Serial Cascade. Phasor A feeds directly into Phasor B!
//!
//! Plus modern DAW stereo configurations:
//! 4. `StereoParallel`: Stereo cross-mixed parallel processing.

use truce::prelude::ParamEnum;

#[derive(ParamEnum)]
pub enum PhasorBInput {
    #[name = "In B (Stereo / Dual Mono)"]
    InB,
    #[name = "In A (Mono -> Stereo)"]
    InA,
    #[name = "Out A (12-Stage Series)"]
    OutA,
    #[name = "Stereo Parallel"]
    StereoParallel,
}
