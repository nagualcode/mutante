//! Mutante: Faithful Physical Emulation of the Classic Mu-Tron Bi-Phase.
//!
//! Features:
//! - Dual 6-Stage Optical Phasors (Phasor A and Phasor B) with CdS photoresistor kinetics.
//! - Photoconductive power-law resistance and asymmetric dual-time-constant relaxation.
//! - Inverting summing feedback loops with op-amp rail soft saturation.
//! - Dual sweep generators (0.1 Hz to 18.0 Hz) with diode-bridge rounded sine and square shapes.
//! - Authentic routing matrix: In A (Mono->Stereo), Out A (12-stage cascade), In B (Dual Mono/Stereo).
//! - Normal and Reverse sweep sync for wide stereo swirling motion.
//! - C-100 optical foot pedal and dynamic envelope follower modulation.
//! - Zero-allocation audio thread processing.

mod allpass;
mod envelope;
mod optocoupler;
mod phasor;
mod routing;
mod sweep;

use std::sync::Arc;
use truce::prelude::*;
use truce_gui::BuiltinEditor;
use truce_gui_types::layout::{dropdown, knob, meter, toggle, widgets, GridLayout};

pub use envelope::{PedalFollower, PedalMode};
pub use phasor::{PhasorASweepSource, PhasorBSweepSource, PhasorChannel, PolesCount};
pub use routing::PhasorBInput;
pub use sweep::{LfoShape, RateControl, SweepGenerator, SweepSync, SyncRate};

#[derive(Params)]
pub struct MutanteParams {
    // --- Sweep Generator 1 ---
    #[param(
        name = "Gen 1 Rate",
        short_name = "Rate 1",
        range = "linear(0.1, 18.0)",
        default = 0.25,
        unit = "Hz",
        smooth = "exp(10)"
    )]
    pub gen_1_rate: FloatParam,

    #[param(name = "Gen 1 Sync", short_name = "Sync 1", default = false)]
    pub gen_1_sync: BoolParam,

    #[param(name = "Gen 1 Synced Rate", short_name = "SyncDiv 1")]
    pub gen_1_synced_rate: EnumParam<SyncRate>,

    #[param(name = "Gen 1 Shape", short_name = "Shape 1")]
    pub gen_1_shape: EnumParam<LfoShape>,

    #[param(name = "Gen 1 Rate Ctrl", short_name = "RateCtrl 1")]
    pub gen_1_rate_ctrl: EnumParam<RateControl>,

    // --- Sweep Generator 2 ---
    #[param(
        name = "Gen 2 Rate",
        short_name = "Rate 2",
        range = "linear(0.1, 18.0)",
        default = 0.40,
        unit = "Hz",
        smooth = "exp(10)"
    )]
    pub gen_2_rate: FloatParam,

    #[param(name = "Gen 2 Sync", short_name = "Sync 2", default = false)]
    pub gen_2_sync: BoolParam,

    #[param(name = "Gen 2 Synced Rate", short_name = "SyncDiv 2")]
    pub gen_2_synced_rate: EnumParam<SyncRate>,

    #[param(name = "Gen 2 Shape", short_name = "Shape 2")]
    pub gen_2_shape: EnumParam<LfoShape>,

    // --- Phasor A ---
    #[param(
        name = "Phasor A Depth",
        short_name = "Depth A",
        range = "linear(0.0, 1.0)",
        default = 0.85,
        smooth = "exp(10)"
    )]
    pub phasor_a_depth: FloatParam,

    #[param(
        name = "Phasor A Feedback",
        short_name = "Fdbk A",
        range = "linear(0.0, 1.0)",
        default = 0.0,
        smooth = "exp(10)"
    )]
    pub phasor_a_feedback: FloatParam,

    #[param(name = "Phasor A Sweep Source", short_name = "Sweep A")]
    pub phasor_a_sweep: EnumParam<PhasorASweepSource>,

    #[param(name = "Phasor A Poles", short_name = "Poles A")]
    pub phasor_a_poles: EnumParam<PolesCount>,

    #[param(name = "Phasor A Bypass", short_name = "Bypass A", default = false)]
    pub phasor_a_bypass: BoolParam,

    // --- Phasor B ---
    #[param(
        name = "Phasor B Depth",
        short_name = "Depth B",
        range = "linear(0.0, 1.0)",
        default = 0.85,
        smooth = "exp(10)"
    )]
    pub phasor_b_depth: FloatParam,

    #[param(
        name = "Phasor B Feedback",
        short_name = "Fdbk B",
        range = "linear(0.0, 1.0)",
        default = 0.0,
        smooth = "exp(10)"
    )]
    pub phasor_b_feedback: FloatParam,

    #[param(name = "Phasor B Sweep Source", short_name = "Sweep B")]
    pub phasor_b_sweep: EnumParam<PhasorBSweepSource>,

    #[param(name = "Phasor B Sweep Sync", short_name = "Sync B")]
    pub phasor_b_sweep_sync: EnumParam<SweepSync>,

    #[param(name = "Phasor B Poles", short_name = "Poles B")]
    pub phasor_b_poles: EnumParam<PolesCount>,

    #[param(name = "Phasor B Bypass", short_name = "Bypass B", default = false)]
    pub phasor_b_bypass: BoolParam,

    // --- Pedal / Envelope Follower ---
    #[param(name = "Pedal Mode", short_name = "Pedal Mode")]
    pub pedal_mode: EnumParam<PedalMode>,

    #[param(
        name = "Pedal Manual",
        short_name = "Pedal Pos",
        range = "linear(0.0, 1.0)",
        default = 0.5,
        smooth = "exp(10)"
    )]
    pub pedal_manual: FloatParam,

    #[param(
        name = "Envelope Sensitivity",
        short_name = "Env Sens",
        range = "linear(0.1, 10.0)",
        default = 1.5,
        smooth = "exp(10)"
    )]
    pub pedal_sensitivity: FloatParam,

    // --- Master & Routing ---
    #[param(name = "Phasor B Input Routing", short_name = "Routing")]
    pub routing: EnumParam<PhasorBInput>,

    #[param(
        name = "Analog Drive",
        short_name = "Drive",
        range = "linear(0.0, 1.0)",
        default = 0.1,
        smooth = "exp(10)"
    )]
    pub drive: FloatParam,

    #[param(
        name = "Mix (Dry/Wet)",
        short_name = "Mix",
        range = "linear(0.0, 1.0)",
        default = 1.0,
        unit = "%",
        smooth = "exp(10)"
    )]
    pub mix: FloatParam,

    #[param(
        name = "Output Gain",
        short_name = "Out Gain",
        range = "linear(-12.0, 12.0)",
        default = 0.0,
        unit = "dB",
        smooth = "exp(10)"
    )]
    pub output_gain: FloatParam,

    #[param(name = "Master Bypass", short_name = "Bypass", default = false)]
    pub bypass: BoolParam,

    // --- Visual Meters ---
    #[meter]
    pub lfo1_meter: MeterSlot,

    #[meter]
    pub lfo2_meter: MeterSlot,

    #[meter]
    pub optical_a_meter: MeterSlot,

    #[meter]
    pub optical_b_meter: MeterSlot,

    #[meter]
    pub out_meter_l: MeterSlot,

    #[meter]
    pub out_meter_r: MeterSlot,
}

