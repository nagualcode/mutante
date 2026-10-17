//! Mutante: component-level emulation of the Musitronics Mu-Tron Bi-Phase.
//!
//! The model follows the ARP/Musitronics service schematics stage by stage:
//! two six-stage op-amp all-pass ladders whose resistors are CdS photocells
//! lit by a shared LED per phasor, the diode-shaped LED current drivers, the
//! Z8B feedback summer, the C12/C14 wet coupling networks, the passive
//! dry/wet mixer with its notch-depth trim, the PM1/PM2 optical footswitch
//! bypass, and the two triangle/square sweep generators with their RC-rounded
//! square outputs. See `circuit.rs` for the transcribed component values.

mod allpass;
mod circuit;
mod driver;
mod envelope;
mod optocoupler;
mod phasor;
mod routing;
mod sweep;
mod ui;

use std::sync::Arc;
use truce::prelude::*;

pub use envelope::{PedalFollower, PedalMode};
pub use phasor::{PhasorASweepSource, PhasorBSweepSource, PhasorChannel};
pub use routing::PhasorBInput;
pub use sweep::{GenCircuit, LfoShape, RateControl, SweepGenerator, SweepSync, SyncRate};

use circuit::{GEN_TRI_PEAK, R_DEPTH_FLOOR, R_DEPTH_POT, VOLTS_PER_FS};

#[derive(Params)]
pub struct MutanteParams {
    // --- Sweep Generator 1 ---
    #[param(name = "Gen 1 Rate", short_name = "Rate 1", range = "log(0.1, 18.0)",
            default = 0.4, smooth = "exp(30)")]
    pub gen_1_rate: FloatParam,

    #[param(name = "Gen 1 Sync", short_name = "Sync 1", default = false)]
    pub gen_1_sync: BoolParam,

    #[param(name = "Gen 1 Synced Rate", short_name = "Div 1")]
    pub gen_1_synced_rate: EnumParam<SyncRate>,

    #[param(name = "Gen 1 Shape", short_name = "Shape 1")]
    pub gen_1_shape: EnumParam<LfoShape>,

    #[param(name = "Gen 1 Rate Control", short_name = "RateCtl 1")]
    pub gen_1_rate_ctrl: EnumParam<RateControl>,

    // --- Sweep Generator 2 ---
    #[param(name = "Gen 2 Rate", short_name = "Rate 2", range = "log(0.1, 18.0)",
            default = 1.6, smooth = "exp(30)")]
    pub gen_2_rate: FloatParam,

    #[param(name = "Gen 2 Sync", short_name = "Sync 2", default = false)]
    pub gen_2_sync: BoolParam,

    #[param(name = "Gen 2 Synced Rate", short_name = "Div 2")]
    pub gen_2_synced_rate: EnumParam<SyncRate>,

    #[param(name = "Gen 2 Shape", short_name = "Shape 2")]
    pub gen_2_shape: EnumParam<LfoShape>,

    // --- Phasor A ---
    #[param(name = "Phasor A Depth", short_name = "Depth A", range = "linear(1.0, 10.0)",
            default = 8.0, smooth = "exp(20)")]
    pub phasor_a_depth: FloatParam,

    #[param(name = "Phasor A Feedback", short_name = "Fdbk A", range = "linear(0.0, 10.0)",
            default = 3.0, smooth = "exp(20)")]
    pub phasor_a_feedback: FloatParam,

    #[param(name = "Phasor A Sweep", short_name = "Sweep A")]
    pub phasor_a_sweep: EnumParam<PhasorASweepSource>,

    #[param(name = "Phasor A On", short_name = "On A", default = true)]
    pub phasor_a_on: BoolParam,

    // --- Phasor B ---
    #[param(name = "Phasor B Depth", short_name = "Depth B", range = "linear(1.0, 10.0)",
            default = 8.0, smooth = "exp(20)")]
    pub phasor_b_depth: FloatParam,

    #[param(name = "Phasor B Feedback", short_name = "Fdbk B", range = "linear(0.0, 10.0)",
            default = 3.0, smooth = "exp(20)")]
    pub phasor_b_feedback: FloatParam,

    #[param(name = "Phasor B Sweep", short_name = "Sweep B")]
    pub phasor_b_sweep: EnumParam<PhasorBSweepSource>,

    #[param(name = "Phasor B Sweep Sync", short_name = "Sync B")]
    pub phasor_b_sweep_sync: EnumParam<SweepSync>,

    #[param(name = "Phasor B On", short_name = "On B", default = true)]
    pub phasor_b_on: BoolParam,

    // --- Phasor B input switch ---
    #[param(name = "Phasor B Input", short_name = "B Input")]
    pub routing: EnumParam<PhasorBInput>,

    // --- C-100 pedal ---
    #[param(name = "Pedal", short_name = "Pedal", range = "linear(0.0, 1.0)",
            default = 0.5, smooth = "exp(10)")]
    pub pedal_position: FloatParam,

    #[param(name = "Pedal Mode", short_name = "Ped Mode")]
    pub pedal_mode: EnumParam<PedalMode>,

    #[param(name = "Envelope Sensitivity", short_name = "Env Sens", range = "log(0.25, 16.0)",
            default = 2.0, smooth = "exp(20)")]
    pub pedal_sensitivity: FloatParam,

    // --- Studio ---
    #[param(name = "Input Level", short_name = "In Lvl", range = "linear(-12.0, 24.0)",
            default = 0.0, unit = "dB", smooth = "exp(20)")]
    pub input_level: FloatParam,

    #[param(name = "Mix", short_name = "Mix", range = "linear(0.0, 1.0)",
            default = 1.0, unit = "%", smooth = "exp(20)")]
    pub mix: FloatParam,

    #[param(name = "Output Gain", short_name = "Out", range = "linear(-12.0, 12.0)",
            default = 0.0, unit = "dB", smooth = "exp(20)")]
    pub output_gain: FloatParam,

    #[param(name = "Bypass", short_name = "Bypass", default = false)]
    pub bypass: BoolParam,

    // --- Display meters ---
    #[meter]
    pub lfo1_meter: MeterSlot,
    #[meter]
    pub lfo2_meter: MeterSlot,
    #[meter]
    pub pod_a_meter: MeterSlot,
    #[meter]
    pub pod_b_meter: MeterSlot,
    #[meter]
    pub notch_a_meter: MeterSlot,
    #[meter]
    pub notch_b_meter: MeterSlot,
    #[meter]
    pub lamp_a_meter: MeterSlot,
    #[meter]
    pub lamp_b_meter: MeterSlot,
    #[meter]
    pub pedal_meter: MeterSlot,
    #[meter]
    pub out_meter_l: MeterSlot,
    #[meter]
    pub out_meter_r: MeterSlot,
}

