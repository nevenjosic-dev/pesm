// Immediate-mode UI kit with the original's dark theme tokens (pesm/ui/theme/Theme.qml).
use macroquad::prelude::*;

pub fn hex(h: u32) -> Color {
    Color::from_rgba((h >> 16) as u8, (h >> 8) as u8, h as u8, 255)
}
pub fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

// Design tokens from the "Gestaltungsleitfaden" (§4): two themes ("Sonntag" light, default; "Flutlicht" dark)
// and club colours that tint the game. Set once per frame with Theme::set.
thread_local! {
    static DARK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CLUB: std::cell::Cell<u32> = const { std::cell::Cell::new(0xd4152f) };
    static AREA: std::cell::Cell<u32> = const { std::cell::Cell::new(0x38bdf8) };
    /// Accent of the theme: from the own crest (Ui::wappen_farbe) or the default blue.
    static AKZENT: std::cell::Cell<u32> = const { std::cell::Cell::new(0x38bdf8) };
}
fn pick(dark: u32, light: u32) -> Color { hex(if DARK.with(|d| d.get()) { dark } else { light }) }
fn lum(c: Color) -> f32 {
    let f = |x: f32| if x <= 0.03928 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) };
    0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
}
/// The colour with the higher contrast on `bg`: white or near black (guide: verein_auf, at least 4.5:1).
pub fn on(bg: Color) -> Color { if (1.05 / (lum(bg) + 0.05)) >= ((lum(bg) + 0.05) / 0.0625) { hex(0xffffff) } else { hex(0x1b1f27) } }
/// Contrast ratio of two colours (WCAG).
pub fn kontrast(a: Color, b: Color) -> f32 { let (x, y) = (lum(a), lum(b)); (x.max(y) + 0.05) / (x.min(y) + 0.05) }
/// The colour with its lightness moved (darker on light backgrounds, lighter on dark ones) until it reaches `min`
/// contrast against `bg`; the hue stays.
pub fn lesbar(c: Color, bg: Color, min: f32) -> Color {
    if kontrast(c, bg) >= min { return c; }
    let hell = lum(bg) > 0.4;
    let l0 = { let (mx, mn) = (c.r.max(c.g).max(c.b), c.r.min(c.g).min(c.b)); (mx + mn) / 2.0 };
    for k in 1..=20 {
        let l = if hell { l0 * (1.0 - k as f32 / 20.0) } else { l0 + (1.0 - l0) * k as f32 / 20.0 };
        let x = with_light(c, l.clamp(0.0, 1.0), 1.0);
        if kontrast(x, bg) >= min { return x; }
    }
    if hell { hex(0x1b1f27) } else { hex(0xffffff) }
}
/// The main colour of a crest picture (RGBA): the most weighty hue among clearly coloured, opaque pixels (white,
/// black, grey and transparent left out), as the mean of that hue's pixels. None when the crest has no real colour
/// (core/vereinsfarben_ableitung.py: vorherrschende kräftige Farbe).
pub fn hauptfarbe(rgba: &[u8]) -> Option<u32> {
    let mut eimer = [(0f32, 0f32, 0f32, 0f32); 24];
    let mut gesamt = 0f32;
    for px in rgba.chunks_exact(4) {
        if px[3] < 128 { continue; }
        let (r, g, b) = (px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
        let l = (mx + mn) / 2.0;
        let d = mx - mn;
        if d < 0.15 || l < 0.1 || l > 0.93 { continue; }
        let s = d / (1.0 - (2.0 * l - 1.0).abs()).max(1e-3);
        if s < 0.3 { continue; }
        let h = if mx == r { ((g - b) / d).rem_euclid(6.0) } else if mx == g { (b - r) / d + 2.0 } else { (r - g) / d + 4.0 } / 6.0;
        let i = ((h * 24.0) as usize).min(23);
        let w = s * d;
        eimer[i].0 += r * w; eimer[i].1 += g * w; eimer[i].2 += b * w; eimer[i].3 += w;
        gesamt += w;
    }
    // neighbouring hue buckets count together, so a gradient does not split one colour
    let (i, _) = (0..24).map(|i| (i, eimer[i].3 + 0.5 * (eimer[(i + 23) % 24].3 + eimer[(i + 1) % 24].3))).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?;
    let e = eimer[i];
    if e.3 <= 0.0 || e.3 < gesamt * 0.15 || gesamt < (rgba.len() / 4) as f32 * 0.01 { return None; }
    let k = |x: f32| ((x / e.3).clamp(0.0, 1.0) * 255.0).round() as u32;
    Some((k(e.0) << 16) | (k(e.1) << 8) | k(e.2))
}
/// Same hue with another lightness. shortcut: HSL instead of OKLCH, close enough for flat colours.
pub fn with_light(c: Color, l: f32, sat: f32) -> Color {
    let (r, g, b) = (c.r, c.g, c.b);
    let mx = r.max(g).max(b); let mn = r.min(g).min(b);
    let d = mx - mn;
    let h = if d == 0.0 { 0.0 } else if mx == r { ((g - b) / d).rem_euclid(6.0) } else if mx == g { (b - r) / d + 2.0 } else { (r - g) / d + 4.0 } / 6.0;
    let s0 = if d == 0.0 { 0.0 } else { d / (1.0 - (mx + mn - 1.0).abs()) };
    let s = s0 * sat;
    let q = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = q * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match (h * 6.0) as i32 { 0 => (q, x, 0.0), 1 => (x, q, 0.0), 2 => (0.0, q, x), 3 => (0.0, x, q), 4 => (x, 0.0, q), _ => (q, 0.0, x) };
    let m = l - q / 2.0;
    Color::new(r1 + m, g1 + m, b1 + m, 1.0)
}

/// Who a drawn figure shows (Ui::figur).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Rolle { Vorstand, Berater, Presse, Scout, Stab, Spieler, Person }
impl Rolle {
    /// The role from a sender or name: board, agent, press, scout or coaching staff by key words (German and English).
    pub fn aus_name(name: &str) -> Rolle {
        let n = name.to_lowercase();
        let hat = |w: &[&str]| w.iter().any(|x| n.contains(x));
        if hat(&[crate::id("präsid"), crate::id("vorstand"), crate::id("aufsichtsrat"), crate::id("board"), crate::id("chairman"), crate::id("president"), crate::id("geschäftsführ"), crate::id("sportdirektor"), crate::id("direktor")]) { Rolle::Vorstand }
        else if hat(&[crate::id("berater"), crate::id("agent"), crate::id("agentur"), crate::id("management")]) { Rolle::Berater }
        else if hat(&[crate::id("redaktion"), crate::id("zeitung"), crate::id("presse"), crate::id("journal"), crate::id("reporter"), crate::id("kicker"), crate::id("magazin"), crate::id("press"), crate::id("news"), crate::id("times"), crate::id("gazette"), crate::id("sportbild")]) { Rolle::Presse }
        else if hat(&[crate::id("scout")]) { Rolle::Scout }
        else if hat(&[crate::id("trainer"), crate::id("co-"), crate::id("coach"), crate::id("physio"), crate::id("arzt"), crate::id("doktor"), crate::id("staff"), crate::id("betreuer")]) { Rolle::Stab }
        else { Rolle::Person }
    }
}

pub struct Theme;
impl Theme {
    pub fn set(dark: bool, club: u32, area: u32) { DARK.with(|d| d.set(dark)); CLUB.with(|c| c.set(club)); AREA.with(|a| a.set(area)); }
    pub fn dark() -> bool { DARK.with(|d| d.get()) }
    pub fn grund() -> Color { pick(0x0f1218, 0xf6f3ee) }
    pub fn grund_tief() -> Color { pick(0x0a0d12, 0xede8e0) }
    pub fn flaeche() -> Color { pick(0x171c26, 0xffffff) }
    pub fn erhoehung() -> Color { pick(0x1e2430, 0xfbf9f6) }
    pub fn ueberlagerung() -> Color { pick(0x2a3242, 0xefe9df) }
    pub fn linie() -> Color { pick(0x2a3242, 0xe4ded4) }
    pub fn text() -> Color { pick(0xeef2f8, 0x1b1f27) }
    pub fn text2() -> Color { pick(0x9aa6b8, 0x5d6676) }
    pub fn gedaempft() -> Color { pick(0x7f8a9c, 0x6b7280) }
    /// The accent, made readable on the current background: at least 3:1 against it (buttons, bars, markings).
    pub fn akzent() -> Color { lesbar(hex(AKZENT.with(|a| a.get())), Theme::grund(), 3.0) }
    /// Text on the accent: white or near black, whichever reads better (at least 4.5:1).
    pub fn auf_akzent() -> Color { on(Theme::akzent()) }
    pub fn set_akzent(c: u32) { AKZENT.with(|a| a.set(c)); }
    pub fn gut() -> Color { hex(0x3ecf8e) }
    pub fn achtung() -> Color { hex(0xf5b942) }
    pub fn problem() -> Color { hex(0xff4d6a) }
    pub fn gold() -> Color { hex(0xf5b942) }
    pub fn kontur() -> Color { Theme::linie() }
    pub fn schatten() -> Color { Color::new(0.0, 0.0, 0.0, if Theme::dark() { 0.35 } else { 0.07 }) }
    pub fn verein() -> Color { hex(CLUB.with(|c| c.get())) }
    pub fn verein_auf() -> Color { on(Theme::verein()) }
    pub fn verein_tief() -> Color { with_light(Theme::verein(), 0.2, 0.9) }
    pub fn verein_weich() -> Color { with_light(Theme::verein(), 0.94, 0.5) }
    /// Area colour of the current page (guide §4 Bereichsfarben), for card heads.
    pub fn bereich() -> Color { hex(AREA.with(|a| a.get())) }
    /// Wertzelle on the PES scale 40-99 (DESIGN.md §5): schwach <55, mittel 55-64, gut 65-74, sehr gut 75-84, Elite 85+.
    pub fn skala(v: f32) -> Color {
        if v >= 85.0 { hex(0x2EE6C5) } else if v >= 75.0 { hex(0x4CC76A) } else if v >= 65.0 { hex(0xC9D94A) } else if v >= 55.0 { hex(0xF0A040) } else { hex(0xA6A8B3) }
    }
    /// Pitch card colour (Theme.stufeFarbe): 80+ gut, 70-79 gelb, <70 orange.
    pub fn stufe(v: f32) -> Color {
        if v >= 80.0 { Theme::gut() } else if v >= 70.0 { hex(0xF2C230) } else { hex(0xF28C38) }
    }
    pub fn teil(pos: &str) -> Color {
        match crate::world::part_of(pos) {
            0 => hex(0xa78bfa),
            1 => hex(0x3b82f6),
            2 => hex(0x14b8a6),
            _ => hex(0xf43f5e),
        }
    }
    /// Match grades (guide §7 Spielbericht): green from 7.0, yellow 6.0-6.9, red below.
    pub fn note(r: f32) -> Color {
        if r >= 7.0 { Theme::gut() } else if r >= 6.0 { Theme::achtung() } else { Theme::problem() }
    }
}

pub struct Ui {
    pub head: Font,
    pub body: Font,
    pub semi: Font,
    pub mouse: Vec2,
    pub clicked: bool,
    pub released_consumed: bool,
    pub wheel: f32,
    pub time: f32,
    pub typed: Vec<char>,
    pub key_back: bool,
    pub key_enter: bool,
    pub sfx: Vec<(String, Option<macroquad::audio::Sound>)>,
    pub sound_queue: Vec<&'static str>,
    pub hover_any: bool,
    /// Full text of a cut text under the mouse (text_passend, buttons); drawn last in the frame.
    pub tooltip: std::cell::RefCell<Option<String>>,
    /// The frames of the cards and tiles drawn in this frame: every text is checked against the innermost one it starts in.
    pub rahmen: std::cell::RefCell<Vec<Rect>>,
    /// Where player photos were drawn this frame (player id, centre, radius): a right click takes the nearest (Punkt 6).
    pub foto_orte: std::cell::RefCell<Vec<(u32, Vec2, f32)>>,
    /// Crests drawn this frame (club id, centre, radius): a right click on a club row finds its club.
    pub wappen_orte: std::cell::RefCell<Vec<(u32, Vec2, f32)>>,
    /// scripted click for --self-test
    pub inject_click: Option<Vec2>,
    /// Volume of the sounds 0…1 (Einstellungen → Klang: aus, leise, normal; default leise).
    pub lautstaerke: f32,
    pub crests: std::cell::RefCell<std::collections::HashMap<u32, Option<Texture2D>>>,
    /// Main colour of a club crest (accent from the own crest), worked out once per club.
    pub wappen_farben: std::cell::RefCell<std::collections::HashMap<u32, Option<u32>>>,
    /// Crest and logo files of the current world (fl27::bilder_index) and the world folder it was built for.
    pub bilder: crate::fl27::Bilder,
    pub bilder_fuer: Option<std::path::PathBuf>,
    pub logos: std::cell::RefCell<std::collections::HashMap<u16, Option<Texture2D>>>,
    pub fotos: std::cell::RefCell<std::collections::HashMap<u32, Option<Texture2D>>>,
    pub stadien: std::cell::RefCell<std::collections::HashMap<u32, Option<Texture2D>>>,
    /// Photos loaded per frame at most (long lists load their visible rows over a few frames, without stutter).
    pub foto_budget: std::cell::Cell<u8>,
    /// Pictures from free web sources (netzbilder.rs) with their credit line, and where credits go this frame.
    pub netz: std::cell::RefCell<std::collections::HashMap<(crate::netzbilder::Art, String), Option<(Texture2D, String)>>>,
    pub nachweise: std::cell::RefCell<Vec<(Rect, String)>>,
}

/// Window-size check (tools/fenstertest.py, PESM_PRUEF=<report.json>): texts drawn beyond the window, texts cut to „…“,
/// and how far right and down anything was drawn (a big empty band shows as a gap).
#[derive(Default)]
pub struct Pruefung { pub ueberlauf: std::collections::BTreeSet<String>, pub gekuerzt: std::collections::BTreeSet<String>, pub kartenrand: std::collections::BTreeSet<String>, pub rechts: f32, pub unten: f32 }
pub static PRUEFUNG: std::sync::Mutex<Option<Pruefung>> = std::sync::Mutex::new(None);
static PRUEFUNG_AN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn pruefung_an() -> bool { PRUEFUNG_AN.load(std::sync::atomic::Ordering::Relaxed) }
pub fn pruefung_starten() { *PRUEFUNG.lock().unwrap() = Some(Pruefung::default()); PRUEFUNG_AN.store(true, std::sync::atomic::Ordering::Relaxed); }
fn pruefung_text(s: &str, x: f32, y: f32, w: f32) {
    if s.trim().is_empty() { return; }
    if let Some(p) = PRUEFUNG.lock().unwrap().as_mut() {
        if x + w > breite() + 2.0 || x < -2.0 || y > hoehe() + 2.0 { p.ueberlauf.insert(format!("{s} @ {x:.0},{y:.0} (bis {:.0})", x + w)); }
        p.rechts = p.rechts.max(x + w);
        p.unten = p.unten.max(y);
    }
}
/// The report as JSON: overflow, cut texts, the right and bottom edge reached, the layout size.
pub fn pruefung_bericht() -> String {
    let g = PRUEFUNG.lock().unwrap();
    let Some(p) = g.as_ref() else { return "{}".into() };
    serde_json::json!({ "ueberlauf": p.ueberlauf, "gekuerzt": p.gekuerzt, "kartenrand": p.kartenrand, "rechts": p.rechts, "unten": p.unten, "breite": breite(), "hoehe": hoehe() }).to_string()
}
/// Reference width of the layout; very wide windows (21:9) get more width instead of side bars (breite()).
pub const W_BASIS: f32 = 1400.0;
static BREITE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x44AF_0000); // 1400.0
/// Width of the layout in its units: 1400, or more when the window is wider than the height allows (2560×1080 → about 1800).
pub fn breite() -> f32 { f32::from_bits(BREITE.load(std::sync::atomic::Ordering::Relaxed)) }
static HOEHE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x4461_0000); // 900.0
/// Height of the layout: the width (1400) is the reference, the height follows the window, at least 760.
/// Seconds since start (for cards that stay a while).
pub fn jetzt() -> f64 { static T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new(); T0.get_or_init(std::time::Instant::now).elapsed().as_secs_f64() }
pub fn hoehe() -> f32 { f32::from_bits(HOEHE.load(std::sync::atomic::Ordering::Relaxed)) }
/// The UI scale chosen in the settings (0.75 … 1.5, Fabian 23.11.: on 3440×1440 everything looked like a touch screen).
static SKALA: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3F80_0000); // 1.0
pub fn ui_skala() -> f32 { f32::from_bits(SKALA.load(std::sync::atomic::Ordering::Relaxed)) }
pub fn ui_skala_setzen(s: f32) { SKALA.store(s.clamp(0.75, 1.5).to_bits(), std::sync::atomic::Ordering::Relaxed); }
/// Widest the layout gets; a wider window keeps the content in the middle (no stretching over 21:9).
pub const W_MAX: f32 = 1920.0;
/// Once per frame, from the window size in pixels: a pixel size of one unit from the window height and the chosen
/// scale (1000 units high at 100 %), never narrower than 1400 or lower than 760 units, at most W_MAX wide and centred.
pub fn hoehe_setzen(sw: f32, sh: f32) {
    if sw > 0.0 && sh > 0.0 {
        let mut ppu = sh / 1000.0 * ui_skala();
        ppu = ppu.min(sw / W_BASIS).min(sh / 760.0);
        let h = sh / ppu;
        HOEHE.store(h.to_bits(), std::sync::atomic::Ordering::Relaxed);
        BREITE.store((sw / ppu).clamp(W_BASIS, W_MAX).to_bits(), std::sync::atomic::Ordering::Relaxed);
    }
}

