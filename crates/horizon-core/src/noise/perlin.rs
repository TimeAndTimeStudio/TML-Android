use super::{
    DensityError, DensityVolume, GradientTable, PERLIN_GRADIENTS, dot, lerp2, lerp3, smoothstep,
    smoothstep_derivative, wrap,
};
use crate::{geometry::floor_i32, random::RandomSource};

#[derive(Clone, Debug)]
pub struct PerlinNoise {
    table: GradientTable,
    /// `table.permutations[i & 255]` with index 256 mirroring index 0, so
    /// adjacent `(i & 255, (i + 1) & 255)` lookups share one mask.
    perm: [u8; 257],
    /// `perm[i] & 15`, avoiding the final gradient mask on every corner lookup.
    perm_grad: [u8; 257],
    /// Precomputed X-to-Y permutation tables for `shifted_y0 = 0.0 + offsets[1]`
    /// (valid for `y == +0.0`, and for `y == -0.0` unless `offsets[1]` is also `-0.0`).
    perm_y0_lo: [u8; 257],
    perm_y0_hi: [u8; 257],
    /// Precomputed `PERLIN_GRADIENTS[g][1] * dy0` and `* (dy0 - 1.0)` at `y == 0.0`.
    y0_gy_dy: [f32; 16],
    y0_gy_dy1: [f32; 16],
    y0_fade_y: f32,
}

