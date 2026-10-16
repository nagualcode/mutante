# Mutante (Mu-Tron Bi-Phase Physical Emulation)

> **Jardim Sonoro** · Zero-Allocation DSP Audio Plugin  
> Emulating the Legendary 1974 Musitronics Mu-Tron Bi-Phase Dual Optical Phaser

[![pluginval](https://img.shields.io/badge/pluginval-Strictness%20Level%205%20PASS-brightgreen.svg)](#audio-verification--laboratory-benchmarks)
[![auval](https://img.shields.io/badge/auval-AUv2%20%26%20AUv3%20PASS-brightgreen.svg)](#audio-verification--laboratory-benchmarks)
[![framework](https://img.shields.io/badge/framework-Truce%206.3%20Rust-orange.svg)](https://github.com/truce-audio/truce)
[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

---

## 🏛️ 1. Overview & Hardware Heritage

The **Mu-Tron Bi-Phase**, designed by Mike Beigel and released by **Musitronics** in 1974, is widely regarded as the holy grail of analog phasers. Prominently featured on records by Lee "Scratch" Perry, Smashing Pumpkins, Pink Floyd, and George Clinton, its hypnotic, three-dimensional acoustic swirl stems from a unique dual-channel optoelectronic architecture:

- **Dual Independent 6-Stage Optical Phasors:** Two discrete 6-pole optical phase-shifter circuits (Phasor A and Phasor B) that can be operated independently (dual mono/stereo), in parallel, or cascaded in series into a colossal **12-stage phaser**.
- **Cadmium Sulfide (CdS) Optical Pods (PM3 & PM4):** Light-tight modules housing central incandescent/LED drivers illuminating 6 matched CdS photoresistors per channel. The organic photoconductive memory, asymmetric relaxation ballistics, and component dispersion create the lush, liquid phasing unattainable by standard solid-state FET or OTA topologies.
- **Dual Sweep Generators:** 0.1 Hz to 18.0 Hz wide-range LFOs with diode-bridge wave-shaping producing pseudo-sine and square waves, capable of synchronous (normal/reverse) or asynchronous swirling.
- **Inverting Feedback Summing:** Distinct feedback loops with analog op-amp rail saturation that generate sharp formant resonances without digital harshness.
- **Dynamic C-100 Foot Controller / Envelope Follower:** Real-time optical expression or signal envelope-driven phase sweeps.

**Mutante** is a component-level physical reconstruction of this circuit in Rust, developed under the **Jardim Sonoro** umbrella using the **Truce 6.3** framework. It guarantees zero memory allocations on the real-time audio thread and has been forensically verified against original Musitronics factory schematics and laboratory measurements.

---

## 🔬 2. Forensic Reverse Engineering & Circuit Architecture

```
           +--------------------------------------------------------------+
           |                    MUTANTE HARDWARE TOPOLOGY                 |
           +--------------------------------------------------------------+

      Audio In A -----------------------------------+
                                                    |
                                                    v
     Sweep Gen 1 +--------+    [Depth A]     +-------------+
     (0.1 - 18Hz)| LFO 1  |------( x )------>|  PHASOR A   |-----> Output A
     Sine/Square +--------+                  |  (6 Poles)  |          |
                                             +------+------+          |
                                                    | (Feedback A)    |
                                                    +--------+        |
                                                             |        |
                                        +--------------------+        |
                                        | (Out A Cascade)             |
                                        v                             |
      Audio In B ----->[S1 Routing Switch]                            |
                         |-- In B (Dual Mono/Stereo)                  |
                         |-- In A (Mono -> Stereo)                    |
                         +-- Out A (12-Stage Series Cascade)          |
                                        |                             |
                                        v                             |
     Sweep Gen 2 +--------+    [Depth B]     +-------------+          |
     (0.1 - 18Hz)| LFO 2  |------( x )------>|  PHASOR B   |-----> Output B
     Sine/Square +--------+                  |  (6 Poles)  |
                                             +------+------+
                                                    | (Feedback B)
                                                    +--------+
```

### 2.1 The CdS Optocoupler Physics (`optocoupler.rs`)
Unlike FET-based phasers (e.g. Phase 90) which exhibit immediate exponential resistance changes and odd-order distortion, Cadmium Sulfide photoresistors possess distinct physical semiconductor properties:

1. **Photoconductive Power Law:**
   $$R(L) = R_{\text{light}} + \frac{R_{\text{dark}} - R_{\text{light}}}{(1 + k \cdot L)^\gamma}$$
   - $R_{\text{dark}} = 2.5\text{ M}\Omega$ (dark resistance)
   - $R_{\text{light}} = 350\,\Omega$ (minimum illuminated resistance)
   - $\gamma = 0.82$ (photo-excitation exponent)

2. **Asymmetric Dual-Time-Constant Kinetics:**
   - **Photoexcitation (Rise / Light On):** Rapid carrier generation governed by $\tau_{\text{rise}} \approx 10\text{ ms}$.
   - **Carrier Recombination & Trap Release (Decay / Light Off):** Two-phase ballistic relaxation:
     - 70% fast direct carrier recombination ($\tau_{\text{fast}} \approx 45\text{ ms}$).
     - 30% persistent photoconductivity from deep crystal lattice traps ($\tau_{\text{slow}} \approx 200\text{ ms}$).

3. **Schematic Shunt Resistor Network:**
   Each LDR is wired in parallel with a fixed $220\text{ k}\Omega$ metal film bias resistor ($R_{\text{bias}}$) and a $6.8\text{ nF}$ poly timing capacitor ($C$), preventing infinite resistance in dark states:
   $$R_{\text{eff}} = \frac{R_{\text{ldr}} \cdot R_{\text{bias}}}{R_{\text{ldr}} + R_{\text{bias}}}$$
   $$f_c = \frac{1}{2\pi \cdot R_{\text{eff}} \cdot C}$$

4. **Organic Stage Dispersion:**
   In the physical hardware pod, light angles and CdS crystal variations create subtle stage-to-stage tolerance spreads ($\pm 1.5\% - 3.1\%$). Mutante accurately preserves these calibrated offsets (`DISPERSION_A` and `DISPERSION_B`), creating organic phase cancellation nulls without sterile synthetic symmetry.

### 2.2 First-Order Allpass Filter Engine (`allpass.rs`)
Each channel cascades 6 active op-amp allpass stages (expandable up to 12 stages). To ensure unconditional stability under audio-rate modulation, each filter is solved via the Bilinear Transform into a Transposed Direct Form II structure:

$$H(z) = \frac{a - z^{-1}}{1 - a z^{-1}}$$
$$a = \frac{\tan(\pi f_c / f_s) - 1}{\tan(\pi f_c / f_s) + 1}$$

- Normalized unity gain across the entire audio spectrum ($|H(e^{j\omega})| \equiv 1$).
- Per-sample modulation safe: internal states are maintained across coefficient updates without clicks or energy blowups.

### 2.3 Op-Amp Rail Soft-Saturation & Feedback Loop (`phasor.rs`)
In the original hardware, feedback is routed through an inverting summing node ($Z8\text{B} / Z9\text{B}$) with $\pm 15\text{V}$ supply rails:
$$x_{\text{sum}}[n] = x_{\text{in}}[n] - \beta \cdot y_{\text{fb}}[n]$$
$$y_{\text{fb}}[n] = \tanh\left(1.2 \cdot y_{\text{allpass}}[n]\right)$$
This prevents feedback runaway while creating rich, singing analog formants as feedback increases towards 100%.

### 2.4 Sweep Generators & Diode Shaping (`sweep.rs`)
- Sweep rate: **0.1 Hz to 18.0 Hz** (Hardware authentic 10-second period per cycle at minimum).
- Shapes:
  - **Diode-Bridge Pseudo-Sine:** Models the 1N914 diode-bridge (CR2–CR5) rounding circuit.
  - **Bipolar Square:** Crisp step-function transitions for stutter and rhythmic phasing.
- Sweep Sync: Phasor B can track Sweep Gen 1 in **Normal** ($0^\circ$) or **Reverse** ($180^\circ$ out-of-phase), producing wide, swirling binaural stereo spreads.

---

## 📊 3. Audio Verification & Laboratory Benchmarks

The Mutante DSP engine was verified via `tools/measure_mutante.py` and industry-standard testing tools:

```
======================================================================
MUTANTE (MU-TRON BI-PHASE) AUDIO VERIFICATION & VALIDATION SUITE
======================================================================

[1] Running Steinberg Pluginval (Strictness Level 5)...
  Result: PASSED (Strictness 5: SUCCESS)

[2] Loading Mutante VST3 into pedalboard engine...
  Loaded: Mutante by Jardim Sonoro
  Total Parameters Exposed: 28

[3] Testing Hard-Bypass Transparency...
  Max Absolute Difference from Dry Wire: 0.0000000000
  Bit-clean wire passthrough verified: 100% transparent!

[4] Measuring Comb-Filter Notches for 6 Poles (Classic Mu-Tron Stage)...
  Found 3 notches in passband:
    Notch 1:    46.9 Hz  (Depth:  -12.5 dB)
    Notch 2:   161.1 Hz  (Depth:  -14.6 dB)
    Notch 3:   474.6 Hz  (Depth:  -32.2 dB)
  -> Exactly 3 cancellation notches verified for 6-pole stage!

[5] Measuring Comb-Filter Notches for 12 Poles (Series Cascade)...
  Found 6 notches in passband:
    Notch 1:    26.4 Hz  (Depth:  -11.5 dB)
    Notch 2:    82.0 Hz  (Depth:  -11.7 dB)
    Notch 3:   146.5 Hz  (Depth:  -12.3 dB)
    Notch 4:   228.5 Hz  (Depth:  -13.7 dB)
    Notch 5:   363.3 Hz  (Depth:  -18.6 dB)
    Notch 6:   925.8 Hz  (Depth:  -39.5 dB)
  -> Exactly 6 cancellation notches verified for 12-pole cascade!

[6] Testing Resonant Peaks with Feedback...
  Detected resonant inter-notch peak formants with Feedback @ 0.85:
    Resonant Peak:    70.3 Hz  (+13.7 dB)
    Resonant Peak:   263.7 Hz  (+13.0 dB)
    Resonant Peak:   673.8 Hz  (+ 9.4 dB)
    Resonant Peak:   802.7 Hz  (+ 9.0 dB)

[7] Comparative Benchmark: Mutante vs Arturia Phaser BI-TRON...
  Arturia Min LFO Rate: 0.2 Hz (Cut off!)
  Mutante Min LFO Rate: 0.1 Hz (Hardware Authentic 10s period!)
  Arturia Default Poles: 2 (Digital 2-pole shortcut)
  Mutante Default Poles: 6 Poles (Hardware Authentic Musitronics RC Ladder)

======================================================================
ALL TESTS PASSED! MUTANTE EMULATION VALIDATED WITH PHYSICAL ACCURACY.
======================================================================
```

### 3.1 Mutante vs Arturia Phaser BI-TRON Comparison

| Specification | Hardware Mu-Tron Bi-Phase | Mutante (Jardim Sonoro) | Arturia Phaser BI-TRON |
| :--- | :--- | :--- | :--- |
| **Minimum Sweep Rate** | $0.1\text{ Hz}$ ($10\text{ s}$ period) | **$0.1\text{ Hz}$ ($10\text{ s}$ period)** | $0.2\text{ Hz}$ (Truncated) |
| **Default Allpass Poles** | 6 Poles (3 notches) | **6 Poles (3 notches)** | 2 Poles (1 notch) |
| **CdS Photo-Kinetics** | Dual-phase decay ($\tau \approx 45/200\text{ms}$) | **Dual-phase decay modeled** | Simplified envelope |
| **Parallel Bias Shunt** | $220\text{ k}\Omega$ metal film | **$220\text{ k}\Omega$ modeled** | Unmodeled |
| **Bypass Transparency** | Bit-clean wire bypass | **$0.0000000000$ diff** | Varies by state |
| **Real-time Safety** | Zero allocations | **Zero allocations (`rt-paranoid`)** | Internal buffers |

---

## 🎛️ 4. Included Factory Presets

Mutante comes preloaded with 7 meticulously calibrated factory presets covering the most celebrated hardware applications:

1. **Classic 6-Stage Stereo Swirl** *(Default)*: Dual 6-pole swirl with Sweep B tracking Sweep Gen 1 in reverse sync ($180^\circ$) for a lush, wide stereo panorama.
2. **12-Stage Series Deep Phase**: Switch S1 routed to `Out A`, cascading both channels into a monolithic 12-pole engine with 6 deep moving notches.
3. **Dual-Speed Asynchronous Swirl**: Independent prime-frequency LFOs ($0.22\text{ Hz}$ and $0.37\text{ Hz}$) yielding ever-evolving, non-repeating psychoacoustic modulation.
4. **Fast Rotary Leslie**: High-speed sweep ($5.8\text{ Hz}$ drum / $6.4\text{ Hz}$ horn) emulating the Doppler spin and cabinet acceleration of a dual-rotor Leslie.
5. **Envelope Auto-Wah Phaser**: Dynamic envelope follower sweeps the optical pods with resonant vocal peaking ($+13.7\text{ dB}$ formants).
6. **Square Chopper Trem-Phase**: Bipolar square wave modulation alternating between two discrete phase states for rhythmic stutter phasing.
7. **Subtle Stereo Widener**: Gentle 2-pole stereo enhancement that collapses cleanly to mono without comb-filtering nulls.

---

## 📦 5. Building & Installation

### Requirements
- **Rust Toolchain:** Stable 1.85+ with `cargo`
- **Truce CLI:** Truce 6.3 (`cargo install truce`)

### Build Bundles
To compile release bundles for CLAP, VST3, and AU:
```bash
cargo truce build
```

### Install Plugins into DAW Directories
To install directly into user plugin directories (`~/Library/Audio/Plug-Ins/`):
```bash
cargo truce install --user
```

Installed locations:
- **VST3:** `~/Library/Audio/Plug-Ins/VST3/Mutante.vst3` (Categorized under **Jardim Sonoro**)
- **CLAP:** `~/Library/Audio/Plug-Ins/CLAP/Mutante.clap`
- **AU v2:** `~/Library/Audio/Plug-Ins/Components/Mutante.component`
- **Presets:** `~/Library/Audio/Presets/Jardim Sonoro/Mutante/`

### Run Standalone
```bash
cargo truce run
```

---

## 📜 6. Authors & Acknowledgments

- **Concept, Engineering & DSP Architecture:** Jardim Sonoro / nagualcode
- **Original Hardware Design:** Mike Beigel & Musitronics Corporation (1974)
- **Audio Framework:** [Truce Audio Framework](https://github.com/truce-audio/truce) v6.3

---
*Mutante is an independent engineering project inspired by vintage hardware. Mu-Tron is a registered trademark of its respective owner.*