impl Ui {
    pub fn begin(&mut self) {
        self.foto_budget.set(6);
        self.rahmen.borrow_mut().clear();
        self.foto_orte.borrow_mut().clear();
        self.wappen_orte.borrow_mut().clear();
        self.nachweise.borrow_mut().clear();
        self.mouse = Vec2::from(mouse_position()) * view_scale() - vec2(versatz(), 0.0);
        self.clicked = is_mouse_button_pressed(MouseButton::Left);
        if let Some(p) = self.inject_click.take() {
            self.mouse = p;
            self.clicked = true;
        }
        self.wheel = mouse_wheel().1;
        self.time = get_time() as f32;
        self.typed.clear();
        while let Some(c) = get_char_pressed() {
            if !c.is_control() { self.typed.push(c); }
        }
        self.key_back = is_key_pressed(KeyCode::Backspace);
        self.key_enter = is_key_pressed(KeyCode::Enter);
        self.hover_any = false;
        *self.tooltip.borrow_mut() = None;
    }

    pub fn play(&mut self, name: &'static str) {
        self.sound_queue.push(name);
    }

    pub fn flush_sounds(&mut self) {
        for name in self.sound_queue.drain(..) {
            if let Some((_, Some(s))) = self.sfx.iter().find(|(n, _)| n == name) {
                if self.lautstaerke > 0.0 { macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: false, volume: self.lautstaerke }); }
            }
        }
    }

    pub fn hover(&mut self, r: Rect) -> bool {
        let h = r.contains(self.mouse);
        if h { self.hover_any = true; }
        h
    }
    pub fn hit(&mut self, r: Rect) -> bool {
        if self.clicked && r.contains(self.mouse) {
            self.clicked = false;
            true
        } else {
            false
        }
    }

    // drawn as given: fixed texts come translated from crate::inhalt::t / tf!, names never go through a translation
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        let (s, g) = self.im_rahmen(s, x, y, size, &|t, g| self.measure(t, g));
        draw_text_ex(&s, x, y, TextParams { font: Some(&self.body), font_size: g as u16, color: c, ..Default::default() });
    }
    pub fn text_semi(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        let (s, g) = self.im_rahmen(s, x, y, size, &|t, g| self.measure_semi(t, g));
        draw_text_ex(&s, x, y, TextParams { font: Some(&self.semi), font_size: g as u16, color: c, ..Default::default() });
    }
    pub fn title(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        let (s, g) = self.im_rahmen(s, x, y, size, &|t, g| self.measure_title(t, g));
        draw_text_ex(&s, x, y, TextParams { font: Some(&self.head), font_size: g as u16, color: c, ..Default::default() });
    }
    pub fn measure_title(&self, s: &str, size: f32) -> f32 {
        measure_text(s, Some(&self.head), size as u16, 1.0).width
    }
    pub fn measure(&self, s: &str, size: f32) -> f32 {
        measure_text(s, Some(&self.body), size as u16, 1.0).width
    }
    pub fn measure_semi(&self, s: &str, size: f32) -> f32 {
        measure_text(s, Some(&self.semi), size as u16, 1.0).width
    }
    pub fn text_right(&self, s: &str, right: f32, y: f32, size: f32, c: Color) {
        let w = self.measure(s, size);
        self.text(s, right - w, y, size, c);
    }
    pub fn text_center(&self, s: &str, cx: f32, y: f32, size: f32, c: Color) {
        let w = self.measure_semi(s, size);
        self.text_semi(s, cx - w / 2.0, y, size, c);
    }
    /// Text in a width (Nachbesserung 22.11.): first smaller (down to 80 %), then over up to `zeilen` lines, only then cut
    /// with „…“; hovering a cut text shows it whole. Returns the number of lines drawn.
    pub fn text_passend(&self, s: &str, x: f32, y: f32, size: f32, maxw: f32, zeilen: usize, c: Color) -> usize {
        let (t, g, ganz) = self.einzeilig(s, size, maxw);
        if ganz || zeilen <= 1 {
            self.text(&t, x, y, g, c);
            if !ganz && Rect::new(x, y - g, maxw, g + 4.0).contains(self.mouse) { *self.tooltip.borrow_mut() = Some(s.to_string()); }
            return 1;
        }
        let lines = self.wrap(s, g, maxw);
        let n = lines.len().min(zeilen);
        let mut gekuerzt = lines.len() > n;
        for (k, l) in lines.iter().take(n).enumerate() {
            let zu_lang = self.measure(l, g) > maxw || (k == n - 1 && lines.len() > n);
            gekuerzt |= zu_lang;
            let l = if zu_lang { self.fit(&format!("{l} …"), g, maxw) } else { l.clone() };
            self.text(&l, x, y + k as f32 * g * 1.2, g, c);
        }
        if gekuerzt && Rect::new(x, y - g, maxw, n as f32 * g * 1.2 + 4.0).contains(self.mouse) { *self.tooltip.borrow_mut() = Some(s.to_string()); }
        n
    }
    /// One line in a width: the text, the size it fits in (down to 80 %), and whether it is whole (else cut with „…“).
    pub fn einzeilig(&self, s: &str, size: f32, maxw: f32) -> (String, f32, bool) {
        let mut g = size;
        while g > size * 0.8 && self.measure_semi(s, g) > maxw { g -= 1.0; }
        if self.measure_semi(s, g) <= maxw { (s.to_string(), g, true) } else { (self.fit(s, g, maxw), g, false) }
    }
    /// Cut a string to fit a width (adds an ellipsis).
    pub fn fit(&self, s: &str, size: f32, maxw: f32) -> String {
                if self.measure(s, size) <= maxw { return s.to_string(); }
        let mut out: String = s.chars().collect();
        while !out.is_empty() && self.measure(&(out.clone() + "…"), size) > maxw { out.pop(); }
        if pruefung_an() { if let Some(p) = PRUEFUNG.lock().unwrap().as_mut() { p.gekuerzt.insert(s.to_string()); } }
        out + "…"
    }
    /// Word-wrap into lines.
    pub fn wrap(&self, s: &str, size: f32, maxw: f32) -> Vec<String> {
        // a line break in the text starts a new line
        if s.contains('\n') { return s.split('\n').flat_map(|z| self.wrap(z, size, maxw)).collect(); }
        let mut lines = vec![];
        let mut cur = String::new();
        for word in s.split_whitespace() {
            let t = if cur.is_empty() { word.to_string() } else { crate::tf!("ui.farm.text", cur, word) };
            if self.measure(&t, size) > maxw && !cur.is_empty() {
                lines.push(cur);
                cur = word.to_string();
            } else {
                cur = t;
            }
        }
        if !cur.is_empty() { lines.push(cur); }
        lines
    }
    pub fn paragraph(&self, s: &str, x: f32, y: f32, size: f32, maxw: f32, c: Color) -> f32 {
        let mut yy = y;
        for l in self.wrap(s, size, maxw) {
            self.text(&l, x, yy, size, c);
            yy += size * 1.3;
        }
        yy
    }

    /// Karte (guide §6): round corners, 1 px line, soft shadow.
    /// A tile drawn without `card` (blocks of a message, chips): its texts are kept inside it like a card's.
    pub fn rahmen(&self, r: Rect) { self.rahmen.borrow_mut().push(r); }
    /// text_passend for every text (Nachbesserung 22.11.): a text that would run over the right edge of its card or tile
    /// first gets smaller (down to 80 %), then is cut with „…“; hovering it shows it whole. Returns the text and size to draw.
    fn im_rahmen(&self, s: &str, x: f32, y: f32, size: f32, mess: &dyn Fn(&str, f32) -> f32) -> (String, f32) {
        let r = self.rahmen.borrow().iter().copied().filter(|r| r.contains(vec2(x + 1.0, y - size * 0.4))).min_by(|a, b| (a.w * a.h).partial_cmp(&(b.w * b.h)).unwrap());
        let w = mess(s, size);
        if pruefung_an() { pruefung_text(s, x, y, w); }
        let Some(r) = r else { return (s.to_string(), size) };
        let rand = r.x + r.w - 6.0;
        if x + w <= rand { return (s.to_string(), size); }
        let maxw = (rand - x).max(10.0);
        let mut g = size;
        while g > size * 0.8 && mess(s, g) > maxw { g -= 1.0; }
        if pruefung_an() { if let Some(p) = PRUEFUNG.lock().unwrap().as_mut() { p.kartenrand.insert(format!("{s} (Karte bis {:.0}, Text bis {:.0})", r.x + r.w, x + w)); } }
        if mess(s, g) <= maxw { return (s.to_string(), g); }
        let mut out: String = s.to_string();
        while !out.is_empty() && mess(&(out.clone() + "…"), g) > maxw { out.pop(); }
        if Rect::new(x, y - g, maxw, g + 4.0).contains(self.mouse) { *self.tooltip.borrow_mut() = Some(s.to_string()); }
        (out + "…", g)
    }
    pub fn card(&self, r: Rect) {
        self.rahmen(r);
        rrect(r.x, r.y + 4.0, r.w, r.h, 15.0, Theme::schatten());
        rrect(r.x, r.y, r.w, r.h, 15.0, Theme::linie());
        rrect(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0, 14.0, Theme::flaeche());
    }
    /// Karte mit farbigem Kopf (guide §3.2): the head carries the area colour of the page.
    /// Verlaufskurve (Teil 3): the values as a line with the last point marked, faint guide lines and the range on the left.
    /// `oben_klein`: smaller is better and stands at the top (the table place, 1 at the top).
    pub fn kurve(&self, r: Rect, werte: &[f32], farbe: Color, oben_klein: bool, fmt: &dyn Fn(f32) -> String) {
        rrect(r.x, r.y, r.w, r.h, 6.0, Theme::erhoehung());
        if werte.len() < 2 { return; }
        let (mut lo, mut hi) = werte.iter().fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        if hi - lo < 1e-3 { lo -= 1.0; hi += 1.0; }
        let (oben, unten) = if oben_klein { (lo, hi) } else { (hi, lo) };
        // the label column is as wide as the longer of the two range labels
        let lw = self.measure(&fmt(oben), 11.0).max(self.measure(&fmt(unten), 11.0));
        let (x0, w) = (r.x + lw + 14.0, r.w - lw - 24.0);
        let (y0, h) = (r.y + 8.0, r.h - 16.0);
        for k in 0..3 { let y = y0 + h * k as f32 / 2.0; draw_line(x0, y, x0 + w, y, 1.0, alpha(Theme::linie(), 0.6)); }
        let y = |v: f32| { let t = (v - lo) / (hi - lo); if oben_klein { y0 + h * t } else { y0 + h * (1.0 - t) } };
        self.text_right(&fmt(oben), x0 - 6.0, y0 + 5.0, 11.0, Theme::gedaempft());
        self.text_right(&fmt(unten), x0 - 6.0, y0 + h + 3.0, 11.0, Theme::gedaempft());
        let pt = |i: usize| vec2(x0 + w * i as f32 / (werte.len() - 1) as f32, y(werte[i]));
        for i in 1..werte.len() { let (a, b) = (pt(i - 1), pt(i)); draw_line(a.x, a.y, b.x, b.y, 2.5, farbe); }
        let e = pt(werte.len() - 1);
        draw_circle(e.x, e.y, 4.5, farbe);
        draw_circle(e.x, e.y, 2.0, Theme::flaeche());
    }
    pub fn card_titled(&self, r: Rect, title: &str, edge: Color) {
        self.card(r);
        let c = if edge == Theme::akzent() { Theme::bereich() } else { edge };
        rrect(r.x + 1.0, r.y + 1.0, r.w - 2.0, 34.0, 14.0, c);
        draw_rectangle(r.x + 1.0, r.y + 20.0, r.w - 2.0, 15.0, c);
        self.title(&title.to_uppercase(), r.x + 16.0, r.y + 26.0, 19.0, on(c));
    }

    /// Hauptknopf (primary) or Zweitknopf (outline) – guide §6.
    pub fn button(&mut self, r: Rect, label: &str, primary: bool) -> bool {
        let h = self.hover(r);
        let pressed = h && is_mouse_button_down(MouseButton::Left);
        let off = if pressed { 1.0 } else { 0.0 };
        let voll = label;
        let (label, size, ganz) = self.einzeilig(voll, (r.h * 0.5).clamp(13.0, 22.0), r.w - 14.0);
        if !ganz && h { *self.tooltip.borrow_mut() = Some(voll.to_string()); }
        let label = label.as_str();
        if primary {
            rrect(r.x, r.y + 3.0, r.w, r.h, 10.0, alpha(Theme::akzent(), 0.25));
            rrect(r.x, r.y + off, r.w, r.h, 10.0, if h { hex(0x6fd0fa) } else { Theme::akzent() });
            self.text_center(label, r.x + r.w / 2.0, r.y + r.h / 2.0 + size * 0.35 + off, size, Theme::auf_akzent());
        } else {
            rrect(r.x, r.y + off, r.w, r.h, 10.0, if h { alpha(Theme::akzent(), 0.6) } else { Theme::linie() });
            rrect(r.x + 1.5, r.y + 1.5 + off, r.w - 3.0, r.h - 3.0, 9.0, if h { Theme::erhoehung() } else { Theme::flaeche() });
            self.text_center(label, r.x + r.w / 2.0, r.y + r.h / 2.0 + size * 0.35 + off, size, Theme::text());
        }
        if self.hit(r) { self.play(crate::id("klick")); true } else { false }
    }
    /// Big main action: Barlow Condensed italic, upper case, shadow in the button colour.
    pub fn button_gold(&mut self, r: Rect, label: &str) -> bool {
        let h = self.hover(r);
        rrect(r.x, r.y + 4.0, r.w, r.h, 10.0, alpha(Theme::akzent(), 0.3));
        rrect(r.x, r.y, r.w, r.h, 10.0, if h { hex(0x6fd0fa) } else { Theme::akzent() });
        let size = (r.h * 0.48).clamp(14.0, 24.0);
        let t = label.to_uppercase();
        let w = self.measure_title(&t, size);
        self.title(&t, r.x + r.w / 2.0 - w / 2.0, r.y + r.h / 2.0 + size * 0.35, size, Theme::auf_akzent());
        if self.hit(r) { self.play(crate::id("klick")); true } else { false }
    }
    /// Filter pills (guide §6): fully round; active = club colour frame on the soft/deep club tone.
    pub fn segment(&mut self, r: Rect, items: &[&str], sel: usize) -> Option<usize> {
        let gap = 6.0;
        let w = (r.w - gap * (items.len() - 1) as f32) / items.len() as f32;
        let mut out = None;
        for (i, it) in items.iter().enumerate() {
            let cell = Rect::new(r.x + i as f32 * (w + gap), r.y, w, r.h);
            let h = self.hover(cell);
            let rad = r.h / 2.0;
            if i == sel {
                rrect(cell.x, cell.y, cell.w, cell.h, rad, Theme::verein());
                rrect(cell.x + 2.0, cell.y + 2.0, cell.w - 4.0, cell.h - 4.0, rad - 2.0, if Theme::dark() { Theme::verein_tief() } else { Theme::verein_weich() });
            } else {
                rrect(cell.x, cell.y, cell.w, cell.h, rad, Theme::linie());
                rrect(cell.x + 1.0, cell.y + 1.0, cell.w - 2.0, cell.h - 2.0, rad - 1.0, if h { Theme::erhoehung() } else { Theme::flaeche() });
            }
            let size = (r.h * 0.5).clamp(12.0, 19.0);
            let label = self.fit(it, size, w - 12.0);
            self.text_center(&label, cell.x + w / 2.0, cell.y + r.h / 2.0 + size * 0.35, size, if i == sel { Theme::text() } else { Theme::text2() });
            if self.hit(cell) && i != sel { self.play(crate::id("klick")); out = Some(i); }
        }
        out
    }
    /// Slider 0..=max integer; returns new value if dragged.
    pub fn slider(&mut self, r: Rect, v: i32, min: i32, max: i32) -> Option<i32> {
        let t = (v - min) as f32 / (max - min) as f32;
        draw_rectangle(r.x, r.y + r.h / 2.0 - 2.0, r.w, 4.0, Theme::ueberlagerung());
        draw_rectangle(r.x, r.y + r.h / 2.0 - 2.0, r.w * t, 4.0, Theme::akzent());
        draw_circle(r.x + r.w * t, r.y + r.h / 2.0, 8.0, Theme::text());
        draw_circle_lines(r.x + r.w * t, r.y + r.h / 2.0, 8.0, 2.0, Theme::akzent());
        let grab = Rect::new(r.x - 8.0, r.y - 4.0, r.w + 16.0, r.h + 8.0);
        if self.hover(grab) && is_mouse_button_down(MouseButton::Left) || (self.clicked && grab.contains(self.mouse)) {
            let nt = ((self.mouse.x - r.x) / r.w).clamp(0.0, 1.0);
            let nv = min + (nt * (max - min) as f32).round() as i32;
            if nv != v { return Some(nv); }
        }
        None
    }
    /// Value pill (Notenpille / Wertzelle).
    pub fn pill(&self, cx: f32, cy: f32, v: f32, w: f32) {
        let c = Theme::skala(v);
        rrect(cx - w / 2.0, cy - 11.0, w, 22.0, 11.0, c);
        let s = format!("{}", v.round() as i32);
        self.text_center(&s, cx, cy + 6.0, 17.0, hex(0x10141c));
    }
    pub fn note_pill(&self, cx: f32, cy: f32, r: f32) {
        let c = Theme::note(r);
        rrect(cx - 21.0, cy - 11.0, 42.0, 22.0, 5.0, c);
        self.text_center(&format!("{:.1}", r), cx, cy + 6.0, 17.0, hex(0x10141c));
    }
    pub fn pos_tag(&self, x: f32, y: f32, pos: &str) {
        let c = Theme::teil(pos);
        let w = self.measure_semi(pos, 13.0) + 10.0;
        rrect(x, y - 9.0, w, 18.0, 4.0, alpha(c, 0.15));
        rrect_line(x, y - 9.0, w, 18.0, 4.0, alpha(c, 0.7));
        self.text_semi(pos, x + 5.0, y + 4.5, 13.0, c);
    }
    pub fn bar(&self, x: f32, y: f32, w: f32, h: f32, t: f32, c: Color) {
        rrect(x, y, w, h, h / 2.0, Theme::ueberlagerung());
        if t > 0.0 { rrect(x, y, w * t.clamp(0.0, 1.0), h, h / 2.0, c); }
    }
    /// Small banner inside a page (club tones).
    pub fn banner(&self, r: Rect, title: &str, sub: &str, _club: Color) {
        gradient_h(r.x, r.y, r.w, r.h, Theme::verein_tief(), alpha(Theme::verein_tief(), 0.6));
        self.rahmen(r);
        draw_rectangle(r.x, r.y + r.h - 4.0, r.w, 4.0, Theme::verein());
        self.title(title, r.x + 20.0, r.y + 36.0, 30.0, hex(0xffffff));
        self.text(sub, r.x + 20.0, r.y + r.h - 14.0, 18.0, hex(0xd8dde6));
    }
    /// Bühnen-Banner (guide §2): club gradient, Überzeile, huge italic title, one sentence, 4 px edge, jersey row.
    pub fn stage(&self, r: Rect, over: &str, title: &str, question: &str) {
        rrect(r.x, r.y, r.w, r.h, 15.0, Theme::verein_tief());
        self.rahmen(r);
        self.rahmen(Rect::new(r.x, r.y, r.w - 390.0, r.h));
        gradient_h(r.x + r.w * 0.45, r.y, r.w * 0.55 - 15.0, r.h - 4.0, alpha(Theme::verein(), 0.0), alpha(Theme::verein(), 0.55));
        draw_rectangle(r.x + 15.0, r.y + r.h - 4.0, r.w - 30.0, 4.0, Theme::verein());
        for i in 0..5 {
            let x = r.x + r.w - 380.0 + i as f32 * 72.0;
            let lift = if i % 2 == 0 { 0.0 } else { 10.0 };
            self.jersey(x, r.y + r.h * 0.55 + lift, 52.0, if i == 2 { hex(0xffffff) } else { Theme::verein() }, &format!("{}", [7, 10, 1, 9, 4][i]));
        }
        self.text_semi(&over.to_uppercase(), r.x + 26.0, r.y + 30.0, 15.0, alpha(hex(0xffffff), 0.75));
        self.title(&title.to_uppercase(), r.x + 24.0, r.y + 82.0, 54.0, hex(0xffffff));
        self.text(question, r.x + 26.0, r.y + 112.0, 19.0, hex(0xe6eaf0));
    }
    /// Trikot (guide §6): jersey outline in a colour, number or position on it.
    pub fn jersey(&self, cx: f32, cy: f32, size: f32, c: Color, label: &str) {
        let s = size / 2.0;
        let dark = Color::new(c.r * 0.7, c.g * 0.7, c.b * 0.7, 1.0);
        // sleeves
        draw_triangle(vec2(cx - s * 0.55, cy - s * 0.85), vec2(cx - s, cy - s * 0.35), vec2(cx - s * 0.55, cy - s * 0.05), dark);
        draw_triangle(vec2(cx + s * 0.55, cy - s * 0.85), vec2(cx + s, cy - s * 0.35), vec2(cx + s * 0.55, cy - s * 0.05), dark);
        draw_triangle(vec2(cx - s, cy - s * 0.35), vec2(cx - s * 0.78, cy - s * 0.1), vec2(cx - s * 0.55, cy - s * 0.05), dark);
        draw_triangle(vec2(cx + s, cy - s * 0.35), vec2(cx + s * 0.78, cy - s * 0.1), vec2(cx + s * 0.55, cy - s * 0.05), dark);
        // body
        draw_rectangle(cx - s * 0.55, cy - s * 0.85, s * 1.1, s * 1.75, c);
        draw_triangle(vec2(cx - s * 0.2, cy - s * 0.85), vec2(cx + s * 0.2, cy - s * 0.85), vec2(cx, cy - s * 0.6), dark);
        let fs = (size * 0.38).max(10.0);
        let w = measure_text(label, Some(&self.head), fs as u16, 1.0).width;
        self.title(label, cx - w / 2.0, cy + fs * 0.45, fs, on(c));
    }
    /// Stimmungs-Smiley: mouth up / straight / down, coloured good / middle / bad.
    pub fn smiley(&self, cx: f32, cy: f32, r: f32, mood: f32) {
        let c = if mood >= 60.0 { Theme::gut() } else if mood >= 40.0 { Theme::achtung() } else { Theme::problem() };
        draw_circle(cx, cy, r + 1.5, Theme::flaeche());
        draw_circle(cx, cy, r, c);
        let ink = hex(0x10141c);
        draw_circle(cx - r * 0.35, cy - r * 0.25, r * 0.13, ink);
        draw_circle(cx + r * 0.35, cy - r * 0.25, r * 0.13, ink);
        let bend = if mood >= 60.0 { 1.0 } else if mood >= 40.0 { 0.0 } else { -1.0 };
        let pts: Vec<Vec2> = (0..=6).map(|i| { let t = i as f32 / 6.0 - 0.5; vec2(cx + t * r * 1.1, cy + r * 0.35 + bend * (0.25 - t * t) * r * 0.9) }).collect();
        for w in pts.windows(2) { draw_line(w[0].x, w[0].y, w[1].x, w[1].y, (r * 0.14).max(1.2), ink); }
    }
    /// Avatar: initials on a colour circle (guide §3.5).
    /// A drawn figure for someone without a photo (board, agent, journalist, scout, staff); the role is guessed
    /// from the name or sender (e.g. "Präsidium", "Sportredaktion"), look and colours are fixed by the name.
    pub fn avatar(&self, cx: f32, cy: f32, r: f32, name: &str) { self.figur(cx, cy, r, name, Rolle::aus_name(name)); }
    /// A drawn figure inside a circle: head with skin tone and hair, clothes by role (suit and tie for the board,
    /// suit and sunglasses for agents, glasses and notebook colours for the press, a cap for scouts, a kit for players).
    pub fn figur(&self, cx: f32, cy: f32, r: f32, name: &str, rolle: Rolle) {
        let h = name.bytes().fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
        let wahl = |n: u32, k: u32| ((h >> k) % n) as usize;
        let grund = hex([0x3ecf8e, 0xf5b942, 0x38bdf8, 0x2dd4bf, 0xf472b6, 0xfb923c, 0xa78bfa, 0x94a3b8][wahl(8, 0)]);
        draw_circle(cx, cy, r, with_light(grund, if Theme::dark() { 0.32 } else { 0.86 }, 0.6));
        let haut = hex([0xf2d3b8, 0xe0b48c, 0xc68e62, 0x9c6a43, 0x6b4428][wahl(5, 3)]);
        let haar = hex([0x2b1d14, 0x5a3a22, 0x8a5a2b, 0xc9a86b, 0x9a9a9a, 0x1b1b1b][wahl(6, 6)]);
        let kleid = match rolle {
            Rolle::Vorstand | Rolle::Berater => hex([0x1f2937, 0x23314d, 0x3b3b3b][wahl(3, 9)]),
            Rolle::Spieler => Theme::verein(),
            _ => hex([0x2563eb, 0x0f766e, 0x7c3aed, 0xb45309, 0x475569][wahl(5, 9)]),
        };
        // shoulders: a trapeze that stays inside the circle
        let (sy, by) = (cy + r * 0.38, cy + r * 0.8);
        let (sw, bw) = (r * 0.42, r * 0.6);
        draw_triangle(vec2(cx - sw, sy), vec2(cx + sw, sy), vec2(cx + bw, by), kleid);
        draw_triangle(vec2(cx - sw, sy), vec2(cx - bw, by), vec2(cx + bw, by), kleid);
        draw_circle(cx - sw + r * 0.04, sy + r * 0.04, r * 0.08, kleid);
        draw_circle(cx + sw - r * 0.04, sy + r * 0.04, r * 0.08, kleid);
        match rolle {
            Rolle::Vorstand | Rolle::Berater => {
                draw_triangle(vec2(cx - r * 0.13, sy), vec2(cx + r * 0.13, sy), vec2(cx, sy + r * 0.3), WHITE);
                if rolle == Rolle::Vorstand { draw_triangle(vec2(cx - r * 0.04, sy + r * 0.04), vec2(cx + r * 0.04, sy + r * 0.04), vec2(cx, sy + r * 0.3), hex(0xb91c1c)); }
            }
            Rolle::Spieler => { draw_triangle(vec2(cx - r * 0.12, sy), vec2(cx + r * 0.12, sy), vec2(cx, sy + r * 0.14), on(kleid)); }
            _ => { draw_rectangle(cx - r * 0.1, sy, r * 0.2, r * 0.06, alpha(WHITE, 0.6)); }
        }
        // neck, head, hair
        draw_rectangle(cx - r * 0.09, cy + r * 0.18, r * 0.18, r * 0.22, haut);
        let (kx, ky, kr) = (cx, cy - r * 0.12, r * 0.35);
        draw_circle(kx, ky, kr, haut);
        let glatze = wahl(7, 12) == 0;
        if !glatze {
            draw_rectangle(kx - kr, ky - kr * 0.55, kr * 2.0, kr * 0.35, haar);
            draw_circle(kx, ky - kr * 0.35, kr * 0.92, haar);
            draw_circle(kx, ky + kr * 0.12, kr * 0.86, haut);
            if wahl(3, 15) == 0 { draw_rectangle(kx - kr * 1.02, ky - kr * 0.3, kr * 0.26, kr * 1.1, haar); draw_rectangle(kx + kr * 0.76, ky - kr * 0.3, kr * 0.26, kr * 1.1, haar); }
        }
        // eyes and a calm mouth
        let auge = hex(0x1f1f1f);
        match rolle {
            Rolle::Berater => { draw_rectangle(kx - kr * 0.72, ky - kr * 0.05, kr * 0.62, kr * 0.28, auge); draw_rectangle(kx + kr * 0.1, ky - kr * 0.05, kr * 0.62, kr * 0.28, auge); draw_line(kx - kr * 0.1, ky + kr * 0.05, kx + kr * 0.1, ky + kr * 0.05, kr * 0.08, auge); }
            Rolle::Presse => {
                draw_circle(kx - kr * 0.38, ky + kr * 0.08, kr * 0.08, auge); draw_circle(kx + kr * 0.38, ky + kr * 0.08, kr * 0.08, auge);
                draw_circle_lines(kx - kr * 0.38, ky + kr * 0.08, kr * 0.26, kr * 0.07, auge); draw_circle_lines(kx + kr * 0.38, ky + kr * 0.08, kr * 0.26, kr * 0.07, auge);
            }
            _ => { draw_circle(kx - kr * 0.35, ky + kr * 0.08, kr * 0.08, auge); draw_circle(kx + kr * 0.35, ky + kr * 0.08, kr * 0.08, auge); }
        }
        draw_line(kx - kr * 0.25, ky + kr * 0.5, kx + kr * 0.25, ky + kr * 0.5, kr * 0.07, alpha(auge, 0.7));
        if rolle == Rolle::Scout {
            let kappe = hex(0x334155);
            draw_rectangle(kx - kr * 0.95, ky - kr * 0.75, kr * 1.9, kr * 0.42, kappe);
            draw_circle(kx, ky - kr * 0.72, kr * 0.9, kappe);
            draw_rectangle(kx - kr * 0.1, ky - kr * 0.38, kr * 1.35, kr * 0.16, kappe);
        }
    }
    /// Zustand leer: a small drawing, one sentence.
    pub fn empty(&self, r: Rect, text: &str) {
        let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h * 0.38);
        draw_circle(cx, cy, 34.0, alpha(Theme::bereich(), 0.18));
        draw_circle(cx, cy, 22.0, Theme::flaeche());
        draw_circle_lines(cx, cy, 22.0, 2.0, Theme::bereich());
        draw_poly(cx, cy, 5, 8.0, -90.0, Theme::bereich());
        for k in 0..5 { let a = (k as f32 * 72.0 - 90.0).to_radians(); draw_line(cx + a.cos() * 8.0, cy + a.sin() * 8.0, cx + a.cos() * 22.0, cy + a.sin() * 22.0, 1.8, Theme::bereich()); }
        let mut y = cy + 64.0;
        for l in self.wrap(text, 19.0, r.w - 60.0) { self.text_center(&l, cx, y, 19.0, Theme::text2()); y += 25.0; }
    }
    /// Fallback crest: initials on the club colour.
    pub fn crest(&self, cx: f32, cy: f32, r: f32, c1: Color, c2: Color, short: &str) {
        draw_circle(cx, cy, r + 2.0, Theme::linie());
        draw_circle(cx, cy, r, c1);
        draw_circle_lines(cx, cy, r * 0.8, 2.0, alpha(c2, 0.6));
        let size = r * 0.6;
        let w = measure_text(short, Some(&self.head), size as u16, 1.0).width;
        self.title(short, cx - w / 2.0, cy + size * 0.35, size, c2);
    }
    /// Club crest from the FL-World pictures; initials only when no picture exists (guide §2).
    pub fn club_crest(&self, cx: f32, cy: f32, r: f32, id: u32, c1: Color, short: &str) {
        self.wappen_orte.borrow_mut().push((id, vec2(cx, cy), r));
        let tex = self.crests.borrow_mut().entry(id).or_insert_with(|| load_crest(self.bilder.verein.get(&id).and_then(|q| q.lesen()), id)).clone();
        if let Some(t) = tex {
            draw_texture_ex(&t, cx - r, cy - r, WHITE, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
        } else {
            self.crest(cx, cy, r, c1, on(c1), short);
        }
    }
    /// The main colour of a club's crest (world folder first, then assets/wappen), or None without a coloured crest.
    pub fn wappen_farbe(&self, id: u32) -> Option<u32> {
        *self.wappen_farben.borrow_mut().entry(id).or_insert_with(|| {
            let bytes = self.bilder.verein.get(&id).and_then(|q| q.lesen()).or_else(|| {
                let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
                [std::path::PathBuf::from(format!("assets/wappen/{id}.png")), exe.join(format!("../../assets/wappen/{id}.png")), exe.join(format!("assets/wappen/{id}.png"))].iter().find_map(|p| std::fs::read(p).ok())
            })?;
            let img = Image::from_file_with_format(&bytes, None).ok()?;
            hauptfarbe(&img.bytes)
        })
    }
    /// Switches to the crests and logos of world folder `welt` (built once per world; nothing is written).
    pub fn bilder_fuer_welt(&mut self, welt: Option<std::path::PathBuf>) {
        if self.bilder_fuer == welt { return; }
        let sider = welt.as_ref().and_then(|w| w.parent()).and_then(|p| p.parent()).map(|p| p.to_path_buf());
        self.bilder = match (&welt, sider) { (Some(w), Some(s)) => crate::fl27::bilder_index(w, &s), _ => Default::default() };
        self.bilder_fuer = welt;
        self.crests.borrow_mut().clear();
        self.wappen_farben.borrow_mut().clear();
        self.logos.borrow_mut().clear();
        self.fotos.borrow_mut().clear();
        self.stadien.borrow_mut().clear();
    }
    /// A player's photo texture: loaded once and kept (world folders, game archives, then the picture cache of the
    /// Python version, read only); None while the frame's loading budget is used up or when there is no photo.
    fn foto_tex(&self, id: u32) -> Option<Texture2D> {
        if let Some(t) = self.fotos.borrow().get(&id) { return t.clone(); }
        if self.foto_budget.get() == 0 { return None; }
        self.foto_budget.set(self.foto_budget.get() - 1);
        let cache = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(format!(".cache/fl-world/bilder/spieler/{id}.png"));
        let t = self.bilder.spieler.get(&id).and_then(|q| q.lesen()).or_else(|| std::fs::read(&cache).ok()).and_then(|b| png_tex(&b));
        self.fotos.borrow_mut().insert(id, t.clone());
        t
    }
    /// The one building block for player photos everywhere: the photo in a circle, else the initials placeholder (no message).
/// Konzept Animationen Stufe A: a little footballer in his club's kit with his shirt number, built from simple
    /// shapes (skin, hair and build fixed by his id), breathing gently. `fuss`: where his feet stand; `h`: his height.
    pub fn kicker(&self, cx: f32, fuss: f32, h: f32, id: u32, trikot: Color, hose: Color, nr: u8, gross: f32) {
        let z = |k: u32| ((id.wrapping_mul(2654435761).wrapping_add(k.wrapping_mul(40503))) >> 8) % 1000;
        let u = h / 10.0;
        let haut = hex([0xf2d3b8, 0xe0b48c, 0xc68e62, 0x9c6a43, 0x6b4428][(z(1) % 5) as usize]);
        let haar = hex([0x2b1d14, 0x5a3a22, 0x8a5a2b, 0xc9a86b, 0x1b1b1b][(z(2) % 5) as usize]);
        let breit = 1.0 + (gross - 0.5) * 0.25;
        let atem = (get_time() as f32 * 2.2 + (id % 7) as f32).sin() * 0.12 * u;
        let schuh = hex(0x111111);
        // legs with socks in the shirt colour, shoes
        for s in [-1.0f32, 1.0] {
            let x = cx + s * 0.55 * u * breit;
            draw_rectangle(x - 0.3 * u, fuss - 2.6 * u, 0.6 * u, 1.3 * u, haut);
            draw_rectangle(x - 0.32 * u, fuss - 1.4 * u, 0.64 * u, 1.2 * u, trikot);
            draw_ellipse(x + s * 0.1 * u, fuss - 0.15 * u, 0.5 * u, 0.22 * u, 0.0, schuh);
        }
        // shorts, shirt with sleeves and arms
        let top = fuss - 7.4 * u + atem;
        draw_rectangle(cx - 1.25 * u * breit, fuss - 3.7 * u, 2.5 * u * breit, 1.3 * u, hose);
        draw_rectangle(cx - 1.3 * u * breit, top + 1.6 * u, 2.6 * u * breit, 2.6 * u, trikot);
        for s in [-1.0f32, 1.0] {
            let x = cx + s * 1.45 * u * breit;
            draw_rectangle(x - 0.35 * u, top + 1.7 * u, 0.7 * u, 0.9 * u, trikot);
            draw_rectangle(x - 0.25 * u, top + 2.6 * u, 0.5 * u, 1.3 * u, haut);
        }
        if nr > 0 { self.text_center(&nr.to_string(), cx, top + 3.4 * u, 1.5 * u, on(trikot)); }
        // head and hair
        draw_rectangle(cx - 0.3 * u, top + 1.2 * u, 0.6 * u, 0.5 * u, haut);
        draw_circle(cx, top + 0.6 * u, 1.0 * u, haut);
        match z(3) % 4 {
            0 => draw_circle(cx, top + 0.2 * u, 0.95 * u, haar),
            1 => draw_rectangle(cx - 1.0 * u, top - 0.4 * u, 2.0 * u, 0.7 * u, haar),
            2 => { draw_circle(cx - 0.5 * u, top - 0.1 * u, 0.6 * u, haar); draw_circle(cx + 0.5 * u, top - 0.1 * u, 0.6 * u, haar); }
            _ => {}
        }
        draw_circle(cx, top + 0.85 * u, 0.75 * u, haut);
        draw_circle(cx - 0.32 * u, top + 0.7 * u, 0.1 * u, hex(0x1b1b1b));
        draw_circle(cx + 0.32 * u, top + 0.7 * u, 0.1 * u, hex(0x1b1b1b));
    }
    pub fn player_photo(&self, cx: f32, cy: f32, r: f32, id: u32, name: &str) {
        self.foto_orte.borrow_mut().push((id, vec2(cx, cy), r));
        let Some(t) = self.foto_tex(id) else { self.figur(cx, cy, r, name, Rolle::Spieler); return };
        draw_circle(cx, cy, r, Theme::erhoehung());
        // the photo inside the circle (inscribed square, head at the top)
        let s = r * 1.414;
        let k = (s / t.width()).max(s / t.height());
        let (w, h) = (t.width() * k, t.height() * k);
        let src = Rect::new((w - s) / 2.0 / k, 0.0, s / k, s / k);
        draw_texture_ex(&t, cx - s / 2.0, cy - s / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(s, s)), source: Some(src), ..Default::default() });
        draw_circle_lines(cx, cy, r, 1.5, Theme::linie());
    }
    /// The player a right click points at: of the photos drawn in this frame, the nearest one in the mouse's row
    /// (at most the photo's height above or below, 320 units beside it). Its id.
    /// The club of the row under the mouse: the nearest crest at the same height.
    pub fn wappen_unter_maus(&self) -> Option<u32> {
        let m = self.mouse;
        self.wappen_orte.borrow().iter().filter(|(_, c, r)| (m.y - c.y).abs() <= r + 10.0 && (m.x - c.x).abs() <= 420.0)
            .min_by(|a, b| (a.1 - m).length().partial_cmp(&(b.1 - m).length()).unwrap()).map(|x| x.0)
    }
    pub fn foto_unter_maus(&self) -> Option<u32> {
        let m = self.mouse;
        self.foto_orte.borrow().iter().filter(|(_, c, r)| (m.y - c.y).abs() <= r + 14.0 && (m.x - c.x).abs() <= 320.0)
            .min_by(|a, b| (a.1 - m).length().partial_cmp(&(b.1 - m).length()).unwrap()).map(|x| x.0)
    }
    /// A photo with the shirt number as a small badge at its lower left.
    pub fn player_photo_nr(&self, cx: f32, cy: f32, r: f32, id: u32, name: &str, no: u8, ring: Color) {
        self.player_photo(cx, cy, r, id, name);
        draw_circle_lines(cx, cy, r, 2.0, ring);
        let (bx, by, br) = (cx - r * 0.72, cy + r * 0.72, (r * 0.36).max(8.0));
        draw_circle(bx, by, br, ring);
        self.text_center(&no.to_string(), bx, by + br * 0.45, br * 1.15, on(ring));
    }
    /// The player's photo in a box (profile head); false draws nothing.
    pub fn spieler_foto(&self, x: f32, y: f32, w: f32, h: f32, id: u32) -> bool {
        let Some(t) = self.foto_tex(id) else { return false };
        let k = (w / t.width()).min(h / t.height());
        draw_texture_ex(&t, x + (w - t.width() * k) / 2.0, y + h - t.height() * k, WHITE, DrawTextureParams { dest_size: Some(vec2(t.width() * k, t.height() * k)), ..Default::default() });
        true
    }
    /// The logo of a competition (emb_<id>_l.png of the world), when the world has one; false draws nothing.
    pub fn wettbewerb_logo(&self, cx: f32, cy: f32, r: f32, cid: Option<u16>) -> bool {
        let Some(id) = cid else { return false };
        let tex = self.logos.borrow_mut().entry(id).or_insert_with(|| self.bilder.emblem.get(&id).and_then(|q| q.lesen()).and_then(|b| png_tex(&b))).clone();
        let Some(t) = tex else { return false };
        let k = (2.0 * r / t.width().max(t.height())).min(4.0);
        draw_texture_ex(&t, cx - t.width() * k / 2.0, cy - t.height() * k / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(t.width() * k, t.height() * k)), ..Default::default() });
        true
    }
    /// The home stadium picture of a team (fl27::stadien_index), cropped to fill `r`; false when there is none.
    /// Without one in the game, the home ground's photo from Wikimedia Commons (netzbilder.rs) with its credit line.
    pub fn stadion_bild(&self, r: Rect, team: u32, verein: &str) -> bool {
        let tex = self.stadien.borrow_mut().entry(team).or_insert_with(|| self.bilder.stadion.get(&team).and_then(|q| q.lesen()).and_then(|b| png_tex(&b))).clone();
        // shortcut: the club is searched by name only (no country), a namesake abroad could win; add the country if that shows up
        let (t, nachweis) = match tex { Some(t) => (t, String::new()), None => match self.netz_bild(crate::netzbilder::Art::Stadion, verein, "") { Some(x) => x, None => return false } };
        let k = (r.w / t.width()).max(r.h / t.height());
        let src = Rect::new((t.width() - r.w / k) / 2.0, (t.height() - r.h / k) / 2.0, r.w / k, r.h / k);
        draw_texture_ex(&t, r.x, r.y, WHITE, DrawTextureParams { dest_size: Some(vec2(r.w, r.h)), source: Some(src), ..Default::default() });
        if !nachweis.is_empty() { self.nachweise.borrow_mut().push((r, nachweis)); }
        true
    }
    /// A picture from a free web source as a texture with its credit; None while it loads or when there is none.
    pub fn netz_bild(&self, art: crate::netzbilder::Art, name: &str, land: &str) -> Option<(Texture2D, String)> {
        let k = (art, name.to_string());
        if let Some(t) = self.netz.borrow().get(&k) { return t.clone(); }
        if self.foto_budget.get() == 0 { return None; }
        let t = match crate::netzbilder::holen(art, name, land) {
            crate::netzbilder::Stand::Laeuft => return None,
            crate::netzbilder::Stand::Fehlt => None,
            crate::netzbilder::Stand::Da(b) => { self.foto_budget.set(self.foto_budget.get() - 1); std::fs::read(&b.datei).ok().and_then(|x| png_tex(&x)).map(|t| (t, b.nachweis)) }
        };
        self.netz.borrow_mut().insert(k, t.clone());
        t
    }
    /// A journalist: the free photo (head part) in a circle when there is one, else the drawn press figure.
    /// `land` is the country of a real reporter of the media pool; "" (invented journalist) never searches the web.
    pub fn presse_kopf(&self, cx: f32, cy: f32, r: f32, name: &str, land: &str) {
        let Some((t, nachweis)) = (!land.is_empty()).then(|| self.netz_bild(crate::netzbilder::Art::Person, name, land)).flatten() else { self.figur(cx, cy, r, name, Rolle::Presse); return };
        draw_circle(cx, cy, r, Theme::erhoehung());
        let s = r * 1.414;
        let seite = t.width().min(t.height());
        // portraits: the face sits in the upper part
        let src = Rect::new((t.width() - seite) / 2.0, ((t.height() - seite) * 0.15).max(0.0), seite, seite);
        draw_texture_ex(&t, cx - s / 2.0, cy - s / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(s, s)), source: Some(src), ..Default::default() });
        draw_circle_lines(cx, cy, r, 1.5, Theme::linie());
        if !nachweis.is_empty() && Rect::new(cx - r, cy - r, 2.0 * r, 2.0 * r).contains(self.mouse) { *self.tooltip.borrow_mut() = Some(nachweis); }
    }
    /// An outlet's logo (public domain/CC0 from Commons) on a light chip inside `r`; false when there is none.
    pub fn medium_logo(&self, r: Rect, name: &str, land: &str) -> bool {
        let Some((t, _)) = self.netz_bild(crate::netzbilder::Art::Medium, name, land) else { return false };
        rrect(r.x, r.y, r.w, r.h, 6.0, WHITE);
        let k = ((r.w - 8.0) / t.width()).min((r.h - 8.0) / t.height());
        let (w, h) = (t.width() * k, t.height() * k);
        draw_texture_ex(&t, r.x + (r.w - w) / 2.0, r.y + (r.h - h) / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        true
    }
    /// The credit lines of this frame's web pictures, small in their bottom right corner (CC BY / BY-SA need them).
    pub fn nachweise_zeichnen(&self) {
        for (r, n) in self.nachweise.borrow().iter() {
            let n = self.fit(n, 11.0, r.w - 16.0);
            let w = self.measure(&n, 11.0) + 10.0;
            rrect(r.x + r.w - w - 4.0, r.y + r.h - 19.0, w, 16.0, 4.0, alpha(BLACK, 0.55));
            self.text(&n, r.x + r.w - w + 1.0, r.y + r.h - 7.0, 11.0, alpha(WHITE, 0.9));
        }
    }
    /// Five drawn stars from x (centre height y), `n` of them filled (halves allowed); the fonts have no star glyph.
    pub fn sterne(&self, x: f32, y: f32, n: f32, r: f32) {
        for i in 0..5 {
            let cx = x + r + i as f32 * r * 2.3;
            let f = (n - i as f32).clamp(0.0, 1.0);
            let col = if f >= 1.0 { Theme::gold() } else if f >= 0.5 { alpha(Theme::gold(), 0.55) } else { alpha(Theme::text(), 0.15) };
            let pt = |k: usize| { let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 5.0; let rr = if k % 2 == 0 { r } else { r * 0.45 }; vec2(cx + a.cos() * rr, y + a.sin() * rr) };
            for k in 0..10 { draw_triangle(vec2(cx, y), pt(k), pt((k + 1) % 10), col); }
        }
    }
    /// Text input field; edits `s` while focused. Returns true on Enter.
    /// This frame's typing into `s`: letters, Backspace and Ctrl+V (paste), at most `max` characters. Every text field
    /// goes through here (23.11.: the country/club search read the keys itself after they were already taken).
    pub fn tippen(&self, s: &mut String, max: usize) -> bool {
        let vorher = s.clone();
        let strg = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) || is_key_down(KeyCode::LeftSuper);
        if strg && is_key_pressed(KeyCode::V) {
            let neu = macroquad::miniquad::window::clipboard_get().unwrap_or_default();
            for c in neu.trim().trim_matches('"').chars().filter(|c| !c.is_control()) { if s.chars().count() < max { s.push(c); } }
        } else {
            for c in self.typed.iter() { if s.chars().count() < max && !(strg && c.eq_ignore_ascii_case(&'v')) { s.push(*c); } }
        }
        if self.key_back { s.pop(); }
        *s != vorher
    }
    pub fn input(&mut self, r: Rect, s: &mut String, focused: bool, placeholder: &str) -> bool { self.input_max(r, s, focused, placeholder, 40) }
    /// A text field of at most `max` characters; Ctrl+V pastes (beta 23.11.: folder paths could not be pasted).
    pub fn input_max(&mut self, r: Rect, s: &mut String, focused: bool, placeholder: &str, max: usize) -> bool {
        rrect(r.x, r.y, r.w, r.h, r.h / 2.0, if focused { Theme::akzent() } else { Theme::linie() });
        rrect(r.x + 1.5, r.y + 1.5, r.w - 3.0, r.h - 3.0, r.h / 2.0 - 1.5, Theme::flaeche());
        if focused { self.tippen(s, max); }
        let size = 20.0;
        if s.is_empty() {
            self.text(placeholder, r.x + 18.0, r.y + r.h / 2.0 + 7.0, size, Theme::gedaempft());
        } else {
            // long text: show the end that fits (the part being typed)
            let mut shown: &str = s;
            while self.measure(shown, size) > r.w - 36.0 { let mut it = shown.chars(); it.next(); shown = it.as_str(); }
            self.text(shown, r.x + 18.0, r.y + r.h / 2.0 + 7.0, size, Theme::text());
        }
        if focused && (self.time * 2.0) as i32 % 2 == 0 {
            let w = self.measure(s, size).min(r.w - 36.0);
            draw_rectangle(r.x + 19.0 + w, r.y + 10.0, 2.0, r.h - 20.0, Theme::akzent());
        }
        focused && self.key_enter
    }
}