use MutanteParamsParamId as P;

/// DEPTH pot (with its 560 Ω floor resistor) as a fraction of the sweep.
#[inline]
fn depth_fraction(dial: f32) -> f32 {
    let p = ((dial - 1.0) / 9.0).clamp(0.0, 1.0);
    (R_DEPTH_FLOOR + R_DEPTH_POT * p) / (R_DEPTH_FLOOR + R_DEPTH_POT)
}

/// Gen 1 rate when its RATE CONTROL switch is on "Ped": the pedal voltage
/// replaces the RATE pot at the VCO input, which is linear in volts.
#[inline]
fn pedal_rate_hz(pedal: f32) -> f32 {
    0.1 + 17.9 * pedal.clamp(0.0, 1.0)
}

/// Maps a centre frequency (Hz) onto 0..1 for the notch display meters.
#[inline]
fn notch_display(hz: f32) -> f32 {
    ((hz / 30.0).ln() / (20_000.0f32 / 30.0).ln()).clamp(0.0, 1.0)
}

pub struct Mutante;

pub struct MutanteDsp {
    gen_1: SweepGenerator,
    gen_2: SweepGenerator,
    pedal: PedalFollower,
    phasor_a: PhasorChannel,
    phasor_b: PhasorChannel,
    sample_rate: f32,
    last_gen_1: f32,
    last_gen_2: f32,
    last_pedal: f32,
    peak_l: f32,
    peak_r: f32,
}

impl Default for MutanteDsp {
    fn default() -> Self {
        let sr = 48_000.0;
        Self {
            gen_1: SweepGenerator::new(sr, GenCircuit::Gen1),
            gen_2: SweepGenerator::new(sr, GenCircuit::Gen2),
            pedal: PedalFollower::new(sr),
            phasor_a: PhasorChannel::new(sr, false),
            phasor_b: PhasorChannel::new(sr, true),
            sample_rate: sr,
            last_gen_1: 0.0,
            last_gen_2: 0.0,
            last_pedal: 0.5,
            peak_l: 0.0,
            peak_r: 0.0,
        }
    }
}