impl PerlinNoise {
    pub fn new(random: &mut impl RandomSource) -> Self {
        let table = GradientTable::new(random, 256.0);
        let raw = &table.permutations;
        let perm: [u8; 257] = std::array::from_fn(|i| raw[i & 255]);
        let perm_grad: [u8; 257] = std::array::from_fn(|i| perm[i] & 15);
        let shifted_y0 = table.offsets[1];
        let cell_y0 = floor_i32(shifted_y0);
        let dy0 = (shifted_y0 - f64::from(cell_y0)) as f32;
        let dy1 = dy0 - 1.0;
        let y0_fade_y = smoothstep(dy0);
        let perm_y0_lo: [u8; 257] = std::array::from_fn(|i| {
            let p = i32::from(perm[i]);
            perm[(p.wrapping_add(cell_y0) & 255) as usize]
        });
        let perm_y0_hi: [u8; 257] = std::array::from_fn(|i| {
            let p = i32::from(perm[i]);
            perm[(p.wrapping_add(cell_y0).wrapping_add(1) & 255) as usize]
        });
        let y0_gy_dy: [f32; 16] = std::array::from_fn(|g| PERLIN_GRADIENTS[g][1] * dy0);
        let y0_gy_dy1: [f32; 16] = std::array::from_fn(|g| PERLIN_GRADIENTS[g][1] * dy1);
        Self {
            table,
            perm,
            perm_grad,
            perm_y0_lo,
            perm_y0_hi,
            y0_gy_dy,
            y0_gy_dy1,
            y0_fade_y,
        }
    }
    pub fn table(&self) -> &GradientTable {
        &self.table
    }
    pub fn range(&self) -> [f32; 2] {
        [-2.0, 2.0]
    }
    #[inline]
    pub fn get2(&self, x: f64, z: f64) -> f32 {
        self.get3_y0(wrap(x), wrap(z))
    }
    #[inline]
    pub fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        if y == 0.0
            && !(y.to_bits() == (-0.0_f64).to_bits()
                && self.table.offsets[1].to_bits() == (-0.0_f64).to_bits())
        {
            return self.get3_y0(x, z);
        }
        let shifted_x = wrap(x) + self.table.offsets[0];
        let shifted_y = wrap(y) + self.table.offsets[1];
        let shifted_z = wrap(z) + self.table.offsets[2];
        let cell_x = floor_i32(shifted_x);
        let cell_y = floor_i32(shifted_y);
        let cell_z = floor_i32(shifted_z);
        let dx = (shifted_x - f64::from(cell_x)) as f32;
        let dy = (shifted_y - f64::from(cell_y)) as f32;
        let dz = (shifted_z - f64::from(cell_z)) as f32;
        self.sample_fast(cell_x, cell_y, cell_z, dx, dy, dz, dy)
    }

    /// Scalar `get3(x, +0.0, z)` reusing the immutable Y=0 cell, fade, and
    /// X-to-Y permutation tables. Bit-identical to `get3(x, y, z)` for `y == +0.0`,
    /// and for `y == -0.0` whenever `table.offsets[1]` is not `-0.0`.
    #[inline]
    pub(super) fn get3_y0(&self, x: f64, z: f64) -> f32 {
        let shifted_x = wrap(x) + self.table.offsets[0];
        let shifted_z = wrap(z) + self.table.offsets[2];
        let cell_x = floor_i32(shifted_x);
        let cell_z = floor_i32(shifted_z);
        let dx = (shifted_x - f64::from(cell_x)) as f32;
        let dz = (shifted_z - f64::from(cell_z)) as f32;
        let dx1 = dx - 1.0;
        let dz1 = dz - 1.0;
        let cx = (cell_x & 255) as usize;
        let lower = i32::from(self.perm_y0_lo[cx]);
        let lower_x = i32::from(self.perm_y0_lo[cx + 1]);
        let upper = i32::from(self.perm_y0_hi[cx]);
        let upper_x = i32::from(self.perm_y0_hi[cx + 1]);
        let z0 = (lower.wrapping_add(cell_z) & 255) as usize;
        let z1 = (lower_x.wrapping_add(cell_z) & 255) as usize;
        let z2 = (upper.wrapping_add(cell_z) & 255) as usize;
        let z3 = (upper_x.wrapping_add(cell_z) & 255) as usize;
        let g0 = usize::from(self.perm_grad[z0]);
        let g4 = usize::from(self.perm_grad[z0 + 1]);
        let g1 = usize::from(self.perm_grad[z1]);
        let g5 = usize::from(self.perm_grad[z1 + 1]);
        let g2 = usize::from(self.perm_grad[z2]);
        let g6 = usize::from(self.perm_grad[z2 + 1]);
        let g3 = usize::from(self.perm_grad[z3]);
        let g7 = usize::from(self.perm_grad[z3 + 1]);
        let dot_lo = |g: usize, px: f32, pz: f32| {
            let grad = PERLIN_GRADIENTS[g];
            grad[0] * px + self.y0_gy_dy[g] + grad[2] * pz
        };
        let dot_hi = |g: usize, px: f32, pz: f32| {
            let grad = PERLIN_GRADIENTS[g];
            grad[0] * px + self.y0_gy_dy1[g] + grad[2] * pz
        };
        let values = [
            dot_lo(g0, dx, dz),
            dot_lo(g1, dx1, dz),
            dot_hi(g2, dx, dz),
            dot_hi(g3, dx1, dz),
            dot_lo(g4, dx, dz1),
            dot_lo(g5, dx1, dz1),
            dot_hi(g6, dx, dz1),
            dot_hi(g7, dx1, dz1),
        ];
        lerp3([smoothstep(dx), self.y0_fade_y, smoothstep(dz)], values)
    }

    #[inline(always)]
    fn cell_grads(&self, p: i32, q: i32, y_cell: i32, cell_z: i32) -> ([usize; 4], [usize; 4]) {
        let py = (p.wrapping_add(y_cell) & 255) as usize;
        let qy = (q.wrapping_add(y_cell) & 255) as usize;
        let r00 = i32::from(self.perm[py]);
        let r01 = i32::from(self.perm[py + 1]);
        let r10 = i32::from(self.perm[qy]);
        let r11 = i32::from(self.perm[qy + 1]);
        let z00 = (r00.wrapping_add(cell_z) & 255) as usize;
        let z10 = (r10.wrapping_add(cell_z) & 255) as usize;
        let z01 = (r01.wrapping_add(cell_z) & 255) as usize;
        let z11 = (r11.wrapping_add(cell_z) & 255) as usize;
        (
            [
                usize::from(self.perm_grad[z00]),
                usize::from(self.perm_grad[z10]),
                usize::from(self.perm_grad[z00 + 1]),
                usize::from(self.perm_grad[z10 + 1]),
            ],
            [
                usize::from(self.perm_grad[z01]),
                usize::from(self.perm_grad[z11]),
                usize::from(self.perm_grad[z01 + 1]),
                usize::from(self.perm_grad[z11 + 1]),
            ],
        )
    }

    #[inline]
    fn sample_fast(
        &self,
        cell_x: i32,
        cell_y: i32,
        cell_z: i32,
        dx: f32,
        dy: f32,
        dz: f32,
        interpolation_y: f32,
    ) -> f32 {
        let cx = (cell_x & 255) as usize;
        let p = i32::from(self.perm[cx]);
        let q = i32::from(self.perm[cx + 1]);
        let ([g0, g1, g4, g5], [g2, g3, g6, g7]) = self.cell_grads(p, q, cell_y, cell_z);
        let dx1 = dx - 1.0;
        let dy1 = dy - 1.0;
        let dz1 = dz - 1.0;
        let cdot = |g: usize, px: f32, py: f32, pz: f32| {
            let grad = PERLIN_GRADIENTS[g];
            grad[0] * px + grad[1] * py + grad[2] * pz
        };
        let values = [
            cdot(g0, dx, dy, dz),
            cdot(g1, dx1, dy, dz),
            cdot(g2, dx, dy1, dz),
            cdot(g3, dx1, dy1, dz),
            cdot(g4, dx, dy, dz1),
            cdot(g5, dx1, dy, dz1),
            cdot(g6, dx, dy1, dz1),
            cdot(g7, dx1, dy1, dz1),
        ];
        lerp3(
            [smoothstep(dx), smoothstep(interpolation_y), smoothstep(dz)],
            values,
        )
    }

    fn coordinates(&self, point: [f64; 3]) -> ([i32; 3], [f64; 3]) {
        let shifted: [f64; 3] = std::array::from_fn(|i| wrap(point[i]) + self.table.offsets[i]);
        let cell = shifted.map(floor_i32);
        (
            cell,
            std::array::from_fn(|i| shifted[i] - f64::from(cell[i])),
        )
    }

    fn gradients(&self, cell: [i32; 3]) -> [[f32; 3]; 8] {
        let cx = (cell[0] & 255) as usize;
        self.gradients_at(
            i32::from(self.perm[cx]),
            i32::from(self.perm[cx + 1]),
            cell[1],
            cell[2],
        )
    }

    /// Corner gradients for a Y cell, with the X column's permutation already
    /// resolved.
    #[inline]
    fn gradients_at(&self, p: i32, q: i32, cell_y: i32, cell_z: i32) -> [[f32; 3]; 8] {
        let ([g0, g1, g4, g5], [g2, g3, g6, g7]) = self.cell_grads(p, q, cell_y, cell_z);
        [
            PERLIN_GRADIENTS[g0],
            PERLIN_GRADIENTS[g1],
            PERLIN_GRADIENTS[g2],
            PERLIN_GRADIENTS[g3],
            PERLIN_GRADIENTS[g4],
            PERLIN_GRADIENTS[g5],
            PERLIN_GRADIENTS[g6],
            PERLIN_GRADIENTS[g7],
        ]
    }

    fn corner_values(gradients: [[f32; 3]; 8], point: [f32; 3]) -> [f32; 8] {
        std::array::from_fn(|i| {
            dot(
                gradients[i],
                [
                    point[0] - (i & 1) as f32,
                    point[1] - ((i >> 1) & 1) as f32,
                    point[2] - ((i >> 2) & 1) as f32,
                ],
            )
        })
    }

    fn sample(&self, cell: [i32; 3], point: [f32; 3], interpolation_y: f32) -> f32 {
        self.sample_fast(
            cell[0],
            cell[1],
            cell[2],
            point[0],
            point[1],
            point[2],
            interpolation_y,
        )
    }

    /// Returns the sampled value and adds its gradient into the supplied array.
    pub fn noise_with_derivative(&self, x: f64, y: f64, z: f64, derivative: &mut [f32; 3]) -> f32 {
        let (cell, relative) = self.coordinates([x, y, z]);
        let point = relative.map(|v| v as f32);
        let gradients = self.gradients(cell);
        let values = Self::corner_values(gradients, point);
        let fade = point.map(smoothstep);
        let gradient =
            std::array::from_fn::<_, 3, _>(|axis| lerp3(fade, gradients.map(|g| g[axis])));
        let changes = [
            lerp2(
                fade[1],
                fade[2],
                [
                    values[1] - values[0],
                    values[3] - values[2],
                    values[5] - values[4],
                    values[7] - values[6],
                ],
            ),
            lerp2(
                fade[2],
                fade[0],
                [
                    values[2] - values[0],
                    values[6] - values[4],
                    values[3] - values[1],
                    values[7] - values[5],
                ],
            ),
            lerp2(
                fade[0],
                fade[1],
                [
                    values[4] - values[0],
                    values[5] - values[1],
                    values[6] - values[2],
                    values[7] - values[3],
                ],
            ),
        ];
        for axis in 0..3 {
            derivative[axis] += gradient[axis] + smoothstep_derivative(point[axis]) * changes[axis];
        }
        lerp3(fade, values)
    }

    /// Adds a scaled volume using vanilla's cached X/Z dot products and Y loop.
    /// Extra buffer elements are untouched; short buffers are rejected up front.
    pub fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) -> Result<(), DensityError> {
        self.add_volume::<true>(buffer, volume, xz_scale, y_scale, amplitude, None)
    }

    /// `BATCH` = false evaluates one Y at a time: the reference the batched loop is tested against.
    fn add_volume<const BATCH: bool>(
        &self,
        buffer: &mut [f32],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
        fudge_y_scale: Option<f64>,
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        let [size_x, size_y, size_z] = volume.sizes();
        // Every X/Z column walks the same Y coordinates, so their cells, offsets
        // and fades are computed once per volume instead of once per column.
        let mut stack = [YSample::default(); STACK_Y];
        let mut heap = Vec::new();
        let ys = if size_y <= STACK_Y {
            &mut stack[..size_y]
        } else {
            heap.resize(size_y, YSample::default());
            &mut heap[..]
        };
        for (y, sample) in ys.iter_mut().enumerate() {
            let input_y = f64::from(volume.coordinate(1, y)) * y_scale;
            let shifted_y = wrap(input_y) + self.table.offsets[1];
            let cell = floor_i32(shifted_y);
            let dy = shifted_y - f64::from(cell);
            *sample = YSample {
                cell,
                fade: smoothstep(dy as f32),
                offset: match fudge_y_scale {
                    Some(scale) => (dy - compute_fudge_y(input_y, dy, scale)) as f32,
                    None => dy as f32,
                },
            };
        }
        let ys = &*ys;
        let mut index = 0;
        for z in 0..size_z {
            let shifted_z =
                wrap(f64::from(volume.coordinate(2, z)) * xz_scale) + self.table.offsets[2];
            let cell_z = floor_i32(shifted_z);
            let dz = (shifted_z - f64::from(cell_z)) as f32;
            let fade_z = smoothstep(dz);
            for x in 0..size_x {
                let shifted_x =
                    wrap(f64::from(volume.coordinate(0, x)) * xz_scale) + self.table.offsets[0];
                let cell_x = floor_i32(shifted_x);
                let dx = (shifted_x - f64::from(cell_x)) as f32;
                let fade_x = smoothstep(dx);
                let p = i32::from(self.perm[(cell_x & 255) as usize]);
                let q = i32::from(self.perm[(cell_x.wrapping_add(1) & 255) as usize]);
                let mut previous_y = i32::MIN;
                let mut xz_dots = [0.0_f32; 8];
                let mut y_gradients = [0.0_f32; 8];
                let column = &mut buffer[index..index + size_y];
                let mut y = 0;
                while y < size_y {
                    let sample = &ys[y];
                    if previous_y != sample.cell {
                        let gradients = self.gradients_at(p, q, sample.cell, cell_z);
                        for i in 0..8 {
                            xz_dots[i] = gradients[i][0] * (dx - (i & 1) as f32)
                                + gradients[i][2] * (dz - ((i >> 2) & 1) as f32);
                            y_gradients[i] = gradients[i][1];
                        }
                        previous_y = sample.cell;
                    }
                    // Four Ys in one cell share the corners: side by side, LLVM computes them
                    // as vectors. Cell boundaries and the column's tail go one at a time.
                    if BATCH
                        && y + 4 <= size_y
                        && ys[y + 1..y + 4].iter().all(|s| s.cell == sample.cell)
                    {
                        add_four(
                            &mut column[y..y + 4],
                            &ys[y..y + 4],
                            xz_dots,
                            y_gradients,
                            fade_x,
                            fade_z,
                            amplitude,
                        );
                        y += 4;
                        continue;
                    }
                    let out = &mut column[y];
                    let dy = sample.offset;
                    let values = std::array::from_fn(|i| {
                        xz_dots[i] + y_gradients[i] * (dy - ((i >> 1) & 1) as f32)
                    });
                    *out += amplitude * lerp3([fade_x, sample.fade, fade_z], values);
                    y += 1;
                }
                index += size_y;
            }
        }
        Ok(())
    }
}

