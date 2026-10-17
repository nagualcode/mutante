# Mutante · Mu-Tron Bi-Phase Emulation

> **Jardim Sonoro** · CLAP / VST3 / AU · macOS (Apple Silicon) · Rust + [Truce](https://github.com/truce-audio/truce) 6.3
> A component-level physical model of the 1974 Musitronics Mu-Tron Bi-Phase dual optical phasor.

[![release](https://img.shields.io/github/v/release/nagualcode/mutante?color=brightgreen)](https://github.com/nagualcode/mutante/releases)
[![pluginval](https://img.shields.io/badge/pluginval-strictness%2010%20PASS-brightgreen.svg)](#8-verification--bench-measurements)
[![auval](https://img.shields.io/badge/auval-PASS-brightgreen.svg)](#8-verification--bench-measurements)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

![Mutante editor](docs/mutante-ui.png)

The **Mu-Tron Bi-Phase** (designed by Mike Beigel at Musitronics Corp., 1974) is two independent six-stage all-pass phase shifters whose variable resistors are cadmium-sulphide (CdS) photocells illuminated by a shared LED inside a sealed "photo pod," driven by two independent sweep generators, an optical C-100 expression pedal, an LED-lit photocell footswitch bypass, and a three-way routing switch that runs the two phasors in mono-to-stereo parallel (**In A**), true series (**Out A**), or independent stereo (**In B**). Lee "Scratch" Perry at the Black Ark, Mad Professor at Ariwa, Billy Corgan, and countless dub, funk, and krautrock engineers built records around its unmistakable deep, vocal sweep and bass-preserving warmth.

**Mutante** models the hardware stage by stage from the original ARP/Musitronics service schematics (`77204 BD 1`, `BD 2`, `77206`, `77207`) stored in this repository—preserving every coupling capacitor, potentiometer loading interaction, diode knee, op-amp rail limit, and per-cell photoconductive time constant.

**Contents:** [Install](#1-install) · [Uninstall](#2-uninstall) · [Sonic Research & Reverse-Engineering](#3-sonic-research--reverse-engineering) · [Complete Signal & Control Chain](#4-complete-signal--control-chain) · [Photocell Calibration](#5-photocell-calibration) · [Codebase Architecture](#6-codebase-architecture) · [Controls & Presets](#7-controls-and-presets) · [Verification & Bench Measurements](#8-verification--bench-measurements) · [Building from Source](#9-building-from-source)

---

## 1. Install

### From the release (recommended)

1. Download `Mutante-macOS-v0.4.0.zip` from the [Releases page](https://github.com/nagualcode/mutante/releases) and unzip it.
2. In Terminal, run the installer from the unzipped folder:
   ```bash
   cd ~/Downloads/Mutante-macOS-v0.4.0
   ./install.sh
   ```
   It copies the plug-ins into your user plug-in folders (no admin password required) and clears the macOS Gatekeeper quarantine flag:

   | Format | Installed to |
   | :-- | :-- |
   | VST3 | `~/Library/Audio/Plug-Ins/VST3/Mutante.vst3` |
   | CLAP | `~/Library/Audio/Plug-Ins/CLAP/Mutante.clap` |
   | AU | `~/Library/Audio/Plug-Ins/Components/Mutante.component` |
   | Presets | `~/Library/Audio/Presets/Jardim Sonoro/Mutante/` |

3. Restart your DAW. Mutante appears under **Jardim Sonoro**.

### Manual install

Copy the three bundles into the folders above, then clear the quarantine attribute (the binaries are ad-hoc signed, not notarized):

```bash
xattr -cr ~/Library/Audio/Plug-Ins/VST3/Mutante.vst3 \
          ~/Library/Audio/Plug-Ins/CLAP/Mutante.clap \
          ~/Library/Audio/Plug-Ins/Components/Mutante.component
```

If Logic Pro or GarageBand does not immediately list the AU, run `killall -9 AudioComponentRegistrar` and reopen your DAW.

## 2. Uninstall

From the release folder (or the repository root):

```bash
./uninstall.sh             # removes the VST3, CLAP and AU bundles
./uninstall.sh --presets   # ...and the factory preset folder too
```

If you built from source, `cargo truce uninstall --user` does the same.

---

## 3. Sonic Research & Reverse-Engineering

Our goal was to reconstruct the physical circuit of the 1974 hardware rather than approximate a generic digital phaser. The research proceeded in five stages:

### Stage 1 · Primary Archival Sources

All primary documentation is preserved in this repository:

| Source | Engineering Data Extracted |
| :-- | :-- |
| `schematics5/Mu-Tron Bi-phase page 1–3.gif` | Factory ARP/Musitronics drawings `77204` (**BD 1**: audio I/O buffers, feedback summer, passive mixer, sweep generators 1 & 2), **BD 2** (two 6-stage all-pass ladders, LED current drivers, photo pods `PM3`/`PM4`), and `77206` (±15 V power supply) |
| `schematics1/AudioIO.png`, `Filters.png` | **Rev 2** of drawing `77204`, which explicitly labels the internal factory trimmers (`NOTCH DEPTH`, `FDBCK`, `LFO LOW SPEED ADJ`, `GEN AMP`) and confirms the `SHAPE` and `SWEEP` switch contact wiring |
| `schematics3/` | The **C-100** optical expression pedal (`77207`), using a servo-stabilized LED and `VTT9313` phototransistors |
| `Bi-Phase-Owners-Manual-Complete.pdf` | Original Musitronics Owner's Manual: operating levels (`2 V` RMS reference), control laws (`0.1–18 Hz` rate, `1–10` depth, `0–10` feedback), series/parallel patching diagrams, and behavioral notes on fast-sweep compression and pedal depth scaling |
| `MutronClone2/`, `MutronClonage/` | Hardware restoration and cloning documentation: RCA `4558` dual op-amps on ±15 V rails, 10% polyester stage capacitors, and custom six-cell CdS optocoupler cans |

### Stage 2 · Net-by-Net Schematic Tracing

Every net on **BD 1** and **BD 2** was traced and cross-referenced against the Owner's Manual:

- **Why the Bi-Phase never thins out the bass (`C12` + `C14` dual wet high-pass):** Unlike textbook phasers that sum the raw all-pass output with the dry signal from DC up, the Bi-Phase AC-couples its wet path twice—first through `C12` (`0.47 µF`) into `J1-1` (`R133 ‖ R26`, cutting below `106–137 Hz`), and again through `C14` (`0.15 µF`) into the passive dry/wet summer (`R31 + R29 + R123`, cutting below `~44 Hz`). Sub-bass frequencies pass purely through the dry `R31` (`12 kΩ`) leg without phase cancellation.
- **Dynamic loading of `C12` by the `FEEDBACK` pot (`R133`):** At `J1-1`, coupling capacitor `C12` sees `R26` (`4.7 kΩ` to the `Z9B` virtual ground) in parallel with the `R133` (`10 kΩ`) feedback pot. As the `FEEDBACK` wiper advances toward `10`, it is loaded by `R7` (`6.2 kΩ`) + `R6` (`4.7 kΩ`), pulling the effective resistance of the `R133` branch down from `10 kΩ` to `5.22 kΩ` and shifting the `C12` high-pass corner from `105.8 Hz` up to `136.8 Hz`.
- **The panel's "Sine" is actually an integrator triangle:** Tracing the Rev 2 schematic confirms that the `SHAPE` switch in its `"Sine"` position takes the raw triangle wave straight from the integrator op-amp (`Z13B` via `R75` on Gen 1; `Z14B` on Gen 2). The smooth, sinusoidal turnaround heard in the audio path is produced entirely by the non-linear `1N4148` diode knee in the LED driver combined with the exponential photoconductive law of the CdS cells.
- **Gen 1 vs. Gen 2 "Square" wave circuit asymmetry:** On **BD 1**, Generator 1's square output takes the `Z13A` comparator through `R76` (`120 kΩ`) into `C36` (`0.47 µF`) loaded by `R77 + R128` ($\tau \approx 18.6\text{ ms}$, attenuated to `4.3 V` peak). Generator 2 takes `Z14A` through `R92` (`10 kΩ`) into `C35` (`2.2 µF`) ($\tau \approx 14.4\text{ ms}$, unattenuated `8.5 V` peak), driving the LED much harder into both extremes of the sweep.
- **The `Z7A/Q1` LED driver diode knee (`CR9`):** The feedback loop around `Z7A` places `R99` (`10 kΩ`) + `CR9` (`1N4148` silicon diode) in parallel with `R100` (`68 kΩ`). Below the diode turn-on threshold (~`0.5 V`), the driver gain is $1 + R_{98}/R_{100} \approx 4.2\times$; once `CR9` conducts, the incremental gain rises to $1 + R_{98}/(R_{100} \parallel R_{99}) \approx 26\times$. Combined with the CdS cell law $G \propto I^\gamma$, this expanding $V \to I$ curve linearizes the perceptual sweep across ~5 octaves.
- **`C4` inside the feedback loop:** `Z8B` is an inverting/non-inverting summer with `R4 = R5 = 4.7 kΩ` and `C4` (`0.001 µF`) across `R5`, which puts a `33.9 kHz` pole inside the feedback loop and slightly softens the resonant peaks at the top of the sweep.
- **Optical footswitch (`PM1` / `PM2`):** The bypass cell is lit by its own internal LED, fed through `R30` (`2.7 kΩ`) from the footswitch line `J6-3`; `CR18` (`MV5054-2`, via `R28 1.2 kΩ`) is only the panel indicator. `R30` is outside the audio path.
- **Sweep Sync "Rev" (`S7`, `Z15B`):** A pure unity inverter (`R106 = R107 = 39.2 kΩ 1 %`, `R108` bias only), so Phasor B receives exactly `−CV` from its `DEPTH B` wiper.
- **Six individual CdS cells per Photo Pod (`PM3` / `PM4`):** Each pod houses a single LED surrounded by six CdS cells. Because illumination coupling varies slightly across the six cells (`±12%`) and CdS carrier recombination slows down at lower carrier densities, dimmer cells darken more slowly than brighter cells during the downward sweep—creating dynamic inter-stage phase dispersion.

---

## 4. Complete Signal & Control Chain

### 4.1. Audio Signal Path (Per Channel, Modelled in Volts: `0 dBFS = 2 Vp`)

```text
Input Jack ──► C1 (0.047µF) / R1 (390kΩ) ──► Z8A Buffer (4558 Clip ±13.5V)
                                                │
                 ┌──────────────────────────────┴─────────────────────────── R31 (12kΩ Dry Leg) ──────────────────────────┐
                 ▼                                                                                                        │
              R4 (4.7kΩ)                                                                                                  │
                 │                                                                                                        ▼
                 ▼                                                                                                  Passive Mixer
          Z8B (−1/+2) + C4/R5 ───► 6× TPT All-Pass Ladder ──► C12 (0.47µF) ──┬──► R26 (4.7kΩ) ──► Z9B (−1, C13/R27) ──► PM1 ──► R123 ──► C14 (0.15µF) ──► R29 (4.7kΩ) ──► Z9A Out Buffer
          (Delay-Free Solve)       (C6–C11 6.8nF, CdS ‖ 220kΩ)   (J1-1)      │                    (33.9kHz LP + Clip)  (Optical) (Trim)  (44Hz Wet HP)
                 ▲                                                           │
                 └────── R6 (4.7kΩ) ‖ C5 (3.3nF) ◄── R7 (6.2kΩ) ◄── R133 Feedback Pot
```

1. **Input Coupling & Buffer (`Z8A` / `Z10A`):** `C1` (`0.047 µF`) and `R1` (`390 kΩ`) form an `8.68 Hz` high-pass filter into a unity-gain RCA `4558` non-inverting buffer with soft-knee rail limiting at `±13.5 V` (`opamp_clip`).
2. **Feedback Summing Amplifier (`Z8B` / `Z10B`):** The dry signal enters the inverting input via `R4 = R5 = 4.7 kΩ` (gain `-1`, with `C4 = 0.001 µF` across `R5` adding a `33.9 kHz` pole), while the feedback signal from `R133` enters the non-inverting input via the `R7` (`6.2 kΩ`) / `R6` (`4.7 kΩ`) divider shunted by `C5` (`0.0033 µF`, `19 kHz` HF roll-off) with non-inverting gain `+2`. Because the 6 all-pass stages, `C12`, `C5`, and `C4` have instantaneous linear feedthrough, **Mutante** solves the algebraic loop analytically at every sample with **zero unit delay** before applying `Z8B`'s `4558` rail limiter.
3. **Six-Stage Op-Amp All-Pass Ladder (`Z1–Z3` / `Z4–Z6`):** Six cascaded first-order all-pass stages (`R_bridge = 4.7 kΩ`, `C_stage = 0.0068 µF` with measured 10% polyester tolerances). Each stage's grounded resistor is its CdS cell in parallel with a `220 kΩ` fixed shunt (`R10, R13, …`) that clamps the darkest stage cutoff to `~106 Hz`. Discretized via Topology-Preserving Transform (`one_pole_g_norm`), pre-warped per stage, and run at **2× the host rate** (see below).
4. **Wet AC Coupling (`C12`) & Inverting Amplifier (`Z9B` / `Z11B`):** `C12` (`0.47 µF`) blocks DC and low bass from entering both the feedback loop and the wet mixer leg, with its cutoff dynamically shifting from `105.8 Hz` (`FB = 0`) to `136.8 Hz` (`FB = 10`) as `R133` loads node `J1-1`. `Z9B` re-inverts the wet signal (`R26 = R27 = 4.7 kΩ`) with `C13` (`0.001 µF`, `33.9 kHz` low-pass) and `4558` rail saturation.
5. **Optical Bypass (`PM1` / `PM2`) & Passive Mixer (`Z9A` / `Z11A`):** The footswitch lights the internal LED of `PM1` through `R30`; its CdS cell (`~500 Ω` lit → `8 MΩ` dark, fast on, slow fade off) sits in series with the `R123` (`NOTCH DEPTH`) factory trimmer, `C14` (`0.15 µF`), and `R29` (`4.7 kΩ`). This wet leg sums passively against the `R31` (`12 kΩ`) dry leg into output buffer `Z9A`.
6. **2× Oversampling:** The `4558` rail limiters sit inside the `Z8B` feedback loop, so they cannot be anti-aliased on their own. Both phasors—and the plug-in's `Mix` control, so dry and wet see identical filtering—run at twice the host rate between 10-coefficient polyphase IIR half-band filters (flat within `±0.05 dB` to `19 kHz`, images and aliases below `−90 dB`, no reported latency).

### 4.2. Control & Optocoupler Chain

```text
Sweep Gen 1 (Tri ±5V / Sq ±4.3V, τ=18.6ms) ──┐
Sweep Gen 2 (Tri ±5V / Sq ±8.5V, τ=14.4ms) ──┼─► SWEEP Switch ──► DEPTH Pot (10kΩ + 560Ω Floor) ──► Z7A/Q1 LED Driver (CR9 1N4148 Knee) ──► 6× CdS Cells (PM3/PM4)
C-100 Optical Pedal / Envelope Follower    ──┘
```

- **Depth Floor (`R96` / `R105 = 560 Ω`):** The bottom of the `10 kΩ` `DEPTH` potentiometer is held above ground by a `560 Ω` resistor, so setting `DEPTH = 1` still passes `560 / 10560 ≈ 5.3%` of the sweep voltage.
- **Pedal + Depth Topology:** Because the `SWEEP` source switch (`GEN / PED`) precedes the `DEPTH` potentiometer on **BD 1**, the `DEPTH` knob scales the manual/envelope pedal range—matching the procedure on page 5 of the Owner's Manual.
- **Log-Domain CdS Carrier Recombination (`src/optocoupler.rs`):** Each of the six cells in `PhotoPod` integrates its light-driven conductance in the log domain (`ln_g`). Brightening follows a fast `3.5 ms` time constant, while darkening is never faster than `25 ms` and slows exponentially up to `1.5 s` as carrier density drops (a full-range step brightens halfway, in octaves, in `~5 ms` and darkens halfway in `~21 ms`). The cell law ($R = 25.4\text{ kΩ} \cdot I_{\text{mA}}^{-0.945}$, pod B `26.8 kΩ`) sweeps each stage from the `220 kΩ`-limited `~110 Hz` floor to `~12 kHz` at the `16 mA` LED maximum. At fast LFO speeds (`8–18 Hz`), the cells cannot fully recombine between cycles and hang on the bright side, naturally compressing the sweep width.

---

## 5. Photocell Calibration

The service schematics fix everything around the photocells (`220 kΩ` shunts, `0.0068 µF` stage caps, the `Z7A/Q1` driver), but not the cells themselves: the original six-cell pods are unobtainable and no datasheet survives. Two numbers set the whole sweep—the cell resistance at `1 mA` and the photoconductive exponent $\gamma$—so they were calibrated against a reference control-to-cutoff curve gathered during local benchmarking (`~110 Hz` floor, `~1.2 / 3.5 / 7.8 kHz` at a quarter / half / three-quarter pedal, `~12.2 kHz` at full toe).

With the driver's own diode-knee $V \to I$ curve left untouched, a single fit of `R@1mA = 25.4 kΩ` and $\gamma = 0.945$ reproduces that curve to within `~115 cents` rms across the full range—evidence that the schematic driver model and the CdS power law have the right shape. Earlier versions (`55 kΩ`, $\gamma = 0.9$) topped out around `5.3 kHz`, about an octave and a quarter low in the upper half of the sweep.

| Pedal (Depth 10) | Stage cutoff v0.3 | Stage cutoff v0.4 | Reference curve |
| :-- | --: | --: | --: |
| `0.00` (heel) | `112 Hz` | `112 Hz` | `110 Hz` |
| `0.25` | `634 Hz` | `1253 Hz` | `1170 Hz` |
| `0.50` | `1453 Hz` | `3185 Hz` | `3493 Hz` |
| `0.75` | `3213 Hz` | `7522 Hz` | `7765 Hz` |
| `1.00` (toe) | `5333 Hz` | `12917 Hz` | `12200 Hz` |

---

## 6. Codebase Architecture

| File | Circuit Block |
| :-- | :-- |
| `src/circuit.rs` | Transcribed component values with schematic designators (`R1–R138`, `C1–C36`), calibrated CdS cell law, dynamic `J1-1` feedback pot loading (`j1_load_resistance`), RCA `4558` rail limiter (`opamp_clip`), and Padé `[3/2]` pre-warped TPT helpers (`one_pole_g_norm`) |
| `src/driver.rs` | `Z7A/Q1` (`Z7B/Q2`) LED current driver with the `R99 + CR9` (`1N4148`) feedback branch solved via Shockley diode bisection into a 512-point lookup table |
| `src/optocoupler.rs` | Six-cell `PhotoPod` (`PM3`/`PM4`) with per-cell log-domain CdS carrier recombination dynamics (`TAU_ON = 3.5 ms`, `TAU_OFF = 25 ms → 1.5 s`), plus the `PM1`/`PM2` optical footswitch cell (`PhotoSwitch`) |
| `src/allpass.rs` | Topology-preserving one-pole all-pass stage (`AllpassStage`), artifact-free under rapid resistance modulation |
| `src/phasor.rs` | Full `PhasorChannel`: input coupling, delay-free `Z8B` feedback loop solver (with `C4`), 6 all-pass stages, dynamic `C12` high-pass, `C5` feedback roll-off, `Z9B` inverting wet stage with `C13/R27`, `PM1` optical switch, and `R31/R29/R123/C14` passive mixer into `Z9A` |
| `src/oversample.rs` | 2× polyphase IIR half-band up/down-samplers (elliptic coefficients computed at start-up) around both phasors and the `Mix` stage |
| `src/sweep.rs` | `SweepGenerator` 1 and 2: integrator triangle ("Sine") and comparator RC-smoothed "Square" with distinct Gen 1 (`R76/C36`) and Gen 2 (`R92/C35`) networks |
| `src/envelope.rs` | `C-100` optical expression pedal and `Envelope` follower mode |
| `src/routing.rs` | Three-position `Phasor B Input` switch (`In A`, `Out A`, `In B`) |
| `src/ui.rs` | Hardware-inspired flat `egui` editor with live complex transfer-function response plot, photo-pod LED glow, and optical footswitch lamps |

---

## 7. Controls and Presets

| Section | Control | Notes |
| :-- | :-- | :-- |
| **Sweep Generator 1** | Rate `0.10–18.00 Hz`, Rate Control `Man`/`Ped`, Shape `∿`/`⊓`, Sync + division | On `Ped`, the C-100 pedal voltage controls the Gen 1 oscillator rate |
| **Sweep Gen 2** | Rate `0.10–18.00 Hz`, Shape `∿`/`⊓`, Sync + division | Gen 2 Square (`8.5 Vp`) drives the LED harder than Gen 1 Square (`4.3 Vp`) |
| **Phasor A** | Depth `1–10`, Feedback `0–10`, Sweep `Gen 1`/`Ped`, Footswitch | Manual's sweet spot for slow sweeps is Depth ≈ 8 |
| **Phasor B** | Depth `1–10`, Feedback `0–10`, Sweep `Gen 1`/`Gen 2`/`Ped`, Sweep Sync `Norm`/`Rev`, Footswitch | `Rev` inverts the sweep CV on Phasor B for counter-sweeping stereo imaging |
| **Phasor B Input** | `In A` / `Out A` / `In B` | Mono-to-stereo parallel / true 12-stage series / true stereo independent |
| **C-100 Pedal** | Heel–toe position `0.00–1.00`, Mode `Manual`/`Envelope`, `Env Sens` | Scales with each phasor's `Depth` knob when `Sweep = Ped` |
| **Studio** | `Input Level` (`-12..+24 dB`), `Mix` (`0..100%`), `Output Gain` (`-12..+12 dB`), `Bypass` | Push `Input Level` with high `Feedback` to drive the 4558 stages into warm rail saturation |

**Factory Presets:** `Stereo Phasing` (Manual Fig. B, default) · `Double Depth Phasing` (Manual Fig. A) · `Two-Speed Phasing` · `Dual Synched Phasing` (Manual Fig. C) · `Unsynchronized Swirl` · `Square Echo Repeat` · `Pedal Wah` · `Pseudo Ring Mod` · `Black Ark Drift`.

---

## 8. Verification & Bench Measurements

Run the unit test suite and the bench tool:

```bash
cargo test --release --lib                                  # 10 circuit, optocoupler & oversampler tests
cargo truce build --vst3 && python3 tools/measure_mutante.py
```

- **`pluginval` (strictness level 10):** PASS
- **`auval` (`aufx Muta JSon`):** PASS
- **Master bypass:** Bit-exact (`max |out - in| = 0.000e+00`)
- **Footswitch off (dry buffer path):** `-0.29 .. +0.06 dB` deviation from `30 Hz` to `18 kHz`
- **Aliasing:** a `5 kHz` sine at `+18 dB` input with `Feedback = 10` leaves inharmonic (aliased) energy at `−95 dB` re. the output (v0.3.0 at 1×: `−38 dB`)
- **CPU load:** `~1.5%` of a single Apple Silicon core at `48 kHz` stereo (2× oversampled, 12 CdS cells)

Static notches (Phasor A, `Sweep = Ped`, `Depth = 10`, `48 kHz`):

| Pedal | `FB = 0` notches | `FB = 10` notches · peaks |
| :-- | :-- | :-- |
| `0.00` | `56 Hz` (−5.0 dB) · `138 Hz` (−14.0) · `480 Hz` (−30.1) | `62` · `161` · `557 Hz` · peaks `220 Hz` (+6.0), `4980 Hz` (+10.8) |
| `0.25` | `384 Hz` (−27.3) · `1254 Hz` (−36.3) · `4421 Hz` (−37.2) | `434` · `1298` · `4834 Hz` · peaks `729 Hz` (+11.3), `2109 Hz` (+11.9) |
| `0.50` | `891 Hz` (−35.5) · `3103 Hz` (−37.8) · `10181 Hz` (−31.9) | `961` · `3214 Hz` · peaks `1793 Hz` (+11.9), `5121 Hz` (+10.8) |
| `0.75` | `2027 Hz` (−37.9) · `7131 Hz` (−34.9) · `19518 Hz` (−23.0) | `2162` · `7535 Hz` · peaks `4143 Hz` (+11.2), `11147 Hz` (+7.3) |
| `1.00` | `3507 Hz` (−37.6) · `11947 Hz` (−30.1) | `3800` · `12642 Hz` · peaks `7075 Hz` (+9.7), `17555 Hz` (+4.1) |

---

## 9. Building from Source

Requires Rust 1.92+ and the Truce CLI:

```bash
git clone https://github.com/nagualcode/mutante.git
cd mutante
cargo install cargo-truce
cargo truce build            # builds CLAP, VST3, and AU bundles in target/bundles/
cargo truce install --user   # installs CLAP / VST3 / AU and presets for your user
cargo truce run              # launches standalone app
cargo truce uninstall --user # removes installed user bundles
```

## License

Code: [MIT](LICENSE), © 2026 Jardim Sonoro / nagualcode. Original hardware design: Mike Beigel and Musitronics Corp. (1974). Mu-Tron is a trademark of its respective owner; Mutante is an independent emulation and is not affiliated with or endorsed by the trademark holder.
