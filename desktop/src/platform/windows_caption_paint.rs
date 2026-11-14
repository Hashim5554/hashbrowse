//! Small, cached premultiplied-alpha caption surface. Native BUTTONs own input
//! and accessibility separately; no theme brush can enter this visible bitmap.
use windows::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};

pub const HEIGHT: f64 = 32.0;
// A 28 DIP cell holds a 20x12 capsule with 4 DIP of air either side, so the
// three lights group the way a title bar groups them.
pub const BUTTON_WIDTH: f64 = 28.0;
pub const PADDING: f64 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Appearance {
    pub dpi: u32,
    pub dark: bool,
    pub active: bool,
    pub maximized: bool,
    pub hot: Option<usize>,
    pub pressed: Option<usize>,
    pub focused: Option<usize>,
    /// A button the window's styles withdraw is drawn dim and never lights.
    pub enabled: [bool; 3],
    /// Window, text, highlight, highlighted text and disabled text, in
    /// COLORREF order.
    pub contrast: Option<[u32; 5]>,
}

pub fn dimensions(dpi: u32) -> (i32, i32, i32, i32) {
    let scale = dpi.max(96) as f64 / 96.0;
    let button = (BUTTON_WIDTH * scale).round() as i32;
    let padding = (PADDING * scale).round() as i32;
    (
        button
            .saturating_mul(3)
            .saturating_add(padding.saturating_mul(2)),
        (HEIGHT * scale).round() as i32,
        button,
        padding,
    )
}

pub struct Surface {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut u32,
    size: (i32, i32),
    appearance: Option<Appearance>,
    presented: Option<(i32, i32, i32, u8)>,
}

impl Surface {
    pub fn new(dpi: u32) -> Option<Self> {
        let (width, height, _, _) = dimensions(dpi);
        // Bound a native allocation even if a malformed DPI reaches this adapter.
        if width > 4096 || height > 1024 {
            return None;
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        unsafe {
            let mut bits = std::ptr::null_mut();
            let bitmap = CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
            let dc = CreateCompatibleDC(None);
            if dc.0.is_null() || bits.is_null() {
                let _ = DeleteObject(bitmap.into());
                if !dc.0.is_null() {
                    let _ = DeleteDC(dc);
                }
                return None;
            }
            let previous = SelectObject(dc, bitmap.into());
            if previous.0.is_null() || previous.0 as isize == -1 {
                let _ = DeleteDC(dc);
                let _ = DeleteObject(bitmap.into());
                return None;
            }
            Some(Self {
                dc,
                bitmap,
                previous,
                bits: bits.cast(),
                size: (width, height),
                appearance: None,
                presented: None,
            })
        }
    }

    pub fn present(
        &mut self,
        hwnd: HWND,
        position: POINT,
        shown: i32,
        alpha: u8,
        appearance: Appearance,
    ) -> bool {
        let (width, height, _, _) = dimensions(appearance.dpi);
        if self.size != (width, height) {
            let Some(replacement) = Self::new(appearance.dpi) else {
                return false;
            };
            *self = replacement;
        }
        let shown = shown.clamp(0, height);
        if shown == 0 || alpha == 0 {
            return true;
        }
        if self.appearance != Some(appearance) {
            // This buffer is private to the UI thread and remains selected into
            // our memory DC until Drop. Animation reuses it without rasterizing.
            let pixels =
                unsafe { std::slice::from_raw_parts_mut(self.bits, (width * height) as usize) };
            raster(pixels, appearance);
            self.appearance = Some(appearance);
            self.presented = None;
        }
        let presentation = (position.x, position.y, shown, alpha);
        if self.presented == Some(presentation) {
            return true;
        }
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: alpha,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        // Crop the travelling portion rather than using a binary window region.
        let result = unsafe {
            UpdateLayeredWindow(
                hwnd,
                None,
                Some(&position),
                Some(&SIZE {
                    cx: width,
                    cy: shown,
                }),
                Some(self.dc),
                Some(&POINT {
                    x: 0,
                    y: height - shown,
                }),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            )
        };
        if result.is_ok() {
            self.presented = Some(presentation);
        }
        result.is_ok()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.dc);
        }
    }
}

