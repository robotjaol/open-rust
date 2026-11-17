//! Procedurally generated application icon (no binary assets in the repository).

/// Distance from point to segment.
fn seg_dist(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let t = (((px - a.0) * abx + (py - a.1) * aby) / (abx * abx + aby * aby)).clamp(0.0, 1.0);
    ((px - a.0 - t * abx).powi(2) + (py - a.1 - t * aby).powi(2)).sqrt()
}

/// RGBA pixels of a `size`x`size` icon: indigo rounded square with a white `>_` prompt.
pub fn rgba(size: u32) -> Vec<u8> {
    let s = size as f32;
    let mut out = vec![0u8; (size * size * 4) as usize];
    let radius = s * 0.22;
    let half = s / 2.0;
    let stroke = s * 0.075;
    let chevron = [((0.30, 0.33), (0.50, 0.50)), ((0.50, 0.50), (0.30, 0.67))];
    let underscore = ((0.55, 0.68), (0.72, 0.68));
    for y in 0..size {
        for x in 0..size {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            // rounded-rect signed distance
            let (qx, qy) = (
                (fx - half).abs() - (half - radius),
                (fy - half).abs() - (half - radius),
            );
            let sd =
                (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - radius;
            let cover = (0.5 - sd).clamp(0.0, 1.0);
            let t = fy / s;
            let (r, g, b) = (
                (0x5B as f32 * (1.0 - t) + 0x2B as f32 * t) as u8,
                (0x6C as f32 * (1.0 - t) + 0x36 as f32 * t) as u8,
                (0xF2 as f32 * (1.0 - t) + 0xA8 as f32 * t) as u8,
            );
            let mut d = f32::MAX;
            for (a, b2) in chevron.iter().copied().chain([underscore]) {
                d = d.min(seg_dist(fx, fy, (a.0 * s, a.1 * s), (b2.0 * s, b2.1 * s)));
            }
            let ink = (stroke / 2.0 + 0.5 - d).clamp(0.0, 1.0);
            let mix = |bg: u8| (f32::from(bg) * (1.0 - ink) + 255.0 * ink) as u8;
            let i = ((y * size + x) * 4) as usize;
            out[i] = mix(r);
            out[i + 1] = mix(g);
            out[i + 2] = mix(b);
            out[i + 3] = (cover * 255.0) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_has_expected_size_and_transparent_corners() {
        let px = rgba(64);
        assert_eq!(px.len(), 64 * 64 * 4);
        assert_eq!(px[3], 0, "corner pixel must be transparent");
        let centre = ((32 * 64 + 32) * 4 + 3) as usize;
        assert_eq!(px[centre], 255);
    }
}