use MutanteParamsParamId as P;

pub struct Mutante;

pub struct MutanteDsp {
    pub sweep_gen_1: SweepGenerator,
    pub sweep_gen_2: SweepGenerator,
    pub pedal_follower: PedalFollower,
    pub phasor_a: PhasorChannel,
    pub phasor_b: PhasorChannel,
    pub sample_rate: f32,

    // Metering state
    last_lfo1: f32,
    last_lfo2: f32,
    peak_out_l: f32,
    peak_out_r: f32,
}

impl Default for MutanteDsp {
    fn default() -> Self {
        let sr = 48000.0;
        Self {
            sweep_gen_1: SweepGenerator::new(sr),
            sweep_gen_2: SweepGenerator::new(sr),
            pedal_follower: PedalFollower::new(sr),
            phasor_a: PhasorChannel::new(sr, false),
            phasor_b: PhasorChannel::new(sr, true),
            sample_rate: sr,
            last_lfo1: 0.0,
            last_lfo2: 0.0,
            peak_out_l: 0.0,
            peak_out_r: 0.0,
        }
    }
}

impl PluginLogic for Mutante {
    type Params = MutanteParams;
    type DspState = MutanteDsp;

    fn reset(state: &mut Self::DspState, _params: &Self::Params, config: &AudioConfig) {
        state.sample_rate = config.sample_rate as f32;
        state.sweep_gen_1.set_sample_rate(state.sample_rate);
        state.sweep_gen_2.set_sample_rate(state.sample_rate);
        state.pedal_follower.set_sample_rate(state.sample_rate);
        state.phasor_a.set_sample_rate(state.sample_rate);
        state.phasor_b.set_sample_rate(state.sample_rate);

        state.sweep_gen_1.reset();
        state.sweep_gen_2.reset();
        state.pedal_follower.reset();
        state.phasor_a.reset();
        state.phasor_b.reset();
        state.peak_out_l = 0.0;
        state.peak_out_r = 0.0;
    }

