//! Editor: the Bi-Phase front panel re-drawn as a flat, modern instrument
//! panel in the spirit of Ableton Live's devices.
//!
//! The section layout mirrors the hardware (Sweep Generator 1 wide on the
//! left, Sweep Gen 2 on the right, Phasor A below it, Phasor B with its
//! "Phasor B Input" and sweep-switch blocks), drawn with the original's
//! silver section outlines and Mu-Tron blue. A live response display on top
//! shows where the notches are, computed from the photocell state the DSP
//! reports, and the footswitch LEDs follow the real optical fade of PM1/PM2.

use crate::circuit::*;
use crate::{MutanteParams, MutanteParamsParamId as P};
use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2};
use std::sync::Arc;
use truce::prelude::*;
use truce_egui::EguiEditor;

const W: f32 = 1120.0;
const H: f32 = 648.0;

// --- Palette ---------------------------------------------------------------
const BG: Color32 = Color32::from_rgb(24, 25, 28);
const PANEL: Color32 = Color32::from_rgb(35, 37, 41);
const PANEL_DARK: Color32 = Color32::from_rgb(20, 21, 24);
const OUTLINE: Color32 = Color32::from_rgb(132, 138, 148);
const OUTLINE_DIM: Color32 = Color32::from_rgb(64, 67, 74);
const TEXT: Color32 = Color32::from_rgb(220, 223, 228);
const TEXT_DIM: Color32 = Color32::from_rgb(138, 144, 154);
const TRACK: Color32 = Color32::from_rgb(58, 61, 68);
const KNOB_BODY: Color32 = Color32::from_rgb(16, 17, 19);
/// Mu-Tron panel blue, brightened for a dark UI.
const BLUE: Color32 = Color32::from_rgb(70, 132, 255);
const AMBER: Color32 = Color32::from_rgb(255, 166, 64);
const LED_RED: Color32 = Color32::from_rgb(255, 59, 48);
const SILVER: Color32 = Color32::from_rgb(201, 205, 211);

pub fn create_editor(params: Arc<MutanteParams>) -> Box<dyn Editor> {
    Box::new(
        EguiEditor::with_ui(params, (W as u32, H as u32), MutanteUi).with_visuals(visuals()),
    )
}

fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.override_text_color = Some(TEXT);
    v
}

struct MutanteUi;

impl truce_egui::EditorUi<MutanteParams> for MutanteUi {
    fn ui(&mut self, ui: &mut egui::Ui, st: &PluginContext<MutanteParams>) {
        // Meters and photocells move on their own.
        ui.ctx().request_repaint();
        let painter = ui.painter().clone();
        painter.rect_filled(Rect::from_min_size(Pos2::ZERO, vec2(W, H)), 0.0, BG);

        header(ui, st);
        response_display(ui, st, Rect::from_min_size(pos2(16.0, 58.0), vec2(W - 32.0, 140.0)));

        sweep_gen_1(ui, st, Rect::from_min_size(pos2(16.0, 210.0), vec2(512.0, 156.0)));
        sweep_gen_2(ui, st, Rect::from_min_size(pos2(540.0, 210.0), vec2(348.0, 156.0)));
        phasor_a(ui, st, Rect::from_min_size(pos2(16.0, 378.0), vec2(360.0, 254.0)));
        phasor_b(ui, st, Rect::from_min_size(pos2(388.0, 378.0), vec2(500.0, 254.0)));
        pedal_section(ui, st, Rect::from_min_size(pos2(904.0, 210.0), vec2(200.0, 230.0)));
        studio_section(ui, st, Rect::from_min_size(pos2(904.0, 452.0), vec2(200.0, 180.0)));
    }
}

// ===========================================================================
// Sections
// ===========================================================================

fn header(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>) {
    let p = ui.painter();
    let band = Rect::from_min_size(pos2(0.0, 0.0), vec2(W, 44.0));
    p.rect_filled(band, 0.0, SILVER);
    p.line_segment(
        [pos2(0.0, 44.0), pos2(W, 44.0)],
        stroke(1.0, Color32::from_rgb(120, 124, 130)),
    );
    mutron_mark(p, pos2(30.0, 22.0), 12.0);
    p.text(
        pos2(50.0, 22.0),
        Align2::LEFT_CENTER,
        "MUTANTE",
        FontId::proportional(22.0),
        Color32::from_rgb(28, 30, 34),
    );
    p.text(
        pos2(160.0, 23.0),
        Align2::LEFT_CENTER,
        "BI-PHASE  ·  DUAL OPTICAL PHASOR",
        FontId::proportional(11.0),
        Color32::from_rgb(70, 74, 82),
    );
    p.text(
        pos2(W - 150.0, 23.0),
        Align2::RIGHT_CENTER,
        "JARDIM SONORO",
        FontId::proportional(10.0),
        Color32::from_rgb(90, 94, 102),
    );
    toggle_button(
        ui,
        st,
        P::Bypass,
        "BYPASS",
        Rect::from_min_size(pos2(W - 128.0, 11.0), vec2(112.0, 22.0)),
        AMBER,
    );
}

