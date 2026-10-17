#!/usr/bin/env python3
"""Bench measurements for Mutante (Mu-Tron Bi-Phase emulation).

Loads the built VST3 with pedalboard and reports:
1. pluginval result (strictness 10).
2. Bit-exact master bypass.
3. Static notch/peak positions across the pedal sweep (Phasor A on "Ped",
   depth 10) at feedback 0, 5 and 10.
4. Footswitch-off transparency (the dry path through the buffers).

Usage: cargo truce build --vst3 && python3 tools/measure_mutante.py
"""

import subprocess
import sys

import numpy as np
import pedalboard
import scipy.signal as ss

MUTANTE_VST = "target/bundles/Mutante.vst3"
PLUGINVAL = "/Applications/pluginval.app/Contents/MacOS/pluginval"
SR = 48000


def run_pluginval(path):
    cmd = [PLUGINVAL, "--strictness-level", "10", "--validate-in-process", "--validate", path]
    res = subprocess.run(cmd, capture_output=True, text=True)
    return res.returncode == 0 and "SUCCESS" in res.stdout


def transfer(plugin):
    """Transfer function of the left channel, measured with white noise."""
    plugin.reset()
    plugin.process(np.zeros((2, SR), np.float32), SR, reset=False)  # settle the cells
    x = (np.random.default_rng(1).standard_normal(SR * 3) * 0.05).astype(np.float32)
    y = plugin.process(np.stack([x, x]), SR, reset=False)
    f, pxy = ss.csd(x, y[0], SR, nperseg=16384)
    _, pxx = ss.welch(x, SR, nperseg=16384)
    return f, 20 * np.log10(np.abs(pxy / pxx) + 1e-9)


def extrema(f, h):
    band = (f > 15) & (f < 20000)
    lo, _ = ss.find_peaks(-h, prominence=2)
    hi, _ = ss.find_peaks(h, prominence=1)
    notches = [(f[i], h[i]) for i in lo if band[i]]
    peaks = [(f[i], h[i]) for i in hi if band[i] and h[i] > 0.5]
    return notches, peaks


def main():
    print("=" * 72)
    print("MUTANTE BENCH MEASUREMENTS")
    print("=" * 72)

    print("\n[1] pluginval (strictness 10) ...", end=" ", flush=True)
    print("PASS" if run_pluginval(MUTANTE_VST) else "FAIL")

    plugin = pedalboard.load_plugin(MUTANTE_VST)

    print("\n[2] Master bypass")
    plugin.bypass = True
    x = np.random.default_rng(0).uniform(-0.5, 0.5, (2, 8192)).astype(np.float32)
    print(f"    max |out - in| = {np.max(np.abs(plugin.process(x, SR) - x)):.3e}")
    plugin.bypass = False

    print("\n[3] Static response across the pedal sweep (Phasor A, depth 10)")
    plugin.phasor_b_input = "In B"
    plugin.phasor_a_sweep = "Ped"
    plugin.pedal_mode = "Manual"
    plugin.phasor_a_depth = 10.0
    plugin.mix = 100.0
    for fb in (0.0, 5.0, 10.0):
        plugin.phasor_a_feedback = fb
        print(f"    feedback {fb:4.1f}")
        for ped in (0.0, 0.25, 0.5, 0.75, 1.0):
            plugin.pedal = ped
            notches, peaks = extrema(*transfer(plugin))
            n_txt = "  ".join(f"{fq:6.0f} Hz {db:6.1f}" for fq, db in notches[:3])
            p_txt = "  ".join(f"{fq:6.0f} Hz +{db:4.1f}" for fq, db in peaks[:3])
            print(f"      pedal {ped:4.2f}  notches: {n_txt}")
            if peaks:
                print(f"                  peaks:   {p_txt}")

    print("\n[4] Footswitch off (dry path through the input/output buffers)")
    plugin.phasor_a_on = False
    f, h = transfer(plugin)
    band = (f > 30) & (f < 18000)
    print(f"    deviation 30 Hz-18 kHz: {h[band].min():+.2f} .. {h[band].max():+.2f} dB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
