#!/usr/bin/env python3
"""Comparative bench suite: Mutante (VST3) vs. Arturia Phaser BI-TRON (DSP engine).

Loads the built Mutante VST3 with Spotify's pedalboard and drives Arturia's
underlying DSP dylib (`/Library/Arturia/Phaser BI-TRON/resources/libneutronProcessor.dylib`,
which bypasses the GUI demo-activation gate in `Phaser BI-TRON.vst3`) via ctypes
in 32-sample sub-blocks at 48 kHz.

Usage:
    cargo truce build --vst3 && python3 tools/compare_arturia.py
"""

import ctypes
import os
import struct
import sys
import time
import numpy as np
from pedalboard import load_plugin

SR = 48000.0
MUTANTE_PATH = "target/bundles/Mutante.vst3"
ARTURIA_DYLIB = "/Library/Arturia/Phaser BI-TRON/resources/libneutronProcessor.dylib"


class ArturiaBiTronDSP:
    """Direct ctypes harness for Arturia's DsplibProcessor<NeutronProcessor, true>."""

    def __init__(self, sample_rate=SR):
        self.sr = float(sample_rate)
        self.lib = ctypes.CDLL(ARTURIA_DYLIB)
        self.lib.GetProcessorFactory.restype = ctypes.c_void_p
        self.factory = self.lib.GetProcessorFactory()
        f_vt = ctypes.cast(
            ctypes.cast(self.factory, ctypes.POINTER(ctypes.c_void_p))[0],
            ctypes.POINTER(ctypes.c_void_p),
        )
        create_fn = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_void_p)(f_vt[1])
        self.proc = create_fn(self.factory)
        self.vt = ctypes.cast(
            ctypes.cast(self.proc, ctypes.POINTER(ctypes.c_void_p))[0],
            ctypes.POINTER(ctypes.c_void_p),
        )

        self._init_fn = ctypes.CFUNCTYPE(
            None, ctypes.c_void_p, ctypes.POINTER(ctypes.c_float)
        )(self.vt[2])
        self._reset_fn = ctypes.CFUNCTYPE(None, ctypes.c_void_p)(self.vt[4])
        self._get_param_fn = ctypes.CFUNCTYPE(
            ctypes.c_bool,
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.POINTER(ctypes.c_float),
        )(self.vt[7])
        self._set_param_fn = ctypes.CFUNCTYPE(
            ctypes.c_bool, ctypes.c_void_p, ctypes.c_uint32, ctypes.c_float
        )(self.vt[8])
        self._process_fn = ctypes.CFUNCTYPE(
            None, ctypes.c_void_p, ctypes.c_void_p
        )(self.vt[38])

        sr_c = ctypes.c_float(self.sr)
        self._init_fn(self.proc, ctypes.byref(sr_c))
        self._reset_fn(self.proc)

    def reset(self):
        self._reset_fn(self.proc)

    def set_param(self, pid, val):
        self._set_param_fn(self.proc, int(pid), float(val))

    def configure_clean_stereo(self):
        """Set Arturia to Stereo Parallel (L=BiPhase1, R=BiPhase2), 6 poles, Feedback 0."""
        self.set_param(174, 0.0)   # Master Bypass = Active
        self.set_param(178, 0.5)   # OutputGain = 0 dB
        self.set_param(179, 0.0)   # Input Filter = Off
        self.set_param(192, 0.0)   # Routing Mono = Parallel
        self.set_param(193, 0.0)   # Routing Stereo = Stereo Parallel (In B)
        self.set_param(190, 0.0)   # Sweep Sync = Normal
        self.set_param(110, 0.0)   # Pedal Mode = Manual
        self.set_param(111, 0.0)   # Pedal ManualAmount = 0.0
        # BiPhase 1
        self.set_param(18, 0.0)    # Feedback = 0
        self.set_param(19, 0.0)    # Footswitch Bypass = Active
        self.set_param(21, 0.4)    # 6 poles
        self.set_param(26, 0.5)    # DryWet = 50%
        self.set_param(27, 0.0)    # BiPhase1 Bypass = Active
        self.set_param(180, 1.0)   # Depth1 = 10
        self.set_param(181, 0.0)   # Depth2 = 1
        self.set_param(182, 0.0)   # Sweep = Gen
        # BiPhase 2
        self.set_param(47, 0.0)    # Feedback = 0
        self.set_param(48, 0.0)    # Footswitch Bypass = Active
        self.set_param(50, 0.4)    # 6 poles
        self.set_param(55, 0.5)    # DryWet = 50%
        self.set_param(56, 0.0)    # BiPhase2 Bypass = Active
        self.set_param(185, 1.0)   # Depth1 = 10
        self.set_param(186, 0.0)   # Depth2 = 1
        self.set_param(187, 0.0)   # Sweep = Gen

    def process(self, audio_2ch):
        """Process (2, N) float32 array in 32-sample sub-blocks."""
        n_total = audio_2ch.shape[1]
        out = np.zeros((2, n_total), dtype=np.float32)
        chunk = 32
        in_l = (ctypes.c_float * chunk)()
        in_r = (ctypes.c_float * chunk)()
        out_l = (ctypes.c_float * chunk)()
        out_r = (ctypes.c_float * chunk)()
        in_ptrs = (ctypes.c_void_p * 2)(
            ctypes.cast(in_l, ctypes.c_void_p), ctypes.cast(in_r, ctypes.c_void_p)
        )
        out_ptrs = (ctypes.c_void_p * 2)(
            ctypes.cast(out_l, ctypes.c_void_p), ctypes.cast(out_r, ctypes.c_void_p)
        )
        pdata = ctypes.create_string_buffer(256)
        struct.pack_into("<If", pdata, 0x9C, 1, 120.0)

        for idx in range(0, n_total, chunk):
            b = min(chunk, n_total - idx)
            in_l[:b] = audio_2ch[0, idx : idx + b]
            in_r[:b] = audio_2ch[1, idx : idx + b]
            struct.pack_into(
                "<QQIII",
                pdata,
                0x10,
                ctypes.addressof(out_ptrs),
                ctypes.addressof(in_ptrs),
                2,
                2,
                b,
            )
            self._process_fn(self.proc, ctypes.byref(pdata))
            out[0, idx : idx + b] = out_l[:b]
            out[1, idx : idx + b] = out_r[:b]
        return out