    fn process(
        state: &mut Self::DspState,
        params: &Self::Params,
        buffer: &mut AudioBuffer,
        _events: &EventList,
        context: &mut ProcessContext,
    ) -> ProcessStatus {
        let num_channels = buffer.channels();

        // 1. Bit-clean hard bypass
        if params.bypass.value() {
            let n_in = buffer.num_input_channels();
            for ch in 0..num_channels {
                let (inp, out) = buffer.io(ch);
                if ch < n_in {
                    out.copy_from_slice(inp);
                } else {
                    out.fill(0.0);
                }
            }
            context.set_meter(P::Lfo1Meter, 0.0);
            context.set_meter(P::Lfo2Meter, 0.0);
            context.set_meter(P::OpticalAMeter, 0.0);
            context.set_meter(P::OpticalBMeter, 0.0);
            context.set_meter(P::OutMeterL, 0.0);
            context.set_meter(P::OutMeterR, 0.0);
            return ProcessStatus::Normal;
        }

        // Host BPM for tempo sync
        let bpm = if context.transport.tempo > 10.0 {
            context.transport.tempo as f32
        } else {
            120.0
        };

        // Effective LFO 1 frequency
        let base_freq_1 = if params.gen_1_sync.value() {
            let beats = params.gen_1_synced_rate.value().beats();
            (bpm / 60.0) / beats
        } else {
            params.gen_1_rate.value()
        };

        // Effective LFO 2 frequency
        let freq_2 = if params.gen_2_sync.value() {
            let beats = params.gen_2_synced_rate.value().beats();
            (bpm / 60.0) / beats
        } else {
            params.gen_2_rate.value()
        };

        let shape_1 = params.gen_1_shape.value();
        let shape_2 = params.gen_2_shape.value();
        let rate_ctrl_1 = params.gen_1_rate_ctrl.value();

        let depth_a = params.phasor_a_depth.value();
        let fdbk_a = params.phasor_a_feedback.value();
        let sweep_src_a = params.phasor_a_sweep.value();
        let poles_a = params.phasor_a_poles.value();
        let bypass_a = params.phasor_a_bypass.value();

        let depth_b = params.phasor_b_depth.value();
        let fdbk_b = params.phasor_b_feedback.value();
        let sweep_src_b = params.phasor_b_sweep.value();
        let sweep_sync_b = params.phasor_b_sweep_sync.value();
        let poles_b = params.phasor_b_poles.value();
        let bypass_b = params.phasor_b_bypass.value();

        let pedal_mode = params.pedal_mode.value();
        let pedal_man = params.pedal_manual.value();
        let pedal_sens = params.pedal_sensitivity.value();

        let routing = params.routing.value();
        let drive = params.drive.value();
        let mix = params.mix.value();
        let out_gain_db = params.output_gain.value();
        let out_gain = 10.0f32.powf(out_gain_db / 20.0);

        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;

        // Process audio frames with stack allocation (no audio-thread heap allocations)
        buffer.for_each_frame_io::<2, 2, _>(|frame_in, frame_out| {
            let in_l_raw = frame_in[0];
            let in_r_raw = frame_in[1];

            // Analog input saturation stage (op-amp + input transformer warmth)
            let in_l = if drive > 0.001 {
                let d = 1.0 + drive * 3.0;
                (in_l_raw * d).tanh() / (1.0 + drive * 0.5)
            } else {
                in_l_raw
            };

            let in_r = if drive > 0.001 {
                let d = 1.0 + drive * 3.0;
                (in_r_raw * d).tanh() / (1.0 + drive * 0.5)
            } else {
                in_r_raw
            };

            // Calculate pedal CV
            let pedal_input = 0.5 * (in_l.abs() + in_r.abs());
            let pedal_cv = state
                .pedal_follower
                .process(pedal_input, pedal_mode, pedal_man, pedal_sens);

            // Frequency for LFO 1: either manual or pedal-modulated
            let freq_1 = match rate_ctrl_1 {
                RateControl::Manual => base_freq_1,
                RateControl::Pedal => (0.1 + pedal_cv * 17.9).clamp(0.1, 18.0),
            };

            // Advance LFOs
            let lfo1_val = state.sweep_gen_1.tick(freq_1, shape_1);
            let lfo2_val = state.sweep_gen_2.tick(freq_2, shape_2);
            state.last_lfo1 = lfo1_val;
            state.last_lfo2 = lfo2_val;

            // Determine Phasor A sweep CV
            let sweep_a = match sweep_src_a {
                PhasorASweepSource::Gen1 => lfo1_val,
                PhasorASweepSource::Pedal => pedal_cv,
            };

            // Determine Phasor B sweep CV
            let base_sweep_b = match sweep_src_b {
                PhasorBSweepSource::Gen1 => lfo1_val,
                PhasorBSweepSource::Gen2 => lfo2_val,
                PhasorBSweepSource::Pedal => pedal_cv,
            };

            // Apply Phasor B Sweep Sync (Norm vs Rev)
            let sweep_b = match sweep_sync_b {
                SweepSync::Norm => base_sweep_b,
                SweepSync::Rev => 1.0 - base_sweep_b,
            };

            // Routing Matrix
            let (out_l_proc, out_r_proc) = match routing {
                PhasorBInput::InA => {
                    // Mono Input A feeds both Phasor A and Phasor B
                    let (pa_out, _) = state.phasor_a.process_sample(
                        in_l, sweep_a, depth_a, fdbk_a, poles_a, bypass_a,
                    );
                    let (pb_out, _) = state.phasor_b.process_sample(
                        in_l, sweep_b, depth_b, fdbk_b, poles_b, bypass_b,
                    );
                    (pa_out, pb_out)
                }
                PhasorBInput::OutA => {
                    // Series Cascade: Phasor A feeds directly into Phasor B (12-stage double phasing!)
                    let (pa_out, pa_wet) = state.phasor_a.process_sample(
                        in_l, sweep_a, depth_a, fdbk_a, poles_a, bypass_a,
                    );
                    // Phasor B receives the phase-shifted output of Phasor A
                    let (pb_out, _) = state.phasor_b.process_sample(
                        pa_wet, sweep_b, depth_b, fdbk_b, poles_b, bypass_b,
                    );
                    (pa_out, pb_out)
                }
                PhasorBInput::InB => {
                    // Dual Mono / Independent Stereo
                    let (pa_out, _) = state.phasor_a.process_sample(
                        in_l, sweep_a, depth_a, fdbk_a, poles_a, bypass_a,
                    );
                    let (pb_out, _) = state.phasor_b.process_sample(
                        in_r, sweep_b, depth_b, fdbk_b, poles_b, bypass_b,
                    );
                    (pa_out, pb_out)
                }
                PhasorBInput::StereoParallel => {
                    // Stereo cross-mixed parallel
                    let (pa_l, _) = state.phasor_a.process_sample(
                        in_l, sweep_a, depth_a, fdbk_a, poles_a, bypass_a,
                    );
                    let (pb_r, _) = state.phasor_b.process_sample(
                        in_r, sweep_b, depth_b, fdbk_b, poles_b, bypass_b,
                    );
                    // Cross blend for maximum stereo richness
                    let sum_l = 0.7 * pa_l + 0.3 * pb_r;
                    let sum_r = 0.7 * pb_r + 0.3 * pa_l;
                    (sum_l, sum_r)
                }
            };

            // Global Dry/Wet blend and output gain
            let final_l = ((1.0 - mix) * in_l_raw + mix * out_l_proc) * out_gain;
            let final_r = ((1.0 - mix) * in_r_raw + mix * out_r_proc) * out_gain;

            frame_out[0] = final_l;
            frame_out[1] = final_r;

            peak_l = peak_l.max(final_l.abs());
            peak_r = peak_r.max(final_r.abs());
        });

        // Update meters
        state.peak_out_l = state.peak_out_l * 0.85 + peak_l * 0.15;
        state.peak_out_r = state.peak_out_r * 0.85 + peak_r * 0.15;

        context.set_meter(P::Lfo1Meter, state.last_lfo1);
        context.set_meter(P::Lfo2Meter, state.last_lfo2);
        context.set_meter(P::OpticalAMeter, state.phasor_a.optocoupler.brightness());
        context.set_meter(P::OpticalBMeter, state.phasor_b.optocoupler.brightness());
        context.set_meter(P::OutMeterL, (state.peak_out_l * 1.2).min(1.0));
        context.set_meter(P::OutMeterR, (state.peak_out_r * 1.2).min(1.0));

        ProcessStatus::Normal
    }