/// The three-colour "U" of the Mu-tron badge, simplified.
fn mutron_mark(p: &egui::Painter, c: Pos2, s: f32) {
    let colors = [
        Color32::from_rgb(40, 170, 90),
        Color32::from_rgb(40, 100, 220),
        Color32::from_rgb(220, 50, 50),
    ];
    for (i, col) in colors.iter().enumerate() {
        let x = c.x - s + i as f32 * s * 0.7;
        let r = Rect::from_min_size(pos2(x, c.y - s), vec2(s * 0.55, s * 2.0));
        p.rect_filled(r, CornerRadius::same(2), *col);
    }
}

fn sweep_gen_1(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    section(ui.painter(), r, "SWEEP GENERATOR 1");
    activity_lamp(ui.painter(), pos2(r.left() + 168.0, r.top() + 14.0), st.get_meter(P::Lfo1Meter));
    let synced = st.get_param(P::Gen1Sync) > 0.5;
    knob(ui, st, P::Gen1Rate, "RATE", pos2(r.left() + 72.0, r.top() + 82.0), 32.0, BLUE, !synced);
    rocker(
        ui,
        st,
        P::Gen1RateCtrl,
        RockerLabels::Text("MAN", "PED"),
        "RATE CONTROL",
        pos2(r.left() + 190.0, r.top() + 80.0),
    );
    rocker(
        ui,
        st,
        P::Gen1Shape,
        RockerLabels::Waves,
        "SHAPE",
        pos2(r.left() + 286.0, r.top() + 80.0),
    );
    sync_block(ui, st, P::Gen1Sync, P::Gen1SyncedRate, pos2(r.left() + 360.0, r.top() + 46.0));
}

fn sweep_gen_2(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    section(ui.painter(), r, "SWEEP GEN 2");
    activity_lamp(ui.painter(), pos2(r.left() + 108.0, r.top() + 14.0), st.get_meter(P::Lfo2Meter));
    rocker(
        ui,
        st,
        P::Gen2Shape,
        RockerLabels::Waves,
        "SHAPE",
        pos2(r.left() + 50.0, r.top() + 80.0),
    );
    let synced = st.get_param(P::Gen2Sync) > 0.5;
    knob(ui, st, P::Gen2Rate, "RATE", pos2(r.left() + 150.0, r.top() + 82.0), 32.0, BLUE, !synced);
    sync_block(ui, st, P::Gen2Sync, P::Gen2SyncedRate, pos2(r.left() + 222.0, r.top() + 46.0));
}

fn phasor_a(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    section(ui.painter(), r, "PHASOR A");
    knob(ui, st, P::PhasorADepth, "DEPTH", pos2(r.left() + 66.0, r.top() + 76.0), 28.0, BLUE, true);
    knob(ui, st, P::PhasorAFeedback, "FEEDBACK", pos2(r.left() + 66.0, r.top() + 180.0), 28.0, BLUE, true);

    let block = Rect::from_min_size(pos2(r.left() + 134.0, r.top() + 112.0), vec2(96.0, 126.0));
    switch_block(ui.painter(), block, "SWEEP");
    rocker(
        ui,
        st,
        P::PhasorASweep,
        RockerLabels::Text("GEN 1", "PED"),
        "",
        pos2(block.center().x, block.top() + 56.0),
    );

    pod_glow(ui.painter(), pos2(r.left() + 182.0, r.top() + 64.0), st.get_meter(P::PodAMeter), BLUE);
    footswitch(ui, st, P::PhasorAOn, P::LampAMeter, pos2(r.left() + 300.0, r.top() + 150.0));
}

fn phasor_b(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    section(ui.painter(), r, "PHASOR B");

    let input_block = Rect::from_min_size(pos2(r.left() + 14.0, r.top() + 112.0), vec2(118.0, 126.0));
    switch_block(ui.painter(), input_block, "PHASOR B INPUT");
    selector(
        ui,
        st,
        P::Routing,
        &["IN A", "OUT A", "IN B"],
        pos2(input_block.center().x, input_block.top() + 66.0),
    );

    let sweep_block = Rect::from_min_size(pos2(r.left() + 144.0, r.top() + 34.0), vec2(124.0, 204.0));
    switch_block(ui.painter(), sweep_block, "SWEEP");
    rocker(
        ui,
        st,
        P::PhasorBSweepSync,
        RockerLabels::Text("NORM", "REV"),
        "SWEEP SYNC",
        pos2(sweep_block.center().x, sweep_block.top() + 50.0),
    );
    selector(
        ui,
        st,
        P::PhasorBSweep,
        &["GEN 1", "GEN 2", "PED"],
        pos2(sweep_block.center().x, sweep_block.top() + 160.0),
    );

    pod_glow(ui.painter(), pos2(r.left() + 72.0, r.top() + 64.0), st.get_meter(P::PodBMeter), AMBER);
    knob(ui, st, P::PhasorBDepth, "DEPTH", pos2(r.left() + 336.0, r.top() + 76.0), 28.0, AMBER, true);
    knob(ui, st, P::PhasorBFeedback, "FEEDBACK", pos2(r.left() + 336.0, r.top() + 180.0), 28.0, AMBER, true);
    footswitch(ui, st, P::PhasorBOn, P::LampBMeter, pos2(r.left() + 440.0, r.top() + 150.0));
}