/// Y columns up to this height keep their per-Y samples on the stack.
const STACK_Y: usize = 64;

#[derive(Clone, Copy, Default)]
struct YSample {
    cell: i32,
    fade: f32,
    /// The Y offset inside the cell used for the corner values, after any fudge.
    offset: f32,
}

#[derive(Clone, Debug)]
pub struct SmearedPerlinNoise {
    base: PerlinNoise,
    fudge_y_scale: f64,
}

impl SmearedPerlinNoise {
    pub fn new(random: &mut impl RandomSource, fudge_y_scale: f64) -> Self {
        Self {
            base: PerlinNoise::new(random),
            fudge_y_scale,
        }
    }
    pub fn table(&self) -> &GradientTable {
        self.base.table()
    }
    pub fn range(&self) -> Result<[f32; 2], super::NanRange> {
        let bound = (self.fudge_y_scale.abs() + 2.0) as f32;
        if bound.is_nan() {
            Err(super::NanRange)
        } else {
            Ok([-bound, bound])
        }
    }
    pub fn get2(&self, x: f64, z: f64) -> f32 {
        self.get3(wrap(x), 0.0, wrap(z))
    }
    pub fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        let (cell, relative) = self.base.coordinates([x, y, z]);
        let fudge = compute_fudge_y(y, relative[1], self.fudge_y_scale);
        self.base.sample(
            cell,
            [
                relative[0] as f32,
                (relative[1] - fudge) as f32,
                relative[2] as f32,
            ],
            relative[1] as f32,
        )
    }
    // This inherited method samples the unsmeared field, just as vanilla does.
    pub fn noise_with_derivative(&self, x: f64, y: f64, z: f64, derivative: &mut [f32; 3]) -> f32 {
        self.base.noise_with_derivative(x, y, z, derivative)
    }
    pub fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) -> Result<(), DensityError> {
        self.base.add_volume::<true>(
            buffer,
            volume,
            xz_scale,
            y_scale,
            amplitude,
            Some(self.fudge_y_scale),
        )
    }
}