/// Virtual units per window pixel: the width 1400 fills the window; only below the minimum height (very wide windows)
/// the layout keeps 760 and extra width stays on the right.
pub fn view_scale() -> f32 { (breite() / screen_width()).max(hoehe() / screen_height()) }
/// Left offset of the layout (0 since the width follows the window; kept for the camera and the mouse).
pub fn versatz() -> f32 { ((screen_width() * view_scale() - breite()) / 2.0).max(0.0) }
pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    // non-overlapping pieces, so translucent colours stay even
    draw_rectangle(x + r, y, w - 2.0 * r, h, c);
    if r <= 0.5 { return; }
    draw_rectangle(x, y + r, r, h - 2.0 * r, c);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, c);
    let corners = [(x + r, y + r, 180.0f32), (x + w - r, y + r, 270.0), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0)];
    for (cx, cy, a0) in corners {
        let n = 6;
        for i in 0..n {
            let a = (a0 + 90.0 * i as f32 / n as f32).to_radians();
            let b = (a0 + 90.0 * (i + 1) as f32 / n as f32).to_radians();
            draw_triangle(vec2(cx, cy), vec2(cx + a.cos() * r, cy + a.sin() * r), vec2(cx + b.cos() * r, cy + b.sin() * r), c);
        }
    }
}
/// A picture from a world folder; a broken file gives None (then the next source) instead of a crash.
fn png_tex(b: &[u8]) -> Option<Texture2D> { let i = Image::from_file_with_format(b, None).ok()?; let t = Texture2D::from_image(&i); t.set_filter(FilterMode::Linear); Some(t) }
fn load_crest(welt: Option<Vec<u8>>, id: u32) -> Option<Texture2D> {
    // the world's own crest first (fl27::bilder_index), then assets/wappen, then the drawn crest (caller)
    if let Some(t) = welt.and_then(|b| png_tex(&b)) { return Some(t); }
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
    let home = std::env::var("HOME").unwrap_or_default();
    let cands = [
        std::path::PathBuf::from(format!("assets/wappen/{id}.png")),
        exe.join(format!("../../assets/wappen/{id}.png")),
        exe.join(format!("assets/wappen/{id}.png")),
        std::path::PathBuf::from(format!("{home}/.cache/fl-world/bilder/verein/{id}.png")),
    ];
    let bytes = cands.iter().find_map(|p| std::fs::read(p).ok())?;
    let t = Texture2D::from_file_with_format(&bytes, None);
    t.set_filter(FilterMode::Linear);
    Some(t)
}
/// Horizontal gradient (left colour to right colour).
pub fn gradient_h(x: f32, y: f32, w: f32, h: f32, a: Color, b: Color) {
    let n = 24;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let c = Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t);
        draw_rectangle(x + w * i as f32 / n as f32, y, w / n as f32 + 0.5, h, c);
    }
}
/// The same from top (a) to bottom (b).
pub fn gradient_v(x: f32, y: f32, w: f32, h: f32, a: Color, b: Color) {
    let n = 24;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let c = Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t);
        draw_rectangle(x, y + h * i as f32 / n as f32, w, h / n as f32 + 0.5, c);
    }
}
pub fn rrect_line(x: f32, y: f32, w: f32, h: f32, _r: f32, c: Color) {
    draw_rectangle_lines(x, y, w, h, 1.0, c);
}

