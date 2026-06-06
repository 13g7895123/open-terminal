#[derive(Clone, Copy)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba {
    const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

const TRANSPARENT: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};
const BG: Rgba = Rgba::rgb(15, 23, 42);
const BG_GLOW: Rgba = Rgba::rgb(30, 41, 59);
const PANEL: Rgba = Rgba::rgb(12, 18, 32);
const PANEL_HI: Rgba = Rgba::rgb(24, 34, 56);
const BORDER: Rgba = Rgba::rgb(71, 85, 105);
const ACCENT: Rgba = Rgba::rgb(34, 197, 94);
const TEXT: Rgba = Rgba::rgb(226, 232, 240);

pub fn render_icon_rgba(size: u32) -> Vec<u8> {
    let scale = 4usize;
    let hi = size as usize * scale;
    let mut canvas = Canvas::new(hi, hi);

    let s = hi as f32;

    canvas.fill_rounded_rect(0.0, 0.0, s, s, s * 0.23, BG);
    canvas.fill_rounded_rect(s * 0.05, s * 0.05, s * 0.90, s * 0.90, s * 0.20, BG_GLOW);
    canvas.fill_rounded_rect(s * 0.085, s * 0.085, s * 0.83, s * 0.83, s * 0.17, BG);

    canvas.fill_rounded_rect(s * 0.16, s * 0.20, s * 0.56, s * 0.48, s * 0.08, BORDER);
    canvas.fill_rounded_rect(s * 0.18, s * 0.22, s * 0.52, s * 0.44, s * 0.07, PANEL);
    canvas.fill_rect(s * 0.18, s * 0.22, s * 0.52, s * 0.08, PANEL_HI);
    canvas.fill_circle(s * 0.225, s * 0.26, s * 0.018, ACCENT);
    canvas.fill_circle(s * 0.27, s * 0.26, s * 0.018, TEXT);

    let prompt_thickness = s * 0.042;
    canvas.stroke_segment(
        (s * 0.28, s * 0.40),
        (s * 0.38, s * 0.49),
        prompt_thickness,
        TEXT,
    );
    canvas.stroke_segment(
        (s * 0.28, s * 0.58),
        (s * 0.38, s * 0.49),
        prompt_thickness,
        TEXT,
    );
    canvas.fill_rect(s * 0.42, s * 0.55, s * 0.14, s * 0.042, TEXT);

    let arrow_thickness = s * 0.078;
    canvas.stroke_segment(
        (s * 0.48, s * 0.64),
        (s * 0.80, s * 0.32),
        arrow_thickness,
        ACCENT,
    );
    canvas.fill_triangle(
        (s * 0.80, s * 0.22),
        (s * 0.90, s * 0.42),
        (s * 0.70, s * 0.42),
        ACCENT,
    );
    canvas.fill_triangle(
        (s * 0.66, s * 0.36),
        (s * 0.76, s * 0.46),
        (s * 0.58, s * 0.54),
        ACCENT,
    );

    canvas.downsample(scale)
}

struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<Rgba>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![TRANSPARENT; width * height],
        }
    }

    fn downsample(self, scale: usize) -> Vec<u8> {
        let out_w = self.width / scale;
        let out_h = self.height / scale;
        let mut out = Vec::with_capacity(out_w * out_h * 4);

        for oy in 0..out_h {
            for ox in 0..out_w {
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut a = 0u32;
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = self.pixels[(oy * scale + sy) * self.width + (ox * scale + sx)];
                        r += px.r as u32;
                        g += px.g as u32;
                        b += px.b as u32;
                        a += px.a as u32;
                    }
                }
                let samples = (scale * scale) as u32;
                out.extend_from_slice(&[
                    (r / samples) as u8,
                    (g / samples) as u8,
                    (b / samples) as u8,
                    (a / samples) as u8,
                ]);
            }
        }

        out
    }

    fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgba) {
        self.fill_shape(x, y, w, h, |px, py| px >= x && px <= x + w && py >= y && py <= y + h, color);
    }

    fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgba) {
        self.fill_shape(
            cx - radius,
            cy - radius,
            radius * 2.0,
            radius * 2.0,
            |px, py| {
                let dx = px - cx;
                let dy = py - cy;
                dx * dx + dy * dy <= radius * radius
            },
            color,
        );
    }

    fn fill_rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Rgba) {
        self.fill_shape(
            x,
            y,
            w,
            h,
            |px, py| point_in_rounded_rect(px, py, x, y, w, h, radius),
            color,
        );
    }

    fn stroke_segment(&mut self, start: (f32, f32), end: (f32, f32), thickness: f32, color: Rgba) {
        let min_x = start.0.min(end.0) - thickness;
        let min_y = start.1.min(end.1) - thickness;
        let max_x = start.0.max(end.0) + thickness;
        let max_y = start.1.max(end.1) + thickness;
        let radius = thickness * 0.5;

        self.fill_shape(
            min_x,
            min_y,
            max_x - min_x,
            max_y - min_y,
            |px, py| distance_to_segment(px, py, start, end) <= radius,
            color,
        );
    }

    fn fill_triangle(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32), color: Rgba) {
        let min_x = a.0.min(b.0).min(c.0);
        let min_y = a.1.min(b.1).min(c.1);
        let max_x = a.0.max(b.0).max(c.0);
        let max_y = a.1.max(b.1).max(c.1);
        self.fill_shape(
            min_x,
            min_y,
            max_x - min_x,
            max_y - min_y,
            |px, py| point_in_triangle((px, py), a, b, c),
            color,
        );
    }

    fn fill_shape<F>(&mut self, x: f32, y: f32, w: f32, h: f32, contains: F, color: Rgba)
    where
        F: Fn(f32, f32) -> bool,
    {
        let min_x = x.floor().max(0.0) as usize;
        let min_y = y.floor().max(0.0) as usize;
        let max_x = (x + w).ceil().min(self.width as f32) as usize;
        let max_y = (y + h).ceil().min(self.height as f32) as usize;

        for py in min_y..max_y {
            for px in min_x..max_x {
                let sample_x = px as f32 + 0.5;
                let sample_y = py as f32 + 0.5;
                if contains(sample_x, sample_y) {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, src: Rgba) {
        let dst = &mut self.pixels[y * self.width + x];
        if src.a == 255 || dst.a == 0 {
            *dst = src;
            return;
        }

        let src_a = src.a as f32 / 255.0;
        let dst_a = dst.a as f32 / 255.0;
        let out_a = src_a + dst_a * (1.0 - src_a);
        if out_a <= f32::EPSILON {
            *dst = TRANSPARENT;
            return;
        }

        let blend = |src_c: u8, dst_c: u8| -> u8 {
            let src_c = src_c as f32 / 255.0;
            let dst_c = dst_c as f32 / 255.0;
            (((src_c * src_a) + (dst_c * dst_a * (1.0 - src_a))) / out_a * 255.0).round() as u8
        };

        *dst = Rgba {
            r: blend(src.r, dst.r),
            g: blend(src.g, dst.g),
            b: blend(src.b, dst.b),
            a: (out_a * 255.0).round() as u8,
        };
    }
}

fn point_in_rounded_rect(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32, radius: f32) -> bool {
    let rx = radius.min(w * 0.5);
    let ry = radius.min(h * 0.5);
    let clamped_x = px.clamp(x + rx, x + w - rx);
    let clamped_y = py.clamp(y + ry, y + h - ry);
    let dx = px - clamped_x;
    let dy = py - clamped_y;
    dx * dx + dy * dy <= rx.min(ry).powi(2)
}

fn distance_to_segment(px: f32, py: f32, start: (f32, f32), end: (f32, f32)) -> f32 {
    let vx = end.0 - start.0;
    let vy = end.1 - start.1;
    let wx = px - start.0;
    let wy = py - start.1;
    let c1 = vx * wx + vy * wy;
    if c1 <= 0.0 {
        return ((px - start.0).powi(2) + (py - start.1).powi(2)).sqrt();
    }
    let c2 = vx * vx + vy * vy;
    if c2 <= c1 {
        return ((px - end.0).powi(2) + (py - end.1).powi(2)).sqrt();
    }
    let t = c1 / c2;
    let proj_x = start.0 + t * vx;
    let proj_y = start.1 + t * vy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

fn point_in_triangle(p: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let area = |p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)| {
        (p1.0 * (p2.1 - p3.1) + p2.0 * (p3.1 - p1.1) + p3.0 * (p1.1 - p2.1)) * 0.5
    };

    let total = area(a, b, c).abs();
    let a1 = area(p, b, c).abs();
    let a2 = area(a, p, c).abs();
    let a3 = area(a, b, p).abs();
    (a1 + a2 + a3 - total).abs() <= 0.5
}