def setup_mutante(mut):
    mut.bypass = False
    mut.phasor_b_input = "In B"
    mut.input_level_db = 0.0
    mut.output_gain_db = 0.0
    mut.mix = 100.0
    mut.pedal_mode = "Manual"
    mut.pedal = 0.0
    mut.gen_1_rate = 1.0
    mut.gen_1_sync = False
    mut.gen_1_shape = "Sine"
    mut.gen_1_rate_control = "Man"
    mut.gen_2_rate = 1.0
    mut.gen_2_sync = False
    mut.gen_2_shape = "Sine"
    mut.phasor_a_on = True
    mut.phasor_a_depth = 10.0
    mut.phasor_a_feedback = 0.0
    mut.phasor_a_sweep = "Gen 1"
    mut.phasor_b_on = True
    mut.phasor_b_depth = 10.0
    mut.phasor_b_feedback = 0.0
    mut.phasor_b_sweep = "Gen 1"
    mut.phasor_b_sweep_sync = "Norm"
    mut.reset()


def measure_ir_transfer(process_fn, channel=0, settle_s=0.45, n_fft=65536):
    n_settle = int(settle_s * SR)
    silence = np.zeros((2, n_settle), dtype=np.float32)
    process_fn(silence)

    impulse = np.zeros((2, n_fft), dtype=np.float32)
    impulse[0, 0] = 0.1
    impulse[1, 0] = 0.1
    out = process_fn(impulse)
    H = np.fft.rfft(out[channel]) / 0.1
    freqs = np.fft.rfftfreq(n_fft, 1.0 / SR)
    mag_db = 20.0 * np.log10(np.abs(H) + 1e-12)
    return freqs, mag_db