fn pedal_section(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    section(ui.painter(), r, "C-100 PEDAL");
    pedal_fader(ui, st, Rect::from_min_size(pos2(r.left() + 22.0, r.top() + 38.0), vec2(48.0, 176.0)));
    let envelope = st.get_param(P::PedalMode) > 0.5;
    segmented(
        ui,
        st,
        P::PedalMode,
        &["MANUAL", "ENV"],
        Rect::from_min_size(pos2(r.left() + 88.0, r.top() + 44.0), vec2(98.0, 22.0)),
    );
    knob(
        ui,
        st,
        P::PedalSensitivity,
        "ENV SENS",
        pos2(r.left() + 137.0, r.top() + 128.0),
        22.0,
        BLUE,
        envelope,
    );
}

fn studio_section(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    section(ui.painter(), r, "STUDIO");
    let y = r.top() + 70.0;
    knob(ui, st, P::InputLevel, "INPUT", pos2(r.left() + 36.0, y), 19.0, BLUE, true);
    knob(ui, st, P::Mix, "MIX", pos2(r.left() + 100.0, y), 19.0, BLUE, true);
    knob(ui, st, P::OutputGain, "OUTPUT", pos2(r.left() + 164.0, y), 19.0, BLUE, true);
    let meter_rect = Rect::from_min_size(pos2(r.left() + 16.0, r.top() + 136.0), vec2(r.width() - 32.0, 26.0));
    stereo_meter(ui.painter(), st, meter_rect);
}

// ===========================================================================
// Response display
// ===========================================================================

#[derive(Clone, Copy)]
struct Cx {
    re: f32,
    im: f32,
}

