# Mutante · Mu-Tron Bi-Phase Physical Emulation

> **Jardim Sonoro** · Zero-Allocation DSP Audio Plugin  
> Component-level physical reconstruction of the legendary 1974 Musitronics Mu-Tron Bi-Phase Dual Optical Phaser

[![GitHub release](https://img.shields.io/github/v/release/nagualcode/mutante?color=brightgreen)](https://github.com/nagualcode/mutante/releases)
[![pluginval](https://img.shields.io/badge/pluginval-Strictness%20Level%205%20PASS-brightgreen.svg)](#-5-laboratory-verification--comparative-benchmarks)
[![auval](https://img.shields.io/badge/auval-AUv2%20%26%20AUv3%20PASS-brightgreen.svg)](#-5-laboratory-verification--comparative-benchmarks)
[![framework](https://img.shields.io/badge/framework-Truce%206.3%20Rust-orange.svg)](https://github.com/truce-audio/truce)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## 🏛️ 1. Overview & Heritage

The **Mu-Tron Bi-Phase**, designed by synthesis pioneer **Mike Beigel** and released by **Musitronics** in 1974, stands as one of the most revered analog phase-shifters in music history. Legendary for its three-dimensional, oceanic swirl, it defined the sound of dub pioneer **Lee "Scratch" Perry** (at the Black Ark Studio), **The Smashing Pumpkins** (Billy Corgan's iconic Siamese Dream lead guitar textures), **Pink Floyd**, and **George Clinton's Parliament-Funkadelic**.

Unlike solid-state JFET or OTA phasers (such as the MXR Phase 90 or EHX Small Stone), the Bi-Phase achieves its signature warmth and liquid depth through:
- **Two discrete 6-pole optical phaser engines** (Phasor A and Phasor B) utilizing custom light-tight Cadmium Sulfide (CdS) photoresistor pods.
- **Asymmetric photoconductive relaxation**: Photoresistors respond dynamically to light intensity with gradual carrier recombination, introducing organic lag and physical memory.
- **Versatile electro-mechanical routing**: 3-way input routing allowing independent dual-mono, stereo swirl, or cascading both engines into a monolithic **12-pole (6-notch) phase monster**.
- **Diode-bridge rounded LFOs**: Dual low-frequency sweep oscillators ranging from a glacial 0.1 Hz (10 seconds per cycle) up to 18.0 Hz, shaped with 1N914 diode arrays into smooth pseudo-sine or sharp square waveforms.
- **Optical expression and dynamic envelope modulation**: Controlled manually via the vintage C-100 optical rocker pedal or dynamic audio envelope following.

**Mutante** is a zero-allocation, mathematically rigorous DSP emulation built in **Rust** using the **Truce 6.3** framework, published under the **Jardim Sonoro** catalog.

---

## 🔬 2. The Research & Forensic Engineering Process

Our objective was not to create an approximation or black-box curve fit, but to perform a component-level physical reconstruction of the 1974 Musitronics hardware.

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

### 2.1 Archival Schematics & Circuit Mining
Our engineering investigation analyzed original factory documentation, including:
1. **Musitronics Audio I/O Schematic (`schematics1/AudioIO.png`):** Disclosed the input buffer amplifiers, the S1 routing switch topology, and the summing amplifier nodes.
2. **Filters Board Schematic (`schematics1/Filters.png` & `schematics5/`):** Revealed the active 1st-order op-amp allpass topologies, the inverting feedback loop node, and the exact $RC$ values.
3. **Power Supply & Driver Schematics (`schematics1/Powersupply.png`):** Uncovered the discrete NPN current driver stages (2N5088) providing drive current to the central lamp/LED pods.
4. **C-100 Foot Controller Schematics (`schematics3/`):** Documented the optical rocker pedal circuit and envelope follower rectifier circuitry.
5. **Cloning & Restoration Archives (`MutronClone2/`):** Bench measurements and component specs from vintage hardware restorations.

### 2.2 Key Circuit Secrets Uncovered
Through this forensic investigation, four critical hardware characteristics were identified:
- **Parallel Shunt Resistors ($220\text{ k}\Omega$):** Each CdS photoresistor is wired in parallel with a fixed $220\text{ k}\Omega$ resistor. This clamps the maximum dark resistance, preventing the allpass break frequency from collapsing into DC and guaranteeing predictable phase sweep bounds ($40\text{ Hz} - 18\text{ kHz}$).
- **Poly Timing Capacitors ($6.8\text{ nF}$):** High-precision polypropylene capacitors establish the nominal break frequencies across the ladder.
- **Inverting Summing Feedback with Op-Amp Rail Saturation:** Feedback is mixed inversely at summing amplifier $Z8\text{B} / Z9\text{B}$. Under high feedback, the op-amp softly saturates before hitting the $\pm 15\text{V}$ power rails, producing creamy harmonic formants without harsh digital clipping.
- **Diode-Bridge Pseudo-Sine Waveshaping:** LFO sine generation is achieved not with pure mathematical sines, but by passing an integrator triangle wave through four anti-parallel 1N914 silicon diodes (CR2–CR5), creating subtle soft-top rounding with enriched harmonic overtones.

---

## ⚙️ 3. Physical & Mathematical DSP Modeling

### 3.1 Cadmium Sulfide (CdS) Optocoupler Kinetics (`src/optocoupler.rs`)
In the Musitronics PM3/PM4 pods, a central light source drives six peripheral CdS photoresistors. CdS is an inorganic semiconductor with deep trap energy levels in its crystal lattice.

1. **Photoconductive Power-Law Resistance:**
   $$R(L) = R_{\text{light}} + \frac{R_{\text{dark}} - R_{\text{light}}}{(1 + k \cdot L)^\gamma}$$
   - $R_{\text{dark}} = 2.5\text{ M}\Omega$ (un-illuminated dark resistance)
   - $R_{\text{light}} = 350\,\Omega$ (maximum saturated illumination)
   - $\gamma = 0.82$ (photoconductive empirical exponent)

2. **Asymmetric Dual-Phase Ballistics:**
   - **Rising Light (Photo-excitation):** Direct valence-to-conduction band transition occurs rapidly:
     $$\tau_{\text{rise}} \approx 10\text{ ms}$$
   - **Decaying Light (Recombination & Trap Relaxation):** Free electrons recombine in two distinct regimes:
     - **Fast component (70%):** Direct carrier recombination ($\tau_{\text{fast}} \approx 45\text{ ms}$).
     - **Slow persistent component (30%):** Thermally assisted release of trapped carriers from lattice defects ($\tau_{\text{slow}} \approx 200\text{ ms}$).

3. **Effective Shunt Network & Cutoff Frequency:**
   $$R_{\text{eff}} = \frac{R_{\text{ldr}} \cdot R_{\text{bias}}}{R_{\text{ldr}} + R_{\text{bias}}} \quad \left(R_{\text{bias}} = 220\text{ k}\Omega\right)$$
   $$f_c = \frac{1}{2\pi \cdot R_{\text{eff}} \cdot C} \quad \left(C = 6.8\text{ nF}\right)$$

4. **Organic Stage Dispersion:**
   In real hardware, subtle geometric differences in photoresistor positioning yield slight component tolerances ($\pm 1.5\% - 3.1\%$). Mutante models these empirical offsets (`DISPERSION_A` and `DISPERSION_B`), creating organic phase cancellation nulls that avoid synthetic comb sterility.

### 3.2 1st-Order Allpass Filter Engine (`src/allpass.rs`)
To ensure unconditional numerical stability under rapid modulation, each allpass filter is mapped via the Bilinear Transform into Transposed Direct Form II:

$$H(z) = \frac{a - z^{-1}}{1 - a z^{-1}} \quad \text{where} \quad a = \frac{\tan(\pi f_c / f_s) - 1}{\tan(\pi f_c / f_s) + 1}$$

- **Phase Response:** $\phi(\omega) = -2 \arctan\left(\frac{\omega}{\omega_c}\right)$ per stage.
- **Cancellation Nulls:** For $N = 6$ stages, phase shift reaches $-180^\circ$, $-540^\circ$, and $-900^\circ$, yielding **exactly 3 cancellation notches** in the combined wet/dry output. For $N = 12$ stages, it yields **exactly 6 cancellation notches**.
- **Modulation Safety:** Filter state registers maintain energy continuity across parameter changes without audible pops or zipper noise.

### 3.3 Rail Soft-Saturation Feedback (`src/phasor.rs`)
Inverting feedback is injected into the input summing node:
$$x_{\text{sum}}[n] = x_{\text{in}}[n] - \beta \cdot y_{\text{fb}}[n]$$
$$y_{\text{fb}}[n] = \tanh\left(1.2 \cdot y_{\text{allpass}}[n]\right)$$
Where $\beta \in [0.0, 0.94]$. The non-linear $\tanh$ soft-clipper emulates op-amp rail compression, focusing the resonant peak into a singing, vocal formant.

---

## 🎛️ 4. User Manual & Operating Guide

### Front Panel Overview

| Section | Parameter | Range / Options | Description |
| :--- | :--- | :--- | :--- |
| **Sweep Gen 1** | **Rate** | `0.1 Hz – 18.0 Hz` | Master LFO 1 modulation speed (down to 10s period). |
| | **Sync** | `Off / On` | Engages DAW host tempo-sync subdivisions. |
| | **Synced Rate**| `4 Bars – 1/16T` | Musical beat divisions (straight, dotted, triplets). |
| | **Shape** | `Sine / Square` | Diode-shaped pseudo-sine or bipolar square wave. |
| | **Rate Ctrl** | `Internal / Pedal` | Switches rate modulation to manual pedal control. |
| **Sweep Gen 2** | **Rate** | `0.1 Hz – 18.0 Hz` | Independent LFO 2 modulation speed. |
| | **Sync / Div**| `Off / On` & Divs | Host tempo sync controls for Sweep Gen 2. |
| | **Shape** | `Sine / Square` | Diode-shaped pseudo-sine or square wave. |
| **Phasor A** | **Depth** | `0.0 – 10.0 (0–100%)`| Sweep excursion width around center break frequency. |
| | **Feedback** | `0.0 – 10.0 (0–100%)`| Regenerative feedback resonance / notch sharpness. |
| | **Sweep Source**| `Gen 1 / Pedal` | Selects Sweep Gen 1 or Expression/Envelope. |
| | **Poles** | `6 (Classic), 2..12` | Stage pole count (6 poles default for 3 notches). |
| | **Bypass** | `Off / On` | Individual channel hard-bypass. |
| **Phasor B** | **Depth** | `0.0 – 10.0 (0–100%)`| Sweep excursion width for Phasor B. |
| | **Feedback** | `0.0 – 10.0 (0–100%)`| Regenerative feedback resonance for Phasor B. |
| | **Sweep Source**| `Gen 1 / Gen 2 / Pedal`| Selects modulation source for Phasor B. |
| | **Sweep Sync** | `Normal / Reverse` | When driven by Gen 1: $0^\circ$ or $180^\circ$ reverse phase. |
| | **Poles** | `6 (Classic), 2..12` | Stage pole count for Phasor B. |
| **Routing** | **Switch S1** | 4 Positions | Input routing topology matrix (see below). |
| **Pedal / Env**| **Pedal Mode**| `Manual / Envelope` | C-100 optical pedal or dynamic envelope follower. |
| | **Env Sens** | `0.1 – 10.0` | Dynamic envelope sensitivity to input signal peaks. |
| **Master** | **Analog Drive**| `0.0 – 100%` | Discrete input stage analog warmth and saturation. |
| | **Mix** | `0.0 – 100%` | Dry/Wet balance (100% wet = maximum comb notches). |
| | **Output Gain**| `-12 dB – +12 dB` | Clean makeup gain staging. |
| | **Bypass** | `Off / On` | Bit-clean wire bypass ($0.0000000000$ deviation). |

### Hardware Routing Modes (Switch S1)
- **`In B (Stereo / Dual Mono)`:** Channels A and B operate completely independently. Left input drives Phasor A; Right input drives Phasor B. Ideal for stereo stems or two distinct instruments.
- **`In A (Mono -> Stereo)`:** Left input drives both Phasor A and Phasor B simultaneously. When Sweep B is set to **Reverse Sync**, this produces the signature wide, swirling binaural stereo panorama.
- **`Out A (12-Stage Series Cascade)`:** The processed output of Phasor A is fed directly into the input of Phasor B. This cascades both 6-stage ladders into a colossal **12-pole phaser** with 6 moving cancellation nulls.
- **`Stereo Parallel`:** Dual parallel routing for studio summing and parallel processing.

---

## 📊 5. Laboratory Verification & Comparative Benchmarks

Every release is validated via our automated audio measurement suite (`tools/measure_mutante.py`):

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

### Mutante vs Arturia Phaser BI-TRON

| Metric / Feature | Musitronics Hardware | Mutante (Jardim Sonoro) | Arturia Phaser BI-TRON |
| :--- | :--- | :--- | :--- |
| **Minimum LFO Rate** | $0.1\text{ Hz}$ ($10\text{ s}$ period) | **$0.1\text{ Hz}$ (Hardware authentic)** | $0.2\text{ Hz}$ (Prematurely truncated) |
| **Default Ladder Poles** | 6 Poles ($3\times$ notches) | **6 Poles (Classic factory)** | 2 Poles (1 notch digital shortcut) |
| **Photoconductive Lag** | Dual-phase CdS decay | **$\tau \approx 45/200\text{ms}$ modeled** | Simplified memoryless curve |
| **Parallel Shunt Bias** | $220\text{ k}\Omega$ metal film | **$220\text{ k}\Omega$ parallel modeled**| Unmodeled |
| **Hard Bypass** | 100% bit-clean wire | **$0.0000000000$ difference** | Varies by internal buffers |
| **Real-time Safety** | Zero memory allocation | **Zero allocations (`rt-paranoid`)** | Internal buffers |
| **Steinberg Pluginval** | Strictness 5 compliant | **Strictness 5: PASSED (0 errors)** | N/A |
| **Apple auval** | AU v2 / AU v3 compliant | **AU v2 / AU v3: PASS** | Closed proprietary |

---

## 🎨 6. Factory Presets Library

Mutante ships with 7 factory presets calibrated to match classic hardware settings:

1. **Classic 6-Stage Stereo Swirl** *(Default)*: Dual 6-pole swirl with Sweep B tracking Sweep Gen 1 in reverse sync ($180^\circ$) for a deep, wide stereo soundstage.
2. **12-Stage Series Deep Phase**: Switch S1 routed to `Out A`, cascading both channels into a monolithic 12-pole engine with 6 deep moving notches.
3. **Dual-Speed Asynchronous Swirl**: Independent prime-frequency LFOs ($0.22\text{ Hz}$ and $0.37\text{ Hz}$) creating non-repeating psychoacoustic motion.
4. **Fast Rotary Leslie**: High-speed sweep ($5.8\text{ Hz}$ drum / $6.4\text{ Hz}$ horn) emulating the Doppler spin and cabinet acceleration of a dual-rotor Leslie.
5. **Envelope Auto-Wah Phaser**: Dynamic envelope follower sweeps the optical pods with resonant vocal peaking ($+13.7\text{ dB}$ formants).
6. **Square Chopper Trem-Phase**: Bipolar square wave modulation alternating between two discrete phase states for rhythmic stutter phasing.
7. **Subtle Stereo Widener**: Gentle 2-pole stereo enhancement that collapses cleanly to mono without comb-filtering nulls.

---

## 🚀 7. Installation & Setup

### Pre-built Binaries (macOS)
Download the latest release archive (`Mutante-macOS-v0.1.0.zip`) from the [Releases](https://github.com/nagualcode/mutante/releases) page.

1. Unzip the archive.
2. Run the included install script:
   ```bash
   ./install.sh
   ```
   *Or manually copy the bundles:*
   - `Mutante.vst3` $\to$ `~/Library/Audio/Plug-Ins/VST3/`
   - `Mutante.clap` $\to$ `~/Library/Audio/Plug-Ins/CLAP/`
   - `Mutante.component` $\to$ `~/Library/Audio/Plug-Ins/Components/`
   - `Presets/*` $\to$ `~/Library/Audio/Presets/Jardim Sonoro/Mutante/`
3. Clear macOS quarantine (if prompted by Gatekeeper):
   ```bash
   xattr -cr ~/Library/Audio/Plug-Ins/VST3/Mutante.vst3
   xattr -cr ~/Library/Audio/Plug-Ins/CLAP/Mutante.clap
   xattr -cr ~/Library/Audio/Plug-Ins/Components/Mutante.component
   ```
4. Restart your DAW. Mutante will appear under the **Jardim Sonoro** vendor category.

### Building From Source
Ensure you have the Rust toolchain (1.85+) and the Truce CLI:
```bash
# Clone the repository
git clone https://github.com/nagualcode/mutante.git
cd mutante

# Install truce CLI
cargo install truce

# Build release bundles
cargo truce build

# Install directly into user plugin directories
cargo truce install --user

# Run standalone application
cargo truce run
```

---

## 📄 License & Attribution

- **Plugin Code & DSP Architecture:** Released under the [MIT License](LICENSE). Copyright (c) 2026 Jardim Sonoro / nagualcode.
- **Original Hardware Design:** Mike Beigel & Musitronics Corporation (1974).
- **Audio Framework:** [Truce Audio Framework](https://github.com/truce-audio/truce) v6.3.

*Mutante is an independent physical modeling project inspired by historical audio hardware. Mu-Tron is a registered trademark of its respective owner.*
