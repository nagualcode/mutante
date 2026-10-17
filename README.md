# Mutante · Mu-Tron Bi-Phase emulation

> **Jardim Sonoro** · CLAP / VST3 / AU · Rust + [Truce](https://github.com/truce-audio/truce) 6.3
> Component-level model of the 1974 Musitronics Mu-Tron Bi-Phase dual optical phasor.

[![pluginval](https://img.shields.io/badge/pluginval-strictness%2010%20PASS-brightgreen.svg)](#verification)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

The Bi-Phase (Mike Beigel, Musitronics, 1974) is two independent six-stage
phase shifters whose resistors are cadmium-sulphide photocells lit by an LED,
plus two sweep generators, an optical footswitch bypass and a switch that runs
the phasors in parallel, in stereo, or in series. Lee "Scratch" Perry, Mad
Professor and many others built records around it.

Mutante models the unit stage by stage from the ARP/Musitronics service
schematics in this repository, not from a generic phaser recipe.

---

## 1. What the schematics say

Sources: `schematics5/Mu-Tron Bi-phase page 1–3.gif` (ARP drawing 77204 "BD 1",
"BD 2" phase-shift/LED-driver board, 77206 power supply), the rev 2 redraw in
`schematics1/AudioIO.png` (which labels the trimmers), the C-100 pedal
(`schematics3/`), and the owner's manual (`Bi-Phase-Owners-Manual-Complete.pdf`).

```
 jack ─C1/R1─ Z8A ─┬──────────────────────────────── R31 12k ──────────────┐
                   │                                                        │
                   └─R4─ Z8B (−1) ─ 6× all-pass ─C12─┬─ R26 ─ Z9B (−1) ─ PM1 ─ R123 ─ C14 ─ R29 ─┴─ Z9A ─ out
                         ▲ (+2)                      │
                         └── R6‖C5 ─ R7 ─ FEEDBACK pot (R133) ─┘
```

| Finding | Where | Effect on the sound |
| :-- | :-- | :-- |
| Each stage is an op-amp all-pass (4.7 k / 4.7 k, 0.0068 µF 10 % polyester) with the photocell ‖ 220 kΩ to ground | BD 2 | Three notches per phasor; the 220 kΩ shunt sets the bottom of the sweep (~110 Hz stage cutoff) |
| One LED per pod lights all six cells | PM3 / PM4 | Cells track together but not identically (light coupling, cap tolerance) → notches are deep but not sterile |
| Wet leg is AC-coupled twice: C12 0.47 µF into R133‖R26 (~106 Hz) and C14 0.15 µF into the mixer (~44 Hz) | BD 1 | The lowest octaves stay dry; bass is never hollowed out |
| Dry (R31 12 k) and wet (R29 + R123 + PM1) summed passively into a buffer; R123 is the **NOTCH DEPTH** trim | rev 2 | 50/50 mix, ~−37 dB notches, no level drop at the peaks |
| FEEDBACK pot taps J1-1 and drives Z8B's non-inverting input through R7/R6 | BD 1 | Positive feedback up to ×0.86: +12 dB peaks between the notches, the notches get shallower ("intensity", as the manual says) |
| Footswitch lights a lamp+cell (PM1/PM2) in series with the wet leg | BD 1 | Bypass is an optical crossfade; the dry signal always flows through the buffers |
| LED driver: Z7A + Q1 with R98/R100 and **R99 + CR9 (1N4148)** in the feedback | BD 2 | Expanding V→I curve (gain ~4 → ~26 above the diode knee); with the CdS law it spreads the sweep evenly over ~5 octaves |
| What the panel calls "sine" is the integrator **triangle** (Z13B via R75, Z14B) | BD 1 | The roundness comes from the driver + photocells, not from a sine shaper |
| "Square" is the comparator through an RC: R76 120 k / C36 0.47 µF (Gen 1), R92 10 k / C35 2.2 µF (Gen 2) | BD 1 | Steps glide in ~15–19 ms; Gen 2's square is hotter than its triangle and slams both ends of the sweep |
| DEPTH pots sit on 560 Ω floor resistors | R96 / R105 | Depth "1" still sweeps ~5 % |
| "Out A" sends the **complete** Phasor A output (dry + wet) into Phasor B | S1, manual p. 3 | Two phasors in series, not a 12-pole ladder with one mix |
| C-100 pedal uses VTT9313 phototransistors with a servo'd LED | 77207 | The pedal itself is fast; only the pods lag |

## 2. The model

| File | Circuit |
| :-- | :-- |
| `src/circuit.rs` | Every component value, with reference designators, plus the 4558 output-swing clipper and TPT one-pole helpers |
| `src/driver.rs` | Z7A/Q1 LED driver solved with a Shockley diode model into a lookup table |
| `src/optocoupler.rs` | CdS cells: G ∝ I^0.9, integrated in the log-conductance domain; brightening τ ≈ 3.5 ms, darkening slows as the cell gets darker (22 ms → 1.5 s). PM1/PM2 bypass cell |
| `src/allpass.rs` | All-pass stage as x − 2·LP(x) on a topology-preserving one-pole, safe under fast resistance sweeps |
| `src/phasor.rs` | Full channel: input coupling, Z8B summer, six stages, C12, C5 feedback roll-off, Z9B, PM1, R31/R29 mixer with C14, op-amp rail limits. The feedback loop is solved without a unit delay each sample |
| `src/sweep.rs` | Triangle/square generators with each generator's own square RC network and level |
| `src/envelope.rs` | C-100 pedal (fast) and an envelope mode that plays the pedal from the input level |
| `src/routing.rs` | Phasor B Input switch |

Signal levels are modelled in volts: 0 dBFS = 2 V peak at the input jack (the
manual's noise reference). **Input Level** pushes the signal harder into the
4558 stages; with feedback near 10 and hot input you hear them reach the rails.

### Calibrated, not documented

The schematics don't give everything. These values were chosen from typical
parts of the era and checked by measurement (see below):

- Photocell sensitivity: 55 kΩ / 58 kΩ at 1 mA (A / B pod), γ = 0.9, 5 MΩ dark.
  Six cells share one LED, so each sees far less light than a vactrol cell.
- Trimmer settings: R130 offset 4.73 V (full-depth sweep just turns the LED off),
  R129 sense 550 Ω total (~16 mA peak), R123 within ±2 % of ideal.
- Photocell time constants and the dark-recovery slope.
- Per-unit tolerances (stage caps, light coupling) — one fixed "unit" per phasor.

### Additions that the hardware doesn't have

Host tempo sync for both generators, the envelope pedal mode, Input Level,
Mix, Output and a master bypass. In plugin context "Input A" is the mono sum of
the stereo input; in "Out A" mode Output B goes to both channels.

## 3. Controls

| Section | Control | Notes |
| :-- | :-- | :-- |
| Sweep Generator 1 | Rate 0.1–18 Hz, Rate Control Man/Ped, Shape ∿/⊓, Sync + division | Ped: the pedal sets the VCO rate (linear, like the circuit) |
| Sweep Gen 2 | Rate, Shape, Sync + division | |
| Phasor A | Depth 1–10, Feedback 0–10, Sweep Gen 1/Ped, footswitch | The manual's sweet spot for slow sweeps is Depth ≈ 8 |
| Phasor B | Depth, Feedback, Sweep Gen 1/Gen 2/Ped, Sweep Sync Norm/Rev, footswitch | Rev inverts the sweep CV: B goes down while A goes up |
| Phasor B Input | In A / Out A / In B | Parallel-stereo / series / independent |
| C-100 Pedal | Heel–toe position, Manual/Envelope, Env Sens | |
| Studio | Input Level, Mix, Output, Bypass | |

The editor redraws the panel layout in a flat, Ableton-style look. The display
on top shows each phasor's response computed from the pod state the DSP
reports; the footswitch LEDs follow the real optical fade.

## 4. Factory presets

From the owner's manual figures, plus a few studio settings:
Stereo Phasing (Fig. B, default) · Double Depth Phasing (Fig. A) ·
Two-Speed Phasing · Dual Synched Phasing (Fig. C) · Unsynchronized Swirl ·
Square Echo Repeat · Pedal Wah · Pseudo Ring Mod · Black Ark Drift.

## Verification

```bash
cargo test --release --lib              # circuit-level regression tests
cargo truce build --vst3 && python3 tools/measure_mutante.py
```

Measured on the built VST3 (Phasor A on the pedal, depth 10):

| Pedal | Notches, feedback 0 | Feedback 10 |
| :-- | :-- | :-- |
| heel | 56 Hz (−5 dB), 138 Hz (−14), 475 Hz (−30) | peaks +7.5 / +11 dB |
| centre | 439 Hz (−29), 1.45 kHz (−37), 5.2 kHz (−39) | notches −12 dB, peaks +12 dB |
| toe | 1.5 kHz (−38), 5.2 kHz (−39), 14 kHz (−39) | |

- With a 0.5 Hz triangle at full depth the stage cutoff sweeps 114 Hz → 5.8 kHz;
  at 8 Hz the bottom rises because the cells can't darken in time.
- Footswitch off: within −0.3/+0.1 dB from 30 Hz to 18 kHz.
- Master bypass: bit-exact. pluginval strictness 10: pass. CPU ≈ 0.7 % of one core.

The Arturia Phaser BI-TRON installed here runs in unactivated demo mode (silent),
so it couldn't be used as a measurement reference; the schematics were.

## Building

```bash
cargo install cargo-truce
cargo truce build            # bundles in target/bundles/
cargo truce install --user   # install CLAP / VST3 / AU
cargo truce run              # standalone
```

## License

Code: [MIT](LICENSE), © 2026 Jardim Sonoro / nagualcode. Original hardware
design: Mike Beigel & Musitronics Corp. (1974). Mu-Tron is a trademark of its
respective owner; this is an independent emulation.