impl Cx {
    const ONE: Cx = Cx { re: 1.0, im: 0.0 };
    fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
    fn add(self, o: Cx) -> Cx {
        Cx::new(self.re + o.re, self.im + o.im)
    }
    fn sub(self, o: Cx) -> Cx {
        Cx::new(self.re - o.re, self.im - o.im)
    }
    fn mul(self, o: Cx) -> Cx {
        Cx::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    fn scale(self, k: f32) -> Cx {
        Cx::new(self.re * k, self.im * k)
    }
    fn div(self, o: Cx) -> Cx {
        let d = o.re * o.re + o.im * o.im;
        Cx::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }
    fn abs(self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

/// Analog transfer function of one phasor channel (same network as the
/// DSP, with the six stages taken at the pod's centre frequency).
fn phasor_response(f: f32, fc: f32, feedback_pos: f32, wet_on: bool, r123: f32) -> Cx {
    if !wet_on {
        return Cx::ONE;
    }
    let two_pi = std::f32::consts::TAU;
    let s = Cx::new(0.0, two_pi * f);
    let first_order_hp = |w: f32| s.div(s.add(Cx::new(w, 0.0)));
    let ap1 = s.scale(1.0 / (two_pi * fc)).sub(Cx::ONE).div(s.scale(1.0 / (two_pi * fc)).add(Cx::ONE));
    let ap2 = ap1.mul(ap1);
    let ap6 = ap2.mul(ap2).mul(ap2);
    let hp12 = first_order_hp(1.0 / (j1_load_resistance(feedback_pos) * C12));
    let (ratio, r_th) = pot_divider(feedback_pos, R_FB_POT);
    let r_src = R7 + r_th;
    let k_fb = ratio * R6 / (R6 + r_src);
    let w5 = 1.0 / (C5 * R6 * r_src / (R6 + r_src));
    let lp5 = Cx::ONE.div(Cx::ONE.add(s.scale(1.0 / w5)));
    let loop_gain = lp5.mul(hp12).mul(ap6).scale(2.0 * k_fb);
    let u = Cx::new(-1.0, 0.0).div(Cx::ONE.sub(loop_gain));
    let wet = hp12.mul(ap6).mul(u).scale(-1.0);
    let r_wet = R_PM_ON + r123 + R29;
    let k = R31 / (R31 + r_wet);
    let hp14 = first_order_hp(1.0 / ((R31 + r_wet) * C14));
    Cx::ONE.add(hp14.mul(wet.sub(Cx::ONE)).scale(k))
}

fn meter_to_hz(m: f32) -> f32 {
    30.0 * (20_000.0f32 / 30.0).powf(m.clamp(0.0, 1.0))
}

fn response_display(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    let p = ui.painter();
    p.rect_filled(r, CornerRadius::same(4), PANEL_DARK);
    p.rect_stroke(r, CornerRadius::same(4), stroke(1.0, OUTLINE_DIM), StrokeKind::Inside);

    let plot = r.shrink2(vec2(14.0, 12.0));
    let (f_lo, f_hi) = (20.0f32, 20_000.0f32);
    let (db_lo, db_hi) = (-36.0f32, 15.0f32);
    let x_of = |f: f32| plot.left() + plot.width() * (f / f_lo).ln() / (f_hi / f_lo).ln();
    let y_of = |db: f32| plot.bottom() - plot.height() * (db.clamp(db_lo, db_hi) - db_lo) / (db_hi - db_lo);

    for &f in &[50.0f32, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10_000.0] {
        let x = x_of(f);
        p.line_segment([pos2(x, plot.top()), pos2(x, plot.bottom())], stroke(1.0, Color32::from_gray(38)));
        let label = if f >= 1000.0 { format!("{}k", f / 1000.0) } else { format!("{f}") };
        p.text(pos2(x + 3.0, plot.bottom() - 2.0), Align2::LEFT_BOTTOM, label, FontId::proportional(9.0), TEXT_DIM);
    }
    for &db in &[-24.0f32, -12.0, 0.0, 12.0] {
        let y = y_of(db);
        let col = if db == 0.0 { Color32::from_gray(58) } else { Color32::from_gray(36) };
        p.line_segment([pos2(plot.left(), y), pos2(plot.right(), y)], stroke(1.0, col));
    }

    let routing = (st.get_param(P::Routing) * 2.0).round() as i32;
    // Before the DSP has run (no meters yet) fall back to the panel state
    // with the pods at mid-sweep.
    let running = st.get_meter(P::NotchAMeter) > 0.0 || st.get_meter(P::NotchBMeter) > 0.0;
    let (fc_a, fc_b, on_a, on_b) = if running {
        (
            meter_to_hz(st.get_meter(P::NotchAMeter)),
            meter_to_hz(st.get_meter(P::NotchBMeter)),
            st.get_meter(P::LampAMeter) > 0.5,
            st.get_meter(P::LampBMeter) > 0.5,
        )
    } else {
        (
            1450.0,
            1250.0,
            st.get_param(P::PhasorAOn) > 0.5,
            st.get_param(P::PhasorBOn) > 0.5,
        )
    };
    let fb_a = st.get_param_plain(P::PhasorAFeedback) * 0.1;
    let fb_b = st.get_param_plain(P::PhasorBFeedback) * 0.1;

    let n = 220;
    let mut pts_a = Vec::with_capacity(n);
    let mut pts_b = Vec::with_capacity(n);
    let mut pts_series = Vec::with_capacity(n);
    for i in 0..n {
        let f = f_lo * (f_hi / f_lo).powf(i as f32 / (n - 1) as f32);
        let ha = phasor_response(f, fc_a, fb_a, on_a, R123_A);
        let hb = phasor_response(f, fc_b, fb_b, on_b, R123_B);
        let db = |h: Cx| 20.0 * h.abs().max(1e-4).log10();
        let x = x_of(f);
        pts_a.push(pos2(x, y_of(db(ha))));
        pts_b.push(pos2(x, y_of(db(hb))));
        pts_series.push(pos2(x, y_of(db(ha.mul(hb)))));
    }

    if routing == 1 {
        p.add(egui::Shape::line(pts_a, stroke(1.0, BLUE.gamma_multiply(0.35))));
        p.add(egui::Shape::line(pts_b, stroke(1.0, AMBER.gamma_multiply(0.35))));
        p.add(egui::Shape::line(pts_series, stroke(2.0, SILVER)));
    } else {
        p.add(egui::Shape::line(pts_b, stroke(2.0, AMBER)));
        p.add(egui::Shape::line(pts_a, stroke(2.0, BLUE)));
    }

    let legend = match routing {
        0 => "IN A  ·  A TO LEFT, B TO RIGHT",
        1 => "OUT A  ·  A INTO B IN SERIES, B TO BOTH",
        _ => "IN B  ·  LEFT INTO A, RIGHT INTO B",
    };
    p.text(pos2(plot.left() + 4.0, plot.top() + 2.0), Align2::LEFT_TOP, legend, FontId::proportional(10.0), TEXT_DIM);
    let dot = |x: f32, col: Color32, label: &str| {
        p.circle_filled(pos2(x, plot.top() + 8.0), 4.0, col);
        p.text(pos2(x + 8.0, plot.top() + 8.0), Align2::LEFT_CENTER, label, FontId::proportional(10.0), TEXT);
    };
    dot(plot.right() - 150.0, BLUE, &format!("A {:>5.0} Hz", fc_a));
    dot(plot.right() - 72.0, AMBER, &format!("B {:>5.0} Hz", fc_b));
}

// ===========================================================================
// Widgets
// ===========================================================================

fn section(p: &egui::Painter, r: Rect, title: &str) {
    p.rect_filled(r, CornerRadius::same(8), PANEL);
    p.rect_stroke(r, CornerRadius::same(8), stroke(1.5, OUTLINE), StrokeKind::Inside);
    p.text(pos2(r.left() + 14.0, r.top() + 14.0), Align2::LEFT_CENTER, title, FontId::proportional(12.5), TEXT);
}

fn switch_block(p: &egui::Painter, r: Rect, title: &str) {
    p.rect_filled(r, CornerRadius::same(6), PANEL_DARK);
    p.rect_stroke(r, CornerRadius::same(6), stroke(1.0, OUTLINE_DIM), StrokeKind::Inside);
    p.text(pos2(r.center().x, r.bottom() - 12.0), Align2::CENTER_CENTER, title, FontId::proportional(9.5), TEXT_DIM);
}

fn activity_lamp(p: &egui::Painter, c: Pos2, lfo: f32) {
    let glow = lfo.clamp(0.0, 1.0);
    p.circle_filled(c, 7.0, LED_RED.gamma_multiply(0.15 * glow));
    p.circle_filled(c, 3.5, lerp_color(Color32::from_rgb(70, 22, 20), LED_RED, glow));
}

/// The light-tight pod: brightness follows the LED current.
fn pod_glow(p: &egui::Painter, c: Pos2, glow: f32, accent: Color32) {
    let g = glow.clamp(0.0, 1.0);
    p.circle_filled(c, 18.0, accent.gamma_multiply(0.10 * g));
    p.circle_filled(c, 12.0, accent.gamma_multiply(0.18 * g));
    p.circle_stroke(c, 12.0, stroke(1.0, OUTLINE_DIM));
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.0 - std::f32::consts::FRAC_PI_2;
        let cc = c + vec2(a.cos(), a.sin()) * 8.0;
        p.circle_filled(cc, 2.0, lerp_color(Color32::from_rgb(60, 50, 30), Color32::from_rgb(255, 214, 120), g));
    }
    p.circle_filled(c, 2.5, lerp_color(Color32::from_rgb(50, 20, 20), LED_RED, g));
    p.text(c + vec2(0.0, 22.0), Align2::CENTER_TOP, "PHOTO POD", FontId::proportional(8.5), TEXT_DIM);
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn param_default_normalized(st: &PluginContext<MutanteParams>, id: u32) -> Option<f64> {
    st.params()
        .param_infos()
        .iter()
        .find(|i| i.id == id)
        .map(|i| i.range.normalize(i.default_plain))
}

fn enum_count(st: &PluginContext<MutanteParams>, id: u32) -> usize {
    st.params()
        .param_infos()
        .iter()
        .find(|i| i.id == id)
        .and_then(|i| i.range.step_count())
        .map_or(2, |n| n.get() as usize + 1)
}

fn enum_index(st: &PluginContext<MutanteParams>, id: u32) -> usize {
    let n = enum_count(st, id);
    (st.get_param(id) * (n - 1) as f32).round() as usize
}

fn set_enum(st: &PluginContext<MutanteParams>, id: u32, index: usize) {
    let n = enum_count(st, id);
    let norm = if n > 1 { index.min(n - 1) as f64 / (n - 1) as f64 } else { 0.0 };
    st.begin_edit(id);
    st.set_param(id, norm);
    st.end_edit(id);
}

/// Flat rotary knob: dark body, 270° track with a coloured value arc and a
/// pointer. Drag vertically (Shift for fine), scroll, double-click to reset.
#[allow(clippy::too_many_arguments)]
fn knob(
    ui: &mut egui::Ui,
    st: &PluginContext<MutanteParams>,
    id: P,
    label: &str,
    c: Pos2,
    radius: f32,
    accent: Color32,
    enabled: bool,
) {
    let is_rate = matches!(id, P::Gen1Rate | P::Gen2Rate);
    let id: u32 = id.into();
    let hit = Rect::from_center_size(c, Vec2::splat(radius * 2.0 + 10.0));
    let resp = ui.interact(hit, egui::Id::new(("knob", id)), Sense::click_and_drag());
    let mut value = st.get_param(id);

    if resp.double_clicked() {
        if let Some(d) = param_default_normalized(st, id) {
            st.begin_edit(id);
            st.set_param(id, d);
            st.end_edit(id);
            value = d as f32;
        }
    } else {
        if resp.drag_started() {
            st.begin_edit(id);
        }
        if resp.dragged() {
            let fine = ui.input(|i| i.modifiers.shift);
            let delta = -resp.drag_delta().y / if fine { 900.0 } else { 220.0 };
            value = (value + delta).clamp(0.0, 1.0);
            st.set_param(id, f64::from(value));
        }
        if resp.drag_stopped() {
            st.end_edit(id);
        }
        if resp.hovered() && !resp.dragged() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                value = (value + scroll / 1500.0).clamp(0.0, 1.0);
                st.begin_edit(id);
                st.set_param(id, f64::from(value));
                st.end_edit(id);
            }
        }
    }

    let p = ui.painter();
    let start = std::f32::consts::FRAC_PI_4 * 3.0;
    let sweep = std::f32::consts::FRAC_PI_2 * 3.0;
    let accent = if enabled { accent } else { TEXT_DIM.gamma_multiply(0.6) };
    let active = resp.hovered() || resp.dragged();

    p.add(egui::Shape::line(arc(c, radius, start, sweep, 48), stroke(3.0, TRACK)));
    if value > 0.002 {
        let n = ((value * 48.0) as usize).max(2);
        p.add(egui::Shape::line(arc(c, radius, start, sweep * value, n), stroke(3.0, accent)));
    }
    p.circle_filled(c, radius - 6.0, KNOB_BODY);
    p.circle_stroke(
        c,
        radius - 6.0,
        stroke(1.0, if active { OUTLINE } else { OUTLINE_DIM }),
    );
    let a = start + sweep * value;
    let dir = vec2(a.cos(), a.sin());
    p.line_segment(
        [c + dir * (radius - 6.0) * 0.25, c + dir * (radius - 8.0)],
        stroke(2.0, if enabled { TEXT } else { TEXT_DIM }),
    );

    let text = if active {
        if is_rate {
            format!("{} Hz", st.format_param(id))
        } else {
            st.format_param(id)
        }
    } else {
        label.to_owned()
    };
    let col = if active { accent } else { TEXT_DIM };
    p.text(c + vec2(0.0, radius + 6.0), Align2::CENTER_TOP, text, FontId::proportional(10.5), col);
}

fn arc(c: Pos2, r: f32, start: f32, sweep: f32, n: usize) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let a = start + sweep * i as f32 / n as f32;
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

enum RockerLabels {
    Text(&'static str, &'static str),
    /// Sine/triangle icon on top, square icon below, as on the panel.
    Waves,
}

/// Vertical rocker switch for two-position parameters (index 0 = top).
fn rocker(
    ui: &mut egui::Ui,
    st: &PluginContext<MutanteParams>,
    id: P,
    labels: RockerLabels,
    caption: &str,
    c: Pos2,
) {
    let id: u32 = id.into();
    let body = Rect::from_center_size(c, vec2(24.0, 50.0));
    let resp = ui.interact(body.expand(6.0), egui::Id::new(("rocker", id)), Sense::click());
    if resp.clicked() {
        let idx = enum_index(st, id);
        set_enum(st, id, 1 - idx.min(1));
    }
    let idx = enum_index(st, id).min(1);
    let p = ui.painter();
    p.rect_filled(body, CornerRadius::same(4), KNOB_BODY);
    p.rect_stroke(
        body,
        CornerRadius::same(4),
        stroke(1.0, if resp.hovered() { OUTLINE } else { OUTLINE_DIM }),
        StrokeKind::Inside,
    );
    let half = Rect::from_min_size(
        pos2(body.left() + 3.0, body.top() + 3.0 + idx as f32 * 22.0),
        vec2(18.0, 22.0),
    );
    p.rect_filled(half, CornerRadius::same(3), SILVER);
    p.line_segment(
        [pos2(half.left() + 4.0, half.center().y), pos2(half.right() - 4.0, half.center().y)],
        stroke(1.0, Color32::from_gray(150)),
    );

    let top = pos2(c.x, body.top() - 10.0);
    let bottom = pos2(c.x, body.bottom() + 10.0);
    let col = |i: usize| if idx == i { TEXT } else { TEXT_DIM };
    match labels {
        RockerLabels::Text(a, b) => {
            p.text(top, Align2::CENTER_CENTER, a, FontId::proportional(10.0), col(0));
            p.text(bottom, Align2::CENTER_CENTER, b, FontId::proportional(10.0), col(1));
        }
        RockerLabels::Waves => {
            wave_icon(p, top, false, col(0));
            wave_icon(p, bottom, true, col(1));
        }
    }
    if !caption.is_empty() {
        p.text(c + vec2(0.0, 48.0), Align2::CENTER_CENTER, caption, FontId::proportional(9.5), TEXT_DIM);
    }
}

fn wave_icon(p: &egui::Painter, c: Pos2, square: bool, col: Color32) {
    let (w, h) = (16.0, 5.0);
    let pts: Vec<Pos2> = if square {
        vec![
            pos2(c.x - w / 2.0, c.y + h),
            pos2(c.x - w / 2.0, c.y - h),
            pos2(c.x, c.y - h),
            pos2(c.x, c.y + h),
            pos2(c.x + w / 2.0, c.y + h),
            pos2(c.x + w / 2.0, c.y - h),
        ]
    } else {
        (0..=24)
            .map(|i| {
                let t = i as f32 / 24.0;
                pos2(c.x - w / 2.0 + w * t, c.y - h * (t * std::f32::consts::TAU).sin())
            })
            .collect()
    };
    p.add(egui::Shape::line(pts, stroke(1.5, col)));
}

/// Three-position rotary selector with labels around the top arc, like the
/// panel's "Phasor B Input" and Phasor B "Sweep" switches. Click a label, or
/// click / drag the knob.
fn selector(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, id: P, labels: &[&str], c: Pos2) {
    let id: u32 = id.into();
    let n = labels.len();
    let angles: Vec<f32> = (0..n)
        .map(|i| -std::f32::consts::FRAC_PI_2 + (i as f32 - (n - 1) as f32 / 2.0) * 0.95)
        .collect();
    let idx = enum_index(st, id).min(n - 1);

    let knob_rect = Rect::from_center_size(c, Vec2::splat(40.0));
    let resp = ui.interact(knob_rect, egui::Id::new(("selector", id)), Sense::click_and_drag());
    if resp.clicked() {
        set_enum(st, id, (idx + 1) % n);
    }
    if resp.dragged()
        && let Some(pp) = resp.interact_pointer_pos() {
            let a = (pp.y - c.y).atan2(pp.x - c.x);
            let best = angles
                .iter()
                .enumerate()
                .min_by(|x, y| (x.1 - a).abs().total_cmp(&(y.1 - a).abs()))
                .map_or(idx, |b| b.0);
            if best != idx {
                set_enum(st, id, best);
            }
        }

    for (i, (&a, &label)) in angles.iter().zip(labels).enumerate() {
        let lp = c + vec2(a.cos() * 42.0, a.sin() * 33.0);
        let lr = Rect::from_center_size(lp, vec2(40.0, 14.0));
        let lresp = ui.interact(lr, egui::Id::new(("selector-label", id, i)), Sense::click());
        if lresp.clicked() {
            set_enum(st, id, i);
        }
        let col = if i == idx {
            TEXT
        } else if lresp.hovered() {
            OUTLINE
        } else {
            TEXT_DIM
        };
        ui.painter().text(lp, Align2::CENTER_CENTER, label, FontId::proportional(9.5), col);
        let tick_in = c + vec2(a.cos(), a.sin()) * 22.0;
        let tick_out = c + vec2(a.cos(), a.sin()) * 26.0;
        ui.painter().line_segment([tick_in, tick_out], stroke(1.0, if i == idx { TEXT } else { OUTLINE_DIM }));
    }

    let p = ui.painter();
    p.circle_filled(c, 18.0, KNOB_BODY);
    p.circle_stroke(c, 18.0, stroke(1.0, if resp.hovered() { OUTLINE } else { OUTLINE_DIM }));
    let a = angles[idx];
    let dir = vec2(a.cos(), a.sin());
    p.line_segment([c + dir * 4.0, c + dir * 15.0], stroke(2.5, TEXT));
}

/// Ableton-style toggle button.
fn toggle_button(
    ui: &mut egui::Ui,
    st: &PluginContext<MutanteParams>,
    id: P,
    label: &str,
    r: Rect,
    accent: Color32,
) {
    let id: u32 = id.into();
    let resp = ui.interact(r, egui::Id::new(("toggle", id)), Sense::click());
    let on = st.get_param(id) > 0.5;
    if resp.clicked() {
        st.begin_edit(id);
        st.set_param(id, if on { 0.0 } else { 1.0 });
        st.end_edit(id);
    }
    let on = st.get_param(id) > 0.5;
    let p = ui.painter();
    let fill = if on { accent } else { Color32::from_rgb(46, 49, 54) };
    p.rect_filled(r, CornerRadius::same(3), fill);
    if resp.hovered() {
        p.rect_stroke(r, CornerRadius::same(3), stroke(1.0, OUTLINE), StrokeKind::Inside);
    }
    let col = if on { Color32::from_rgb(20, 20, 22) } else { TEXT_DIM };
    p.text(r.center(), Align2::CENTER_CENTER, label, FontId::proportional(10.0), col);
}

/// Segmented control for small enums.
fn segmented(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, id: P, labels: &[&str], r: Rect) {
    let id: u32 = id.into();
    let n = labels.len();
    let idx = enum_index(st, id);
    let w = r.width() / n as f32;
    for (i, label) in labels.iter().enumerate() {
        let cell = Rect::from_min_size(pos2(r.left() + i as f32 * w, r.top()), vec2(w, r.height()));
        let resp = ui.interact(cell, egui::Id::new(("seg", id, i)), Sense::click());
        if resp.clicked() {
            set_enum(st, id, i);
        }
        let on = i == idx;
        let p = ui.painter();
        p.rect_filled(
            cell.shrink(0.5),
            CornerRadius::same(3),
            if on { AMBER } else { Color32::from_rgb(46, 49, 54) },
        );
        p.text(
            cell.center(),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(9.5),
            if on { Color32::from_rgb(20, 20, 22) } else { TEXT_DIM },
        );
    }
}

/// Tempo-sync toggle with a stepper for the musical division (modern
/// addition, kept visually apart from the original controls).
fn sync_block(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, sync: P, div: P, origin: Pos2) {
    let block = Rect::from_min_size(origin, vec2(112.0, 88.0));
    let p = ui.painter();
    p.rect_stroke(block, CornerRadius::same(5), stroke(1.0, OUTLINE_DIM), StrokeKind::Inside);
    p.text(pos2(block.left() + 8.0, block.top() + 10.0), Align2::LEFT_CENTER, "HOST SYNC", FontId::proportional(9.0), TEXT_DIM);
    toggle_button(
        ui,
        st,
        sync,
        "SYNC",
        Rect::from_min_size(pos2(block.left() + 8.0, block.top() + 24.0), vec2(96.0, 22.0)),
        AMBER,
    );
    stepper(ui, st, div, Rect::from_min_size(pos2(block.left() + 8.0, block.top() + 54.0), vec2(96.0, 22.0)));
}

fn stepper(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, id: P, r: Rect) {
    let id: u32 = id.into();
    let n = enum_count(st, id);
    let idx = enum_index(st, id);
    let resp = ui.interact(r, egui::Id::new(("stepper", id)), Sense::click());
    if resp.clicked()
        && let Some(pp) = resp.interact_pointer_pos() {
            let next = if pp.x < r.center().x { idx.saturating_sub(1) } else { (idx + 1).min(n - 1) };
            set_enum(st, id, next);
        }
    let p = ui.painter();
    p.rect_filled(r, CornerRadius::same(3), KNOB_BODY);
    p.rect_stroke(
        r,
        CornerRadius::same(3),
        stroke(1.0, if resp.hovered() { OUTLINE } else { OUTLINE_DIM }),
        StrokeKind::Inside,
    );
    p.text(pos2(r.left() + 9.0, r.center().y), Align2::CENTER_CENTER, "‹", FontId::proportional(13.0), TEXT_DIM);
    p.text(pos2(r.right() - 9.0, r.center().y), Align2::CENTER_CENTER, "›", FontId::proportional(13.0), TEXT_DIM);
    p.text(r.center(), Align2::CENTER_CENTER, st.format_param(id), FontId::proportional(10.5), TEXT);
}

/// Footswitch with the phasor's red LED. The LED shows the actual state of
/// the PM1/PM2 photocell, so it fades with the optical bypass.
fn footswitch(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, id: P, lamp: P, c: Pos2) {
    let id: u32 = id.into();
    let resp = ui.interact(Rect::from_center_size(c, Vec2::splat(52.0)), egui::Id::new(("foot", id)), Sense::click());
    if resp.clicked() {
        let on = st.get_param(id) > 0.5;
        st.begin_edit(id);
        st.set_param(id, if on { 0.0 } else { 1.0 });
        st.end_edit(id);
    }
    let on = st.get_param(id) > 0.5;
    let lit = st.get_meter(lamp);
    let p = ui.painter();
    let led = c + vec2(0.0, -52.0);
    let glow = if on { 1.0 } else { 0.0f32.max(lit * 0.5) };
    p.circle_filled(led, 10.0, LED_RED.gamma_multiply(0.18 * glow));
    p.circle_filled(led, 4.5, lerp_color(Color32::from_rgb(70, 20, 18), LED_RED, glow));
    p.circle_filled(c, 24.0, Color32::from_rgb(54, 57, 63));
    p.circle_stroke(c, 24.0, stroke(1.0, if resp.hovered() { SILVER } else { OUTLINE }));
    p.circle_filled(c, 15.0, Color32::from_rgb(150, 155, 162));
    p.circle_filled(c, 11.0, Color32::from_rgb(186, 190, 196));
    p.text(
        c + vec2(0.0, 36.0),
        Align2::CENTER_CENTER,
        if on { "ON" } else { "BYPASSED" },
        FontId::proportional(9.5),
        if on { TEXT } else { TEXT_DIM },
    );
}

/// Vertical pedal: heel at the bottom, toe at the top. In envelope mode the
/// filled bar shows the pedal position the envelope is producing.
fn pedal_fader(ui: &mut egui::Ui, st: &PluginContext<MutanteParams>, r: Rect) {
    let id: u32 = P::PedalPosition.into();
    let resp = ui.interact(r, egui::Id::new(("pedal", id)), Sense::click_and_drag());
    if resp.drag_started() {
        st.begin_edit(id);
    }
    if (resp.dragged() || resp.clicked())
        && let Some(pp) = resp.interact_pointer_pos() {
            let v = ((r.bottom() - 10.0 - pp.y) / (r.height() - 20.0)).clamp(0.0, 1.0);
            if resp.clicked() {
                st.begin_edit(id);
            }
            st.set_param(id, f64::from(v));
            if resp.clicked() {
                st.end_edit(id);
            }
        }
    if resp.drag_stopped() {
        st.end_edit(id);
    }
    if resp.double_clicked()
        && let Some(d) = param_default_normalized(st, id) {
            st.begin_edit(id);
            st.set_param(id, d);
            st.end_edit(id);
        }

    let p = ui.painter();
    p.rect_filled(r, CornerRadius::same(6), KNOB_BODY);
    p.rect_stroke(r, CornerRadius::same(6), stroke(1.0, if resp.hovered() { OUTLINE } else { OUTLINE_DIM }), StrokeKind::Inside);
    let live = st.get_meter(P::PedalMeter).clamp(0.0, 1.0);
    let inner = r.shrink(6.0);
    let fill = Rect::from_min_max(pos2(inner.left(), inner.bottom() - inner.height() * live), inner.max);
    p.rect_filled(fill, CornerRadius::same(3), BLUE.gamma_multiply(0.35));
    for i in 0..=10 {
        let y = inner.bottom() - inner.height() * i as f32 / 10.0;
        p.line_segment([pos2(inner.right() - 6.0, y), pos2(inner.right(), y)], stroke(1.0, OUTLINE_DIM));
    }
    let set = st.get_param(id);
    let y = r.bottom() - 10.0 - (r.height() - 20.0) * set;
    let handle = Rect::from_center_size(pos2(r.center().x, y), vec2(r.width() + 6.0, 12.0));
    p.rect_filled(handle, CornerRadius::same(3), SILVER);
    p.line_segment(
        [pos2(handle.left() + 8.0, y), pos2(handle.right() - 8.0, y)],
        stroke(1.0, Color32::from_gray(120)),
    );
    p.text(pos2(r.center().x, r.top() - 6.0), Align2::CENTER_BOTTOM, "TOE", FontId::proportional(8.5), TEXT_DIM);
    p.text(pos2(r.center().x, r.bottom() + 4.0), Align2::CENTER_TOP, "HEEL", FontId::proportional(8.5), TEXT_DIM);
}

fn stereo_meter(p: &egui::Painter, st: &PluginContext<MutanteParams>, r: Rect) {
    let rows = [(P::OutMeterL, "L"), (P::OutMeterR, "R")];
    let h = (r.height() - 4.0) / 2.0;
    for (i, (id, label)) in rows.iter().enumerate() {
        let y = r.top() + i as f32 * (h + 4.0);
        let bar = Rect::from_min_size(pos2(r.left() + 12.0, y), vec2(r.width() - 12.0, h));
        p.text(pos2(r.left(), bar.center().y), Align2::LEFT_CENTER, *label, FontId::proportional(9.0), TEXT_DIM);
        p.rect_filled(bar, CornerRadius::same(2), KNOB_BODY);
        let level = truce::core::meter_display(st.get_meter(*id));
        if level > 0.001 {
            let fill = Rect::from_min_size(bar.min, vec2(bar.width() * level, h));
            let col = if level > 0.97 { LED_RED } else { BLUE };
            p.rect_filled(fill, CornerRadius::same(2), col);
        }
    }
}

#[inline]
fn stroke(width: f32, color: Color32) -> Stroke {
    Stroke::new(width, color)
}