fn compute_fudge_y(input_y: f64, relative_y: f64, scale: f64) -> f64 {
    let limited_y = if input_y >= 0.0 && input_y < relative_y {
        input_y
    } else {
        relative_y
    };
    f64::from(floor_i32(limited_y / scale + f64::from(1e-7_f32))) * scale
}

/// Adds four Y samples of one cell, laid out lane by lane so LLVM vectorizes them (no intrinsics,
/// no target feature). Each lane does the one-at-a-time loop's operations in the same order.
#[inline]
fn add_four(
    out: &mut [f32],
    ys: &[YSample],
    dots: [f32; 8],
    gradients: [f32; 8],
    fade_x: f32,
    fade_z: f32,
    amplitude: f32,
) {
    let dy: [f32; 4] = std::array::from_fn(|lane| ys[lane].offset);
    let fy: [f32; 4] = std::array::from_fn(|lane| ys[lane].fade);
    let values: [[f32; 4]; 8] = std::array::from_fn(|i| {
        std::array::from_fn(|lane| dots[i] + gradients[i] * (dy[lane] - ((i >> 1) & 1) as f32))
    });
    for lane in 0..4 {
        out[lane] += amplitude * lerp3([fade_x, fy[lane], fade_z], values.map(|v| v[lane]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise::reference::{
        ref_perlin_get2, ref_perlin_get3, ref_smeared_get2, ref_smeared_get3,
    };
    use crate::random::LegacyRandom;

    /// The scalar fast paths (permutation tables, the Y=0 family) are bit-for-bit the
    /// pre-change `coordinates` + `gradients` + `corner_values` reference.
    #[test]
    fn scalar_fast_paths_match_the_reference() {
        let grid = [
            -257.25_f64,
            -256.5,
            -1.5,
            -0.0,
            0.0,
            0.5,
            1.5,
            255.5,
            256.25,
            511.5,
            -16_777_216.0,
            16_777_216.0,
        ];
        for seed in [0_i64, -1, 1234, i64::MIN] {
            let noise = PerlinNoise::new(&mut LegacyRandom::new(seed));
            for &x in &grid {
                for &z in &grid {
                    assert_eq!(
                        noise.get2(x, z).to_bits(),
                        ref_perlin_get2(noise.table(), x, z).to_bits(),
                        "get2 seed={seed} ({x:?}, {z:?})"
                    );
                    for &y in &grid {
                        assert_eq!(
                            noise.get3(x, y, z).to_bits(),
                            ref_perlin_get3(noise.table(), x, y, z).to_bits(),
                            "get3 seed={seed} ({x:?}, {y:?}, {z:?})"
                        );
                    }
                }
            }
            for scale in [0.125_f64, -0.25, 0.0] {
                let smeared = SmearedPerlinNoise::new(&mut LegacyRandom::new(seed), scale);
                for &x in &grid {
                    for &z in &grid {
                        assert_eq!(
                            smeared.get2(x, z).to_bits(),
                            ref_smeared_get2(smeared.table(), scale, x, z).to_bits(),
                            "smeared get2 seed={seed} scale={scale} ({x:?}, {z:?})"
                        );
                    }
                }
                for &x in &[0.25_f64, 1.5, -3.75, 255.5] {
                    for &y in &[-8.5_f64, -0.0, 0.0, 0.75, 12.25] {
                        for &z in &[0.25_f64, -1.5, 256.25] {
                            assert_eq!(
                                smeared.get3(x, y, z).to_bits(),
                                ref_smeared_get3(smeared.table(), scale, x, y, z).to_bits(),
                                "smeared get3 seed={seed} scale={scale} ({x:?}, {y:?}, {z:?})"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn batched_volume_is_bit_identical() {
        let scales = [
            0.0,
            -0.0,
            0.001,
            -0.001,
            0.125,
            -0.5,
            1.0,
            4.0,
            1e100,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ];
        let amplitudes = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::from_bits(1),
        ];
        for seed in [0, -1, 1234, i64::MIN] {
            let noise = PerlinNoise::new(&mut LegacyRandom::new(seed));
            for height in [1, 2, 3, 4, 5, 7, 8, 9, 31, 64, 65, 97] {
                for (case, &scale) in scales.iter().enumerate() {
                    let minimum = [
                        [-17, -64, 49],
                        [i32::MAX, i32::MIN, -16777216],
                        [16777216, 33554431, -33554432],
                    ][case % 3];
                    let volume = DensityVolume::new(
                        [2, height, 2],
                        minimum,
                        [4, if case % 3 == 0 { i32::MAX } else { 1 }, 16],
                    )
                    .unwrap();
                    for fudge in [None, Some(0.125), Some(-0.25), Some(0.0), Some(f64::NAN)] {
                        let initial = [0.0, -0.0, 1.5, f32::from_bits(1)];
                        let mut a: Vec<_> =
                            (0..volume.size() + 3).map(|i| initial[i % 4]).collect();
                        let mut b = a.clone();
                        let amplitude = amplitudes[case % amplitudes.len()];
                        noise
                            .add_volume::<false>(&mut a, volume, scale, scale, amplitude, fudge)
                            .unwrap();
                        noise
                            .add_volume::<true>(&mut b, volume, scale, scale, amplitude, fudge)
                            .unwrap();
                        for (i, (a, b)) in a.iter().zip(&b).enumerate() {
                            assert_eq!(
                                a.to_bits(),
                                b.to_bits(),
                                "seed={seed} height={height} scale={scale} fudge={fudge:?} index={i}"
                            );
                        }
                    }
                }
            }
        }
    }
}
