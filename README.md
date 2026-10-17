# Mutante · Mu-Tron Bi-Phase emulation

> **Jardim Sonoro** · CLAP / VST3 / AU · macOS (Apple Silicon + Intel) · Rust + [Truce](https://github.com/truce-audio/truce) 6.3
> A component-level model of the 1974 Musitronics Mu-Tron Bi-Phase dual optical phasor.

[![release](https://img.shields.io/github/v/release/nagualcode/mutante?color=brightgreen)](https://github.com/nagualcode/mutante/releases)
[![pluginval](https://img.shields.io/badge/pluginval-strictness%2010%20PASS-brightgreen.svg)](#7-verification)
[![auval](https://img.shields.io/badge/auval-PASS-brightgreen.svg)](#7-verification)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

![Mutante editor](docs/mutante-ui.png)

The Bi-Phase (Mike Beigel and Musitronics, 1974) is two independent six-stage
phase shifters whose resistors are cadmium-sulphide photocells lit by an LED,
plus two sweep generators, an optical footswitch bypass and a switch that runs
the two phasors in parallel, in stereo, or in series. Lee "Scratch" Perry at
the Black Ark, Mad Professor at Ariwa and many others built records around it.

Mutante models the unit stage by stage from the ARP/Musitronics service
schematics in this repository, not from a generic phaser recipe.

**Contents:** [Install](#1-install) · [Uninstall](#2-uninstall) ·
[How we reverse-engineered it](#3-how-we-reverse-engineered-it) ·
[What the schematics say](#4-what-the-schematics-say) ·
[The model](#5-the-model) · [Controls & presets](#6-controls-and-presets) ·
[Verification](#7-verification) · [Building from source](#8-building-from-source)

---

## 1. Install

### From the release (recommended)

1. Download `Mutante-macOS-v0.2.0.zip` from the
   [Releases page](https://github.com/nagualcode/mutante/releases) and unzip it.
2. In Terminal, run the installer from the unzipped folder:
   ```bash
   cd ~/Downloads/Mutante-macOS-v0.2.0
   ./install.sh
   ```
   It copies the plug-ins into your user plug-in folders (no admin password)
   and clears the download quarantine flag:

   | Format | Installed to |
   | :-- | :-- |
   | VST3 | `~/Library/Audio/Plug-Ins/VST3/Mutante.vst3` |
   | CLAP | `~/Library/Audio/Plug-Ins/CLAP/Mutante.clap` |
   | AU | `~/Library/Audio/Plug-Ins/Components/Mutante.component` |
   | Presets | `~/Library/Audio/Presets/Jardim Sonoro/Mutante/` |

3. Restart your DAW. Mutante appears under **Jardim Sonoro**.

### Manual install

Copy the three bundles into the folders above, then remove the quarantine flag
(the binaries are ad-hoc signed, not notarized, so Gatekeeper would otherwise
block them):

```bash
xattr -cr ~/Library/Audio/Plug-Ins/VST3/Mutante.vst3 \
          ~/Library/Audio/Plug-Ins/CLAP/Mutante.clap \
          ~/Library/Audio/Plug-Ins/Components/Mutante.component
```

If Logic or GarageBand don't see the AU, run `killall -9 AudioComponentRegistrar`
and reopen the DAW.

## 2. Uninstall

From the release folder (or the repository root):

```bash
./uninstall.sh             # removes the VST3, CLAP and AU bundles
./uninstall.sh --presets   # ...and the factory preset folder too
```

Copies installed system-wide in `/Library/Audio/Plug-Ins` need
`sudo ./uninstall.sh`. To do it by hand, delete the three bundles listed in the
install table (and the preset folder if you like), then restart the DAW.
If you built from source, `cargo truce uninstall` does the same.

---

## 3. How we reverse-engineered it

The goal was to rebuild the circuit, not to approximate "a phaser that sounds
like a Bi-Phase". The work went in five steps.

### Step 1 · Gather everything

Everything used lives in this repository:

| Source | What it gave us |
| :-- | :-- |
| `schematics5/Mu-Tron Bi-phase page 1–3.gif` | ARP drawings 77204 (BD 1: audio I/O, feedback, mixer, sweep generators), BD 2 (phase-shift ladders, LED drivers), 77206 (power supply) |
| `schematics1/AudioIO.png` | Rev 2 of 77204, which **labels the trimmers** ("NOTCH DEPTH", "FDBCK", "LFO LOW SPEED ADJ", "GEN AMP") and confirms the shape-switch contacts |
| `schematics3/` | The C-100 control pedal (77207) |
| `Bi-Phase-Owners-Manual-Complete.pdf` | Panel layout, control ranges (rate 0.1–18 Hz, depth 1–10, feedback 0–10), the three routing modes, and how each control should *behave* |
| `MutronClone2/`, `MutronClonage/` | Restorers' and cloners' notes: RCA 4558 op-amps, 10 % carbon resistors and polyester caps in the originals, photocell ranges people used in clones |
| `phaser-bi-tron_Manual_1_1_EN.pdf` | Arturia's plug-in manual (feature list only; no circuit data) |

### Step 2 · Read the schematics net by net

The scanned drawings were enlarged region by region and every net was traced:
the input buffer, the Z8B summing amplifier, the six all-pass stages, the wet
coupling, the passive mixer, the footswitch photocell, the LED driver, and both
oscillators. Each finding was cross-checked against the owner's manual. A few
examples of how that played out:

- **The "sine" wave.** The rev 2 drawing labels the SHAPE switch contacts. The
  "sine" contact is fed from the integrator through R75 — it is a **triangle**.
  The "square" contact is the comparator through R76 into C36: an RC-smoothed
  square. Our first version had invented a diode sine shaper; it was removed.
- **"Out A" mode.** The S1 contact and the manual ("as if you had patched from
  Phasor Output A to the input of Phasor B") agree that Phasor B receives the
  *complete* output of Phasor A — dry and wet — so the series mode is two
  phasors in a row, not one 12-pole ladder.
- **The LED driver.** Z7A's feedback network has R99 and a 1N4148 (CR9) in
  parallel with R100. Solving it gives a V→I curve whose gain jumps from ~4 to
  ~26 once the diode conducts. That curve, not a hand-tuned exponent, is what
  maps the sweep generator onto the photocells.
- **The wet coupling.** C12 (0.47 µF into R133‖R26) and C14 (0.15 µF into the
  mixer) high-pass the wet leg around 100 Hz and 44 Hz. The lowest octaves pass
  dry, which is why the Bi-Phase never sounds thin in the bass.
- **Levels and limits.** Pot floor resistors (R96/R105: depth "1" still sweeps
  ~5 %), the feedback divider (maximum loop gain ≈ 0.86), and the 4558 rail
  limits all come straight from the drawings.

### Step 3 · Look at the reference plug-in

The Arturia Phaser BI-TRON installed on the development machine was loaded in
[pedalboard](https://github.com/spotify/pedalboard) to measure it as a black
box, and its binary was inspected. It turned out to be in unactivated demo
mode (silent output), so it could not serve as a measurement reference. Its
manual confirmed the panel features but no circuit details. The schematics
became the only reference — which is the better reference anyway.

### Step 4 · Prototype and calibrate

Before writing Rust, the control chain (sweep CV → LED driver → photocell →
stage cut-off) was prototyped in Python to choose the values the schematics
don't give: photocell sensitivity, and where the factory trimmers were set.
The criterion was the manual's behaviour — a full-depth sweep that reaches the
dark end of the cells, notches spread evenly over the musical range, and a
sweep that narrows at high rates. The values chosen are listed in
[Calibrated, not documented](#calibrated-not-documented).

### Step 5 · Build, measure, iterate

The circuit was then written in Rust (one module per circuit block), and the
built VST3 was measured with pedalboard: notch positions and depths across the
pedal sweep, feedback peaks, the sweep trajectory at slow and fast rates,
footswitch transparency, bypass, CPU. The same checks live on as unit tests
(`cargo test`) and as `tools/measure_mutante.py`.

---

## 4. What the schematics say

```
 jack ─C1/R1─ Z8A ─┬──────────────────────────────── R31 12k ──────────────┐
                   │                                                        │
                   └─R4─ Z8B (−1) ─ 6× all-pass ─C12─┬─ R26 ─ Z9B (−1) ─ PM1 ─ R123 ─ C14 ─ R29 ─┴─ Z9A ─ out
                         ▲ (+2)                      │
                         └── R6‖C5 ─ R7 ─ FEEDBACK pot (R133) ─┘
```

| Finding | Where | Effect on the sound |
| :-- | :-- | :-- |
| Each stage is an op-amp all-pass (4.7 k / 4.7 k, 0.0068 µF 10 % polyester) with the photocell ‖ 220 kΩ to ground | BD 2 | Three notches per phasor; the 220 kΩ shunt sets the bottom of the sweep (~110 Hz stage cut-off) |
| One LED per pod lights all six cells | PM3 / PM4 | Cells track together but not identically (light coupling, cap tolerance) → deep but organic notches |
| Wet leg AC-coupled twice: C12 into R133‖R26 (~106 Hz) and C14 into the mixer (~44 Hz) | BD 1 | The lowest octaves stay dry; bass is never hollowed out |
| Dry (R31 12 k) and wet (R29 + R123 + PM1) summed passively into a buffer; R123 is the **NOTCH DEPTH** trim | rev 2 | 50/50 mix, ~−37 dB notches, no level drop at the peaks |
| FEEDBACK pot taps J1-1 and drives Z8B's non-inverting input through R7/R6 | BD 1 | Positive feedback up to ×0.86: +12 dB peaks between notches, shallower notches — the manual's "intensity" |
| The footswitch lights a lamp + cell (PM1/PM2) in series with the wet leg | BD 1 | Bypass is an optical crossfade; the dry signal always flows through the buffers |
| LED driver: Z7A + Q1 with R98/R100 and **R99 + CR9 (1N4148)** in the feedback | BD 2 | Expanding V→I curve; with the CdS law it spreads the sweep evenly over ~5 octaves |
| The panel's "sine" is the integrator **triangle** (Z13B via R75, Z14B) | BD 1 | The roundness comes from the driver and photocells |
| "Square" is the comparator through R76 120 k / C36 0.47 µF (Gen 1) or R92 10 k / C35 2.2 µF (Gen 2) | BD 1 | Steps glide in ~15–19 ms; Gen 2's square is hotter than its triangle and slams both ends of the sweep |
| DEPTH pots sit on 560 Ω floor resistors | R96 / R105 | Depth "1" still sweeps ~5 % |
| "Out A" sends the **complete** Phasor A output into Phasor B | S1, manual p. 3 | Two phasors in series |
| The C-100 pedal uses VTT9313 phototransistors with a servo'd LED | 77207 | The pedal is fast; only the pods lag |

## 5. The model

| File | Circuit |
| :-- | :-- |
| `src/circuit.rs` | Every component value with its reference designator, the 4558 output-swing limiter, TPT one-pole helpers |
| `src/driver.rs` | Z7A/Q1 LED driver, solved with a Shockley diode model into a lookup table |
| `src/optocoupler.rs` | CdS cells: G ∝ I^0.9, integrated in the log-conductance domain; brightening τ ≈ 3.5 ms, darkening slows as the cell gets darker (22 ms → 1.5 s). PM1/PM2 bypass cell |
| `src/allpass.rs` | All-pass stage as x − 2·LP(x) on a topology-preserving one-pole, stable under fast resistance sweeps |
| `src/phasor.rs` | Full channel: input coupling, Z8B summer, six stages, C12, C5 feedback roll-off, Z9B, PM1, R31/R29 mixer with C14, op-amp rail limits. The feedback loop is solved each sample without a unit delay |
| `src/sweep.rs` | Triangle/square generators with each generator's own square RC network and level |
| `src/envelope.rs` | C-100 pedal and an envelope mode that plays the pedal from the input level |
| `src/routing.rs` | Phasor B Input switch |
| `src/ui.rs` | egui editor |

Signal levels are modelled in volts: 0 dBFS = 2 V peak at the input jack (the
manual's noise-spec reference). **Input Level** pushes the signal harder into
the 4558 stages; with feedback near 10 and a hot input you hear them reach the
rails.

### Calibrated, not documented

The schematics don't give everything. These were chosen from typical parts of
the era and checked by measurement:

- Photocell sensitivity: 55 kΩ / 58 kΩ at 1 mA (A / B pod), γ = 0.9, 5 MΩ dark.
  Six cells share one LED, so each sees much less light than a vactrol cell.
- Trimmer settings: R130 offset 4.73 V (a full-depth sweep just turns the LED
  off), R129 sense 550 Ω total (~16 mA peak), R123 within ±2 % of ideal.
- Photocell time constants and the dark-recovery slope.
- Per-unit tolerances (stage caps, light coupling): one fixed "unit" per phasor.

### Additions the hardware doesn't have

Host tempo sync for both generators, the envelope pedal mode, Input Level, Mix,
Output and a master bypass. In plugin context "Input A" is the mono sum of the
stereo input, and in "Out A" mode Output B goes to both channels (the manual's
one-amp setup).

## 6. Controls and presets

| Section | Control | Notes |
| :-- | :-- | :-- |
| Sweep Generator 1 | Rate 0.1–18 Hz, Rate Control Man/Ped, Shape ∿/⊓, Sync + division | On "Ped" the pedal sets the oscillator rate |
| Sweep Gen 2 | Rate, Shape, Sync + division | |
| Phasor A | Depth 1–10, Feedback 0–10, Sweep Gen 1/Ped, footswitch | The manual's sweet spot for slow sweeps is depth ≈ 8 |
| Phasor B | Depth, Feedback, Sweep Gen 1/Gen 2/Ped, Sweep Sync Norm/Rev, footswitch | Rev: B sweeps down while A sweeps up |
| Phasor B Input | In A / Out A / In B | Parallel-stereo / series / independent |
| C-100 Pedal | Heel–toe position, Manual/Envelope, Env Sens | |
| Studio | Input Level, Mix, Output, Bypass | |

The editor redraws the hardware's section layout in a flat, Ableton-style look.
The display on top shows each phasor's response computed from the pod state the
DSP reports, the small "photo pod" glows with the LED current, and the
footswitch LEDs follow the real optical fade. Knobs: drag (Shift for fine),
scroll, double-click to reset.

**Factory presets**, from the owner's manual figures plus a few studio
settings: Stereo Phasing (Fig. B, default) · Double Depth Phasing (Fig. A) ·
Two-Speed Phasing · Dual Synched Phasing (Fig. C) · Unsynchronized Swirl ·
Square Echo Repeat · Pedal Wah · Pseudo Ring Mod · Black Ark Drift.

## 7. Verification

```bash
cargo test --release --lib                                  # circuit regression tests
cargo truce build --vst3 && python3 tools/measure_mutante.py
```

Measured on the built VST3 (Phasor A on the pedal, depth 10):

| Pedal | Notches, feedback 0 | Feedback 10 |
| :-- | :-- | :-- |
| heel | 56 Hz (−5 dB), 138 Hz (−14), 475 Hz (−30) | peaks +7.5 / +11 dB |
| centre | 439 Hz (−29), 1.45 kHz (−37), 5.2 kHz (−39) | notches −12 dB, peaks +12 dB |
| toe | 1.5 kHz (−38), 5.2 kHz (−39), 14 kHz (−39) | |

- With a 0.5 Hz triangle at full depth the stage cut-off sweeps 114 Hz → 5.8 kHz;
  at 8 Hz the bottom rises because the cells can't darken in time.
- Footswitch off: within −0.3 / +0.1 dB from 30 Hz to 18 kHz.
- Master bypass is bit-exact. CPU ≈ 0.7 % of one Apple-silicon core.
- Release binaries: pluginval strictness 10 and auval pass on both arm64 and
  x86_64 (Rosetta).

## 8. Building from source

Requires Rust 1.92+ and the Truce CLI.

```bash
git clone https://github.com/nagualcode/mutante.git
cd mutante
cargo install cargo-truce
cargo truce build            # bundles in target/bundles/
cargo truce install --user   # install CLAP / VST3 / AU for your user
cargo truce run              # standalone app
cargo truce uninstall        # remove what install put in place
```

## License

Code: [MIT](LICENSE), © 2026 Jardim Sonoro / nagualcode. Original hardware
design: Mike Beigel and Musitronics Corp. (1974). Mu-Tron is a trademark of its
respective owner; Mutante is an independent emulation and is not affiliated
with or endorsed by the trademark holder.