fn line_distance(x: f64, y: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let vx = b.0 - a.0;
    let vy = b.1 - a.1;
    let t = (((x - a.0) * vx + (y - a.1) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
    (x - a.0 - t * vx).hypot(y - a.1 - t * vy)
}

fn ink_distance(index: usize, maximized: bool, x: f64, y: f64) -> f64 {
    match index {
        0 => line_distance(x, y, (-4.5, 0.5), (4.5, 0.5)),
        1 if maximized => {
            let front = (x + 1.0).abs().max((y - 1.0).abs());
            (front - 3.5)
                .abs()
                .min(line_distance(x, y, (-2.5, -4.5), (4.5, -4.5)))
                .min(line_distance(x, y, (4.5, -4.5), (4.5, 2.5)))
        }
        1 => (x.abs().max(y.abs()) - 4.0).abs(),
        _ => line_distance(x, y, (-4.0, -4.0), (4.0, 4.0)).min(line_distance(
            x,
            y,
            (4.0, -4.0),
            (-4.0, 4.0),
        )),
    }
}

/// Three identical capsules. They are one control drawn three times, so they
/// share one colour and one shape; only the ink inside differs. macOS tells a
/// window's three actions apart by hue, but a browser already drawn in blue
/// gains more from one quiet, precise material that brightens under the
/// pointer than from a second colour language in its top-right corner.
const LIGHT_HALF_WIDTH: f64 = 9.5;
const LIGHT_HALF_HEIGHT: f64 = 5.5;

/// A capsule's signed distance: one rounded bar, ends fully round.
fn capsule_distance(x: f64, y: f64) -> f64 {
    (x.abs() - (LIGHT_HALF_WIDTH - LIGHT_HALF_HEIGHT))
        .max(0.0)
        .hypot(y)
        - LIGHT_HALF_HEIGHT
}

/// The ink the capsules are drawn in: cool white on dark chrome, a deep
/// blue-grey on light. A window that is not in front fades toward its ground,
/// so the lights state the window's state without changing hue.
const INK_DARK: [f64; 3] = [222.0, 236.0, 255.0];
const INK_DARK_IDLE: [f64; 3] = [116.0, 138.0, 174.0];
const INK_LIGHT: [f64; 3] = [32.0, 56.0, 100.0];
const INK_LIGHT_IDLE: [f64; 3] = [132.0, 152.0, 184.0];
/// The filled state: the capsule under the pointer, one blue for all three,
/// with dark ink on it. Hover is a lamp switching on, not a second drawing.
const PLATE: [f64; 3] = [124.0, 178.0, 255.0];
const PLATE_PRESSED: [f64; 3] = [100.0, 152.0, 236.0];
const PLATE_INK: [f64; 3] = [9.0, 15.0, 27.0];

fn ink_of(appearance: Appearance) -> [f64; 3] {
    match (appearance.dark, appearance.active) {
        (true, true) => INK_DARK,
        (true, false) => INK_DARK_IDLE,
        (false, true) => INK_LIGHT,
        (false, false) => INK_LIGHT_IDLE,
    }
}

/// One light, at one point, composited onto a transparent premultiplied
/// sample. Drawn at 4x4 supersampling by the caller, so every contour here can
/// be a hard distance test and still arrive smooth.
#[allow(clippy::too_many_arguments)]
fn paint_light(
    pixel: &mut [f64; 4],
    appearance: Appearance,
    index: usize,
    live: bool,
    hot: bool,
    pressed: bool,
    x: f64,
    y: f64,
    scale: f64,
) {
    let distance = capsule_distance(x, y) * scale;
    let coverage = (0.5 - distance).clamp(0.0, 1.0);
    let stroke =
        (0.5 + 0.4 * scale - ink_distance(index, appearance.maximized, x, y) * scale).clamp(0.0, 1.0);

    if let Some(colors) = appearance.contrast {
        // High contrast draws filled, glyphs always on.
        if hot {
            over(pixel, color(colors[2]), coverage);
        }
        let slot = if !live {
            4
        } else if hot {
            3
        } else {
            1
        };
        over(pixel, color(colors[slot]), stroke);
        return;
    }

    if hot && live {
        over(pixel, if pressed { PLATE_PRESSED } else { PLATE }, coverage);
        // One hairline of light along the crown: the only ornament in the
        // control, and it describes a surface rather than decorating one.
        let crown = ((y + LIGHT_HALF_HEIGHT + 0.35) / 0.9).clamp(0.0, 1.0);
        over(pixel, [255.0, 255.0, 255.0], crown * coverage * 0.18);
        over(pixel, PLATE_INK, stroke);
    } else {
        // At rest the capsule is a faint plate inside a drawn contour. The rim
        // is the shape; the fill only has to keep the shape from reading flat.
        let presence = if live { 1.0 } else { 0.3 };
        let lower = ((y / LIGHT_HALF_HEIGHT) * 0.5 + 0.5).clamp(0.0, 1.0);
        let fill = 0.055 + 0.035 * lower;
        // A plate caught by light on dark chrome, and a shade on light chrome.
        // Either way it is a tint of the surface, never a colour of its own.
        if appearance.dark {
            over(pixel, [255.0, 255.0, 255.0], coverage * fill * presence * 0.7);
        } else {
            over(pixel, [0.0, 0.0, 0.0], coverage * fill * presence * 0.45);
        }
        let rim = (1.0 - ((distance + 0.6).abs() * scale / 0.5)).clamp(0.0, 1.0);
        over(pixel, ink_of(appearance), rim * 0.34 * presence);
        if live {
            over(pixel, ink_of(appearance), stroke * 0.5);
        }
    }

    if appearance.focused == Some(index) {
        // A keyboard focus ring, one step outside the capsule.
        let ring = (0.5 + 0.5 * scale - ((capsule_distance(x, y) + 1.7) * scale).abs())
            .clamp(0.0, 1.0);
        over(pixel, [150.0, 200.0, 255.0], ring * 0.9);
    }
}

fn color(value: u32) -> [f64; 3] {
    [
        (value & 255) as f64,
        ((value >> 8) & 255) as f64,
        ((value >> 16) & 255) as f64,
    ]
}

fn over(pixel: &mut [f64; 4], rgb: [f64; 3], alpha: f64) {
    for channel in 0..3 {
        pixel[channel] = rgb[channel] * alpha + pixel[channel] * (1.0 - alpha);
    }
    pixel[3] = alpha * 255.0 + pixel[3] * (1.0 - alpha);
}

/// Four samples per axis. The controls are small and drawn from analytic
/// distance fields, so a hard test would give jagged edges at every DPI; this
/// is the cheapest way to make a 12 DIP pill look drawn rather than plotted.
const SAMPLES: i32 = 4;

fn raster(pixels: &mut [u32], appearance: Appearance) {
    let (width, height, button, padding) = dimensions(appearance.dpi);
    let scale = appearance.dpi.max(96) as f64 / 96.0;
    let samples = (SAMPLES * SAMPLES) as f64;
    for row in 0..height {
        for column in 0..width {
            let index = (column - padding).div_euclid(button);
            let mut pixel = [0.0; 4];
            if let Some(colors) = appearance.contrast {
                over(&mut pixel, color(colors[0]), 1.0);
            }
            if (0..3).contains(&index) {
                let index = index as usize;
                let live = appearance.enabled[index];
                let hot = live && appearance.hot == Some(index);
                let pressed = appearance.pressed == Some(index) && hot;
                let origin = (padding + index as i32 * button) as f64 + button as f64 / 2.0;
                let mut total = [0.0; 4];
                for sy in 0..SAMPLES {
                    for sx in 0..SAMPLES {
                        let x = (column as f64
                            + (sx as f64 + 0.5) / SAMPLES as f64
                            - origin)
                            / scale;
                        let y = (row as f64 + (sy as f64 + 0.5) / SAMPLES as f64
                            - height as f64 / 2.0)
                            / scale;
                        let mut sample = [0.0; 4];
                        if let Some(colors) = appearance.contrast {
                            over(&mut sample, color(colors[0]), 1.0);
                        }
                        paint_light(
                            &mut sample,
                            appearance,
                            index,
                            live,
                            hot,
                            pressed,
                            x,
                            y,
                            scale,
                        );
                        for channel in 0..4 {
                            total[channel] += sample[channel] / samples;
                        }
                    }
                }
                // Background was seeded into `pixel` above and every sample is
                // itself already composited, so the average is the answer.
                pixel = total;
            }
            let [r, g, b, a] = pixel.map(|value| value.round().clamp(0.0, 255.0) as u32);
            pixels[(row * width + column) as usize] = (a << 24) | (r << 16) | (g << 8) | b;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_alpha_surface_accepts_cropped_frames_and_dpi_changes() {
        struct Window(HWND);
        impl Drop for Window {
            fn drop(&mut self) {
                let _ = unsafe { DestroyWindow(self.0) };
            }
        }
        let window = Window(
            unsafe {
                CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                    windows::core::w!("STATIC"),
                    windows::core::w!(""),
                    WS_POPUP,
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    None,
                    None,
                )
            }
            .expect("hidden test surface"),
        );
        let mut surface = Surface::new(96).expect("caption DIB");
        for dpi in [96, 120, 144, 192, 96] {
            let appearance = Appearance {
                dpi,
                dark: true,
                active: true,
                maximized: false,
                hot: Some(1),
                pressed: None,
                focused: None,
                enabled: [true; 3],
                contrast: None,
            };
            let (_, height, _, _) = dimensions(dpi);
            for shown in [1, height / 2, height] {
                assert!(surface.present(window.0, POINT { x: 0, y: 0 }, shown, 255, appearance));
            }
        }
        assert!(!unsafe { IsWindowVisible(window.0) }.as_bool());
    }
    #[test]
    fn caption_pixels_have_real_transparency_and_premultiplied_edges_at_every_dpi() {
        for dpi in [96, 120, 144, 192] {
            let appearance = Appearance {
                dpi,
                dark: true,
                active: true,
                maximized: false,
                hot: Some(1),
                pressed: None,
                focused: None,
                enabled: [true; 3],
                contrast: None,
            };
            let (width, height, _, _) = dimensions(dpi);
            let mut pixels = vec![0; (width * height) as usize];
            raster(&mut pixels, appearance);
            assert_eq!(pixels[0], 0);
            assert!(pixels.iter().any(|pixel| (1..255).contains(&(pixel >> 24))));
            for pixel in pixels {
                let alpha = pixel >> 24;
                assert!(((pixel >> 16) & 255) <= alpha);
                assert!(((pixel >> 8) & 255) <= alpha);
                assert!((pixel & 255) <= alpha);
            }
        }
    }
    /// Pixel luminance, premultiplied, in the order the DIB stores it.
    fn lum(pixel: u32) -> f64 {
        0.2126 * f64::from((pixel >> 16) & 255)
            + 0.7152 * f64::from((pixel >> 8) & 255)
            + 0.0722 * f64::from(pixel & 255)
    }

    fn render_one(appearance: Appearance) -> (Vec<u32>, i32) {
        let (width, height, _, _) = dimensions(appearance.dpi);
        let mut pixels = vec![0; (width * height) as usize];
        raster(&mut pixels, appearance);
        (pixels, width)
    }

    /// One pixel of the rendered strip, addressed from a light's own centre in
    /// DIP so the same assertion holds at every DPI.
    fn sample(pixels: &[u32], dpi: u32, index: usize, dx: f64, dy: f64) -> u32 {
        let (width, height, button, padding) = dimensions(dpi);
        let scale = dpi.max(96) as f64 / 96.0;
        let column = (padding + index as i32 * button) as f64 + button as f64 / 2.0 + dx * scale;
        let row = height as f64 / 2.0 + dy * scale;
        pixels[(row as i32 * width + column as i32) as usize]
    }

    fn resting(dpi: u32, hot: Option<usize>) -> Appearance {
        Appearance {
            dpi,
            dark: true,
            active: true,
            maximized: false,
            hot,
            pressed: None,
            focused: None,
            enabled: [true; 3],
            contrast: None,
        }
    }

    #[test]
    fn all_three_capsules_share_one_colour_and_one_shape() {
        let base = resting(96, None);
        let (pixels, _) = render_one(base);
        // The ends of a capsule carry no glyph, so they show the material
        // itself; if all three match there, the triad is one control.
        for dy in [-4.0, -2.5, 0.0, 2.5, 4.0] {
            let first = sample(&pixels, 96, 0, -8.0, dy);
            assert_eq!(first, sample(&pixels, 96, 1, -8.0, dy));
            assert_eq!(first, sample(&pixels, 96, 2, -8.0, dy));
        }
        // And the same holds when the window is not in front, which is the
        // other ink the triad can be drawn in.
        let idle = Appearance {
            active: false,
            ..base
        };
        let (pixels, _) = render_one(idle);
        let first = sample(&pixels, 96, 0, -8.0, 0.0);
        assert_eq!(first, sample(&pixels, 96, 1, -8.0, 0.0));
        assert_eq!(first, sample(&pixels, 96, 2, -8.0, 0.0));
    }

    #[test]
    fn a_capsule_at_rest_is_a_contour_around_a_faint_plate() {
        for dpi in [96, 144, 192] {
            let (pixels, _) = render_one(resting(dpi, None));
            let fill = lum(sample(&pixels, dpi, 0, 0.0, 1.5));
            // The edge is a hairline, so it is found rather than guessed: the
            // brightest pixel of the lower half of the capsule is the rim.
            let contour = (0..=60)
                .map(|step| lum(sample(&pixels, dpi, 0, 0.0, f64::from(step) * 0.1)))
                .fold(0.0_f64, f64::max);
            // The drawn edge is what gives the shape; the fill only stops it
            // reading flat, so the rim must be the brighter of the two.
            assert!(contour > fill * 1.5, "contour {contour} vs fill {fill}");
            assert!(fill > 0.0, "the plate should still be a plate");
            // Nothing glows at rest: no light escapes the capsule.
            assert_eq!(sample(&pixels, dpi, 0, 0.0, 9.0) >> 24, 0);
        }
    }

    #[test]
    fn hovering_fills_the_capsule_without_moving_its_outline() {
        let (rest, _) = render_one(resting(96, None));
        let (hover, _) = render_one(resting(96, Some(0)));

        // The two states describe the same capsule: every pixel the resting
        // contour marks is marked again when filled, so the control cannot
        // jump under the pointer.
        let marked = |pixels: &[u32]| -> Vec<bool> {
            pixels.iter().map(|pixel| pixel >> 24 > 40).collect()
        };
        let (rest_mark, hover_mark) = (marked(&rest), marked(&hover));
        assert!(rest_mark.iter().any(|on| *on));
        for (index, on) in rest_mark.iter().enumerate() {
            if *on {
                assert!(hover_mark[index]);
            }
        }

        let total = |pixels: &[u32]| pixels.iter().map(|pixel| lum(*pixel)).sum::<f64>();
        assert!(total(&hover) > total(&rest) * 2.0);
        // And it is the face that lights, not only the edge.
        assert!(lum(sample(&hover, 96, 0, 0.0, 2.0)) > lum(sample(&rest, 96, 0, 0.0, 2.0)) * 5.0);
    }

    #[test]
    fn the_glyph_is_inked_on_the_capsule_it_sits_on() {
        let (pixels, _) = render_one(resting(96, Some(0)));
        // Minimize's dash runs through the middle; a shoulder is pure fill.
        let dash = lum(sample(&pixels, 96, 0, 0.0, 0.0));
        let shoulder = lum(sample(&pixels, 96, 0, 7.5, -2.0));
        assert!(
            dash < shoulder,
            "the dash ({dash}) should read as ink over the fill ({shoulder})"
        );
    }

    #[test]
    fn a_withdrawn_button_is_dim_and_never_lights() {
        let appearance = |hot, enabled| Appearance {
            dpi: 96,
            dark: true,
            active: true,
            maximized: false,
            hot,
            pressed: None,
            focused: None,
            enabled,
            contrast: None,
        };
        let (width, height, _, _) = dimensions(96);
        let render = |appearance| {
            let mut pixels = vec![0; (width * height) as usize];
            raster(&mut pixels, appearance);
            pixels
        };
        let withdrawn = [true, false, true];
        assert_eq!(
            render(appearance(Some(1), withdrawn)),
            render(appearance(None, withdrawn))
        );
        // Ink is the only thing painted when the pointer is elsewhere.
        let ink = |pixels: &[u32]| {
            pixels
                .iter()
                .map(|pixel| u64::from(pixel >> 24))
                .sum::<u64>()
        };
        assert!(
            ink(&render(appearance(None, withdrawn))) < ink(&render(appearance(None, [true; 3])))
        );
    }
    #[test]
    fn caption_bitmap_is_bounded_and_high_contrast_is_opaque() {
        let appearance = Appearance {
            dpi: 192,
            dark: true,
            active: true,
            maximized: true,
            hot: Some(2),
            pressed: Some(2),
            focused: Some(0),
            enabled: [true, false, true],
            contrast: Some([0, 0xffffff, 0xffffff, 0, 0x808080]),
        };
        let (width, height, _, _) = dimensions(appearance.dpi);
        let mut pixels = vec![0; (width * height) as usize];
        assert!(pixels.len() * size_of::<u32>() < 72 * 1024);
        raster(&mut pixels, appearance);
        assert!(pixels.iter().all(|pixel| pixel >> 24 == 255));
    }
}