impl PluginLogic for Mutante {
    type Params = MutanteParams;
    type DspState = MutanteDsp;

    fn reset(state: &mut Self::DspState, params: &Self::Params, config: &AudioConfig) {
        let sr = config.sample_rate as f32;
        state.sample_rate = sr;
        state.gen_1.set_sample_rate(sr);
        state.gen_2.set_sample_rate(sr);
        state.pedal.set_sample_rate(sr);
        state.phasor_a.set_sample_rate(sr);
        state.phasor_b.set_sample_rate(sr);

        state.gen_1.reset();
        state.gen_2.reset();
        let pedal = params.pedal_position.value();
        state.pedal.reset(pedal);
        // Settle the pods at the bottom of the sweep, where the generators
        // start, so playback does not begin with a fade-in from darkness.
        let start_cv = -GEN_TRI_PEAK;
        state.phasor_a.reset(
            start_cv * depth_fraction(params.phasor_a_depth.value()),
            params.phasor_a_on.value(),
        );
        state.phasor_b.reset(
            start_cv * depth_fraction(params.phasor_b_depth.value()),
            params.phasor_b_on.value(),
        );
        state.peak_l = 0.0;
        state.peak_r = 0.0;
    }

    fn process(
        state: &mut Self::DspState,
        params: &Self::Params,
        buffer: &mut AudioBuffer,
        _events: &EventList,
        context: &mut ProcessContext,
    ) -> ProcessStatus {
        if params.bypass.value() {
            let n_in = buffer.num_input_channels();
            for ch in 0..buffer.channels() {
                let (inp, out) = buffer.io(ch);
                if ch < n_in {
                    out.copy_from_slice(inp);
                } else {
                    out.fill(0.0);
                }
            }
            state.peak_l = 0.0;
            state.peak_r = 0.0;
            context.set_meter(P::OutMeterL, 0.0);
            context.set_meter(P::OutMeterR, 0.0);
            return ProcessStatus::Normal;
        }

        let bpm = if context.transport.tempo > 10.0 {
            context.transport.tempo as f32
        } else {
            120.0
        };
        let synced = |beats: f32| (bpm / 60.0) / beats;

        let gen_1_sync = params.gen_1_sync.value();
        let gen_1_div = params.gen_1_synced_rate.value().beats();
        let gen_1_shape = params.gen_1_shape.value();
        let gen_1_rate_ctrl = params.gen_1_rate_ctrl.value();
        let gen_2_sync = params.gen_2_sync.value();
        let gen_2_div = params.gen_2_synced_rate.value().beats();
        let gen_2_shape = params.gen_2_shape.value();

        let sweep_a = params.phasor_a_sweep.value();
        let on_a = params.phasor_a_on.value();
        let sweep_b = params.phasor_b_sweep.value();
        let sync_b = params.phasor_b_sweep_sync.value();
        let on_b = params.phasor_b_on.value();
        let routing = params.routing.value();
        let pedal_mode = params.pedal_mode.value();

        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;

        buffer.for_each_frame_io::<2, 2, _>(|frame_in, frame_out| {
            let rate_1 = params.gen_1_rate.read();
            let rate_2 = params.gen_2_rate.read();
            let depth_a = depth_fraction(params.phasor_a_depth.read());
            let depth_b = depth_fraction(params.phasor_b_depth.read());
            let fb_a = params.phasor_a_feedback.read() * 0.1;
            let fb_b = params.phasor_b_feedback.read() * 0.1;
            let pedal_manual = params.pedal_position.read();
            let sensitivity = params.pedal_sensitivity.read();
            let in_gain = db_to_gain(params.input_level.read());
            let mix = params.mix.read();
            let out_gain = db_to_gain(params.output_gain.read());

            let (in_l, in_r) = (frame_in[0], frame_in[1]);
            let volts = VOLTS_PER_FS * in_gain;
            let (jack_l, jack_r) = (in_l * volts, in_r * volts);
            let jack_mono = 0.5 * (jack_l + jack_r);

            // C-100 pedal (or envelope follower driving it).
            let pedal = state.pedal.process(
                0.5 * (in_l.abs() + in_r.abs()) * in_gain,
                pedal_mode,
                pedal_manual,
                sensitivity,
            );
            let pedal_cv = PedalFollower::cv(pedal);

            // Sweep generators.
            let f1 = match gen_1_rate_ctrl {
                RateControl::Pedal => pedal_rate_hz(pedal),
                RateControl::Manual if gen_1_sync => synced(gen_1_div),
                RateControl::Manual => rate_1,
            };
            let f2 = if gen_2_sync { synced(gen_2_div) } else { rate_2 };
            let g1 = state.gen_1.tick(f1, gen_1_shape);
            let g2 = state.gen_2.tick(f2, gen_2_shape);

            // Sweep switches and DEPTH pots.
            let cv_a = depth_a
                * match sweep_a {
                    PhasorASweepSource::Gen1 => g1,
                    PhasorASweepSource::Pedal => pedal_cv,
                };
            let src_b = match sweep_b {
                PhasorBSweepSource::Gen1 => g1,
                PhasorBSweepSource::Gen2 => g2,
                PhasorBSweepSource::Pedal => pedal_cv,
            };
            let cv_b = depth_b
                * match sync_b {
                    SweepSync::Norm => src_b,
                    SweepSync::Rev => -src_b,
                };

            // Phasor B input switch.
            let (out_a, out_b) = match routing {
                PhasorBInput::InA => {
                    let a = state.phasor_a.process(jack_mono, cv_a, fb_a, on_a);
                    let b = state.phasor_b.process(jack_mono, cv_b, fb_b, on_b);
                    (a, b)
                }
                PhasorBInput::OutA => {
                    let a = state.phasor_a.process(jack_mono, cv_a, fb_a, on_a);
                    let b = state.phasor_b.process(a, cv_b, fb_b, on_b);
                    (b, b)
                }
                PhasorBInput::InB => {
                    let a = state.phasor_a.process(jack_l, cv_a, fb_a, on_a);
                    let b = state.phasor_b.process(jack_r, cv_b, fb_b, on_b);
                    (a, b)
                }
            };

            let to_fs = 1.0 / volts;
            let l = ((1.0 - mix) * in_l + mix * out_a * to_fs) * out_gain;
            let r = ((1.0 - mix) * in_r + mix * out_b * to_fs) * out_gain;
            frame_out[0] = l;
            frame_out[1] = r;

            peak_l = peak_l.max(l.abs());
            peak_r = peak_r.max(r.abs());
            state.last_gen_1 = g1;
            state.last_gen_2 = g2;
            state.last_pedal = pedal;
        });

        state.peak_l = peak_l.max(state.peak_l * 0.8);
        state.peak_r = peak_r.max(state.peak_r * 0.8);

        let lfo_norm = |v: f32| (0.5 + 0.5 * v / GEN_TRI_PEAK).clamp(0.0, 1.0);
        context.set_meter(P::Lfo1Meter, lfo_norm(state.last_gen_1));
        context.set_meter(P::Lfo2Meter, lfo_norm(state.last_gen_2));
        context.set_meter(P::PodAMeter, state.phasor_a.pod.glow());
        context.set_meter(P::PodBMeter, state.phasor_b.pod.glow());
        context.set_meter(P::NotchAMeter, notch_display(state.phasor_a.pod.center_hz()));
        context.set_meter(P::NotchBMeter, notch_display(state.phasor_b.pod.center_hz()));
        context.set_meter(P::LampAMeter, f32::from(u8::from(state.phasor_a.switch.is_lit())));
        context.set_meter(P::LampBMeter, f32::from(u8::from(state.phasor_b.switch.is_lit())));
        context.set_meter(P::PedalMeter, state.last_pedal);
        context.set_meter(P::OutMeterL, state.peak_l.min(1.0));
        context.set_meter(P::OutMeterR, state.peak_r.min(1.0));

        ProcessStatus::Normal
    }

    fn editor(params: Arc<MutanteParams>) -> Box<dyn Editor> {
        ui::create_editor(params)
    }
}

#[inline]
fn db_to_gain(db: f32) -> f32 {
    10.0f32.powf(db * 0.05)
}

truce::plugin! {
    logic: Mutante,
    params: MutanteParams,
}

truce::enable_rt_paranoid!();