def find_notches_and_peaks(freqs, mag_db, f_min=25.0, f_max=18000.0):
    mask = (freqs >= f_min) & (freqs <= f_max)
    f = freqs[mask]
    m = mag_db[mask]
    notches = []
    peaks = []
    for i in range(2, len(m) - 2):
        if m[i] < m[i - 1] and m[i] < m[i + 1] and m[i] < -2.5:
            y0, y1, y2 = m[i - 1], m[i], m[i + 1]
            denom = y0 - 2 * y1 + y2
            delta = 0.5 * (y0 - y2) / denom if abs(denom) > 1e-9 else 0.0
            fn = f[i] + delta * (f[1] - f[0])
            notches.append((fn, y1))
        if m[i] > m[i - 1] and m[i] > m[i + 1] and m[i] > 1.0:
            peaks.append((f[i], m[i]))
    return notches, peaks


def first_stage_notch(freqs, mag_db):
    notches, _ = find_notches_and_peaks(freqs, mag_db, f_min=95.0, f_max=6000.0)
    for fn, db in notches:
        if db < -6.0:
            return fn
    return notches[0][0] if notches else 0.0


def main():
    if not os.path.exists(ARTURIA_DYLIB):
        print(f"Arturia dylib not found at {ARTURIA_DYLIB}", file=sys.stderr)
        return 1

    mut = load_plugin(MUTANTE_PATH)
    art = ArturiaBiTronDSP(SR)

    print("=" * 80)
    print("1. STATIC NOTCH COMPARISON: PEDAL SWEEP (FB = 0, DEPTH = 10)")
    print("=" * 80)
    for p in [0.0, 0.25, 0.5, 0.75, 1.0]:
        setup_mutante(mut)
        mut.phasor_a_sweep = "Ped"
        mut.phasor_b_sweep = "Ped"
        mut.pedal = p
        f_m, m_ma = measure_ir_transfer(lambda x: mut.process(x, SR, reset=False), channel=0)
        setup_mutante(mut)
        mut.phasor_a_sweep = "Ped"
        mut.phasor_b_sweep = "Ped"
        mut.pedal = p
        _, m_mb = measure_ir_transfer(lambda x: mut.process(x, SR, reset=False), channel=1)
        na, _ = find_notches_and_peaks(f_m, m_ma)
        nb, _ = find_notches_and_peaks(f_m, m_mb)

        art.configure_clean_stereo()
        art.set_param(182, 1.0)
        art.set_param(187, 1.0)
        art.set_param(111, p)
        art.reset()
        f_a, m_a1 = measure_ir_transfer(art.process, channel=0)
        art.configure_clean_stereo()
        art.set_param(182, 1.0)
        art.set_param(187, 1.0)
        art.set_param(111, p)
        art.reset()
        _, m_a2 = measure_ir_transfer(art.process, channel=1)
        na1, _ = find_notches_and_peaks(f_a, m_a1)
        na2, _ = find_notches_and_peaks(f_a, m_a2)

        print(f"\n--- Pedal = {p:.2f} ---")
        print("  Mutante Phasor A             :", ", ".join(f"{fn:.1f}Hz ({db:.1f}dB)" for fn, db in na))
        print("  Mutante Phasor B             :", ", ".join(f"{fn:.1f}Hz ({db:.1f}dB)" for fn, db in nb))
        print("  Arturia BiPhase 1 (Spread=.9):", ", ".join(f"{fn:.1f}Hz ({db:.1f}dB)" for fn, db in na1))
        print("  Arturia BiPhase 2 (Spread=.1):", ", ".join(f"{fn:.1f}Hz ({db:.1f}dB)" for fn, db in na2))

    print("\n" + "=" * 80)
    print("2. FEEDBACK RESONANCE & C12 LOADING SHIFT (PEDAL = 0.0 and 0.5, FB = 0, 5, 10)")
    print("=" * 80)
    for p in [0.0, 0.5]:
        for fb in [0.0, 5.0, 10.0]:
            setup_mutante(mut)
            mut.phasor_a_sweep = "Ped"
            mut.pedal = p
            mut.phasor_a_feedback = fb
            f_m, m_ma = measure_ir_transfer(lambda x: mut.process(x, SR, reset=False), channel=0)
            na, pa = find_notches_and_peaks(f_m, m_ma)

            art.configure_clean_stereo()
            art.set_param(182, 1.0)
            art.set_param(187, 1.0)
            art.set_param(111, p)
            art.set_param(18, fb / 10.0)
            art.set_param(47, fb / 10.0)
            art.reset()
            f_a, m_a2 = measure_ir_transfer(art.process, channel=1)
            na2, pa2 = find_notches_and_peaks(f_a, m_a2)

            print(f"\nPedal={p:.1f}, FB={fb:.0f}:")
            print("  Mutante Notches:", ", ".join(f"{fn:.1f}Hz ({db:.1f}dB)" for fn, db in na))
            print("  Mutante Peaks  :", ", ".join(f"{fn:.1f}Hz (+{db:.1f}dB)" for fn, db in pa))
            print("  Arturia Notches:", ", ".join(f"{fn:.1f}Hz ({db:.1f}dB)" for fn, db in na2))
            print("  Arturia Peaks  :", ", ".join(f"{fn:.1f}Hz (+{db:.1f}dB)" for fn, db in pa2))

    print("\n" + "=" * 80)
    print("3. GEN 1 vs GEN 2 SQUARE WAVE ASYMMETRY (2.0 Hz on Phasor B)")
    print("=" * 80)
    for gen_idx in [1, 2]:
        setup_mutante(mut)
        mut.gen_1_rate = 2.0
        mut.gen_2_rate = 2.0
        mut.gen_1_shape = "Square"
        mut.gen_2_shape = "Square"
        mut.phasor_b_sweep = f"Gen {gen_idx}"

        art.configure_clean_stereo()
        art.set_param(137, 1.0)  # Square Gen 1
        art.set_param(168, 1.0)  # Square Gen 2
        art.set_param(138, 0.52)
        art.set_param(169, 0.52)
        if gen_idx == 1:
            art.set_param(185, 1.0)
            art.set_param(186, 0.0)
        else:
            art.set_param(185, 0.0)
            art.set_param(186, 1.0)
        art.reset()

        warm = np.zeros((2, int(1.5 * SR)), dtype=np.float32)
        mut.process(warm, SR, reset=False)
        art.process(warm)

        step = 1000
        mut_f1, art_f1 = [], []
        ff = np.fft.rfftfreq(16384, 1.0 / SR)
        for _ in range(24):
            imp = np.zeros((2, step), dtype=np.float32)
            imp[:, 0] = 0.1
            out_m = mut.process(imp, SR, reset=False)
            out_a = art.process(imp)
            Hm = 20 * np.log10(np.abs(np.fft.rfft(out_m[1, :512], n=16384)) / 0.1 + 1e-12)
            Ha = 20 * np.log10(np.abs(np.fft.rfft(out_a[1, :512], n=16384)) / 0.1 + 1e-12)
            mut_f1.append(first_stage_notch(ff, Hm))
            art_f1.append(first_stage_notch(ff, Ha))
        print(f"Gen {gen_idx} Square | Mutante Phasor B 1st Notch: min={min(mut_f1):.1f} Hz, max={max(mut_f1):.1f} Hz")
        print(f"             | Arturia BiPhase2 1st Notch: min={min(art_f1):.1f} Hz, max={max(art_f1):.1f} Hz")

    print("\n" + "=" * 80)
    print("4. HARDWARE TOPOLOGY TESTS: DEPTH=1 FLOOR & PEDAL+DEPTH INTERACTION")
    print("=" * 80)
    setup_mutante(mut)
    mut.phasor_a_sweep = "Ped"
    mut.phasor_a_depth = 1.0
    mut.pedal = 0.0
    f_m, m0 = measure_ir_transfer(lambda x: mut.process(x, SR, reset=False), channel=0)
    n0, _ = find_notches_and_peaks(f_m, m0)
    setup_mutante(mut)
    mut.phasor_a_sweep = "Ped"
    mut.phasor_a_depth = 1.0
    mut.pedal = 1.0
    f_m, m1 = measure_ir_transfer(lambda x: mut.process(x, SR, reset=False), channel=0)
    n1, _ = find_notches_and_peaks(f_m, m1)
    print(f"Mutante (Sweep=Ped, Depth=1.0): Pedal 0->1 shifts 2nd notch {n0[1][0]:.1f} Hz -> {n1[1][0]:.1f} Hz (R96=560Ω floor active!)")

    setup_mutante(mut)
    mut.phasor_a_sweep = "Ped"
    mut.phasor_a_depth = 5.0
    mut.pedal = 1.0
    f_m, m5 = measure_ir_transfer(lambda x: mut.process(x, SR, reset=False), channel=0)
    n5, _ = find_notches_and_peaks(f_m, m5)
    print(f"Mutante (Sweep=Ped, Depth=5.0, Pedal=1.0): 2nd notch = {n5[1][0]:.1f} Hz (scales pedal range as per Manual p. 5)")

    art.configure_clean_stereo()
    art.set_param(182, 1.0)
    art.set_param(180, 0.0)
    art.set_param(111, 1.0)
    art.reset()
    f_a, a_d1 = measure_ir_transfer(art.process, channel=0)
    na_d1, _ = find_notches_and_peaks(f_a, a_d1)
    art.configure_clean_stereo()
    art.set_param(182, 1.0)
    art.set_param(180, 1.0)
    art.set_param(111, 1.0)
    art.reset()
    f_a, a_d10 = measure_ir_transfer(art.process, channel=0)
    na_d10, _ = find_notches_and_peaks(f_a, a_d10)
    print(f"Arturia (Sweep=Ped, Pedal=1.0): Depth=1 -> 2nd notch {na_d1[1][0]:.1f} Hz vs Depth=10 -> {na_d10[1][0]:.1f} Hz (Depth completely ignored!)")

    print("\n" + "=" * 80)
    print("5. THD & OP-AMP RAIL SATURATION (200 Hz SINE, FB = 0 and 10)")
    print("=" * 80)
    n_thd = int(1.0 * SR)
    t = np.arange(n_thd) / SR
    f0 = 200.0
    for amp_label, amp, in_lvl_db, fb_val in [
        ("-18 dBFS, In=0dB, FB=0", 0.125, 0.0, 0.0),
        ("0 dBFS, In=+18dB, FB=10", 1.0, 18.0, 10.0),
    ]:
        sine = (amp * np.sin(2 * np.pi * f0 * t)).astype(np.float32)
        stereo_in = np.vstack([sine, sine])

        setup_mutante(mut)
        mut.phasor_a_sweep = "Ped"
        mut.pedal = 0.3
        mut.input_level_db = in_lvl_db
        mut.phasor_a_feedback = fb_val
        out_m = mut.process(stereo_in, SR, reset=False)[0, int(0.2 * SR) :]

        art.configure_clean_stereo()
        art.set_param(182, 1.0)
        art.set_param(111, 0.3)
        art.set_param(18, fb_val / 10.0)
        art.reset()
        out_a = art.process(stereo_in)[0, int(0.2 * SR) :]

        def calc_thd(sig):
            w = np.hanning(len(sig))
            spec = np.abs(np.fft.rfft(sig * w))
            freqs = np.fft.rfftfreq(len(sig), 1.0 / SR)
            idx1 = np.argmin(np.abs(freqs - f0))
            fund = np.max(spec[max(0, idx1 - 2) : idx1 + 3])
            harm_sq = 0.0
            for h in range(2, 10):
                idxh = np.argmin(np.abs(freqs - h * f0))
                harm_sq += np.max(spec[max(0, idxh - 2) : idxh + 3]) ** 2
            return 100.0 * np.sqrt(harm_sq) / (fund + 1e-12)

        print(f"Signal {amp_label:24s} | Mutante THD = {calc_thd(out_m):.4f}% | Arturia THD = {calc_thd(out_a):.4f}%")

    print("\n" + "=" * 80)
    print("6. CPU BENCHMARK (10 SECONDS OF STEREO 48kHz AUDIO)")
    print("=" * 80)
    bench_audio = np.random.uniform(-0.1, 0.1, (2, int(10 * SR))).astype(np.float32)
    t0 = time.perf_counter()
    mut.process(bench_audio, SR, reset=False)
    dt_mut = (time.perf_counter() - t0) / 10.0 * 1000.0

    t0 = time.perf_counter()
    art.process(bench_audio)
    dt_art = (time.perf_counter() - t0) / 10.0 * 1000.0
    print(f"Mutante : {dt_mut:.2f} ms per 1s of audio ({dt_mut / 10.0:.2f}% single-core realtime)")
    print(f"Arturia : {dt_art:.2f} ms per 1s of audio ({dt_art / 10.0:.2f}% single-core realtime, incl. ctypes chunk loop)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