#[cfg(test)]
mod farb_tests {
    use super::*;

    fn bild(px: &[(u8, u8, u8, u8)], n: usize) -> Vec<u8> { px.iter().flat_map(|&(r, g, b, a)| std::iter::repeat([r, g, b, a]).take(n)).flatten().collect() }

    #[test]
    fn hauptfarbe_aus_dem_wappen() {
        // red crest with white and black parts and a transparent border: red wins
        let rot = bild(&[(220, 20, 40, 255), (255, 255, 255, 255), (0, 0, 0, 255), (0, 90, 200, 0)], 100);
        let f = hauptfarbe(&rot).unwrap();
        assert!(f >> 16 > 180 && (f >> 8) & 0xff < 60, "{f:06x}");
        // yellow with a little black (Dortmund-like)
        let gelb = bild(&[(253, 225, 0, 255), (0, 0, 0, 255)], 100);
        let g = hauptfarbe(&gelb).unwrap();
        assert!(g >> 16 > 200 && (g >> 8) & 0xff > 180 && g & 0xff < 60, "{g:06x}");
        // only grey, white and black: no colour, the caller falls back
        assert!(hauptfarbe(&bild(&[(128, 128, 128, 255), (255, 255, 255, 255), (10, 10, 10, 255)], 100)).is_none());
    }

    #[test]
    fn akzent_bleibt_lesbar() {
        let hell = hex(0xf6f3ee);
        let dunkel = hex(0x0f1218);
        for f in [0xfde100u32, 0x000080, 0xd4152f, 0x38bdf8, 0x00ff00] {
            assert!(kontrast(lesbar(hex(f), hell, 3.0), hell) >= 3.0, "{f:06x} hell");
            assert!(kontrast(lesbar(hex(f), dunkel, 3.0), dunkel) >= 3.0, "{f:06x} dunkel");
            let a = lesbar(hex(f), hell, 3.0);
            assert!(kontrast(on(a), a) >= 4.5 || kontrast(on(a), a) >= 3.0, "Schrift auf dem Akzent");
        }
    }
}
let mut touch_wheel = 0.0f32;
let mut next_y = None;
for t in touches() {
    if t.phase == TouchPhase::Moved || t.phase == TouchPhase::Stationary {
        next_y = Some(t.position.y);
        if let Some(prev_y) = self.last_touch_y {
            if t.phase == TouchPhase::Moved {
                touch_wheel += (t.position.y - prev_y) * 0.04;
            }
        }
        break;
    }
}
self.last_touch_y = next_y;
self.wheel += touch_wheel;