    fn editor(params: Arc<MutanteParams>) -> Box<dyn Editor> {
        // Ergonomic 19" rack-style vintage Mu-Tron layout:
        let layout = GridLayout::build(vec![
            // --- Row 0: Sweep Generators ---
            widgets(vec![
                knob(P::Gen1Rate, "Rate 1").at(0, 0),
                dropdown(P::Gen1Shape, "Shape 1").at(1, 0),
                dropdown(P::Gen1RateCtrl, "Rate Ctrl 1").at(2, 0),
                toggle(P::Gen1Sync, "Sync 1").at(3, 0),
                dropdown(P::Gen1SyncedRate, "Div 1").at(4, 0),
                meter(&[P::Lfo1Meter], "LFO 1").at(5, 0),

                knob(P::Gen2Rate, "Rate 2").at(6, 0),
                dropdown(P::Gen2Shape, "Shape 2").at(7, 0),
                toggle(P::Gen2Sync, "Sync 2").at(8, 0),
                dropdown(P::Gen2SyncedRate, "Div 2").at(9, 0),
                meter(&[P::Lfo2Meter], "LFO 2").at(10, 0),
            ]),

            // --- Row 1: Phasor Engines ---
            widgets(vec![
                knob(P::PhasorADepth, "Depth A").at(0, 1),
                knob(P::PhasorAFeedback, "Feedback A").at(1, 1),
                dropdown(P::PhasorASweep, "Sweep A").at(2, 1),
                dropdown(P::PhasorAPoles, "Poles A").at(3, 1),
                meter(&[P::OpticalAMeter], "Opto A").at(4, 1),
                toggle(P::PhasorABypass, "Phasor A").at(5, 1),

                knob(P::PhasorBDepth, "Depth B").at(6, 1),
                knob(P::PhasorBFeedback, "Feedback B").at(7, 1),
                dropdown(P::PhasorBSweep, "Sweep B").at(8, 1),
                dropdown(P::PhasorBSweepSync, "Sync B").at(9, 1),
                dropdown(P::PhasorBPoles, "Poles B").at(10, 1),
                meter(&[P::OpticalBMeter], "Opto B").at(11, 1),
                toggle(P::PhasorBBypass, "Phasor B").at(12, 1),
            ]),

            // --- Row 2: Master Routing, Expression & Output ---
            widgets(vec![
                dropdown(P::Routing, "Routing (S1)").at(0, 2),
                dropdown(P::PedalMode, "Pedal Mode").at(1, 2),
                knob(P::PedalManual, "Pedal Pos").at(2, 2),
                knob(P::PedalSensitivity, "Env Sens").at(3, 2),
                knob(P::Drive, "Analog Drive").at(4, 2),
                knob(P::Mix, "Mix (Dry/Wet)").at(5, 2),
                knob(P::OutputGain, "Output Gain").at(6, 2),
                meter(&[P::OutMeterL], "Out L").at(7, 2),
                meter(&[P::OutMeterR], "Out R").at(8, 2),
                toggle(P::Bypass, "Master Bypass").at(9, 2),
            ]),
        ])
        .with_title("MUTANTE")
        .with_subtitle("MU-TRON BI-PHASE OPTICAL EMULATION - JARDIM SONORO")
        .resizable(false);

        // Vintage Mu-Tron aesthetic theme: Deep Slate Charcoal with Electric Cyan / Amber accents
        let mutron_theme = truce_gui::Theme {
            background: truce_gui::theme::Color::rgb(0.09, 0.10, 0.12), // Deep Charcoal Blue #171A1F
            surface: truce_gui::theme::Color::rgb(0.15, 0.17, 0.20),    // Vintage module metal #262B33
            primary: truce_gui::theme::Color::rgb(0.18, 0.78, 0.95),    // Musitronics Electric Cyan #2EC7F2
            accent: truce_gui::theme::Color::rgb(1.00, 0.65, 0.15),     // Vintage Amber Warning #FFA626
            text: truce_gui::theme::Color::rgb(0.95, 0.96, 0.98),       // Crisp Stencil White #F2F5FA
            text_dim: truce_gui::theme::Color::rgb(0.60, 0.65, 0.70),   // Brushed Aluminum Grey #99A6B3
            knob_track: truce_gui::theme::Color::rgb(0.18, 0.21, 0.25), // Recessed Track #2E3640
            knob_fill: truce_gui::theme::Color::rgb(0.18, 0.78, 0.95),  // Glowing Cyan Arc
            knob_pointer: truce_gui::theme::Color::rgb(1.0, 1.0, 1.0),  // Pure White Indicator
            header_bg: truce_gui::theme::Color::rgb(0.05, 0.06, 0.08),  // Dark Top Stencil Bar
            header_text: truce_gui::theme::Color::rgb(0.95, 0.96, 0.98),// Stencil Title Text
        };

        Box::new(BuiltinEditor::new_grid(params, layout).with_theme(mutron_theme))
    }
}

truce::plugin! {
    logic: Mutante,
    params: MutanteParams,
}

truce::enable_rt_paranoid!();
