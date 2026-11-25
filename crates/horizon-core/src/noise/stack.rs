use super::{DensityError, DensityVolume, NanRange, PerlinNoise, SimplexNoise, SmearedPerlinNoise};
use std::any::Any;
use std::sync::Arc;

/// Immutable noise sampling. Implementations may override the default scalar
/// volume traversal when vanilla has a distinct bulk arithmetic path.
pub trait Noise: Any + std::fmt::Debug + Send + Sync {
    fn range(&self) -> Result<[f32; 2], NanRange>;
    fn get2(&self, x: f64, z: f64) -> f32;
    fn get3(&self, x: f64, y: f64, z: f64) -> f32;
    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        let [sx, sy, sz] = volume.sizes();
        let mut index = 0;
        for z in 0..sz {
            let z = f64::from(volume.coordinate(2, z)) * xz_scale;
            for x in 0..sx {
                let x = f64::from(volume.coordinate(0, x)) * xz_scale;
                for y in 0..sy {
                    let y = f64::from(volume.coordinate(1, y)) * y_scale;
                    buffer[index] += amplitude * self.get3(x, y, z);
                    index += 1;
                }
            }
        }
        Ok(())
    }
}

impl Noise for PerlinNoise {
    fn range(&self) -> Result<[f32; 2], NanRange> {
        Ok(PerlinNoise::range(self))
    }
    fn get2(&self, x: f64, z: f64) -> f32 {
        PerlinNoise::get2(self, x, z)
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        PerlinNoise::get3(self, x, y, z)
    }
    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) -> Result<(), DensityError> {
        PerlinNoise::add_to_volume(self, buffer, volume, xz_scale, y_scale, amplitude)
    }
}
impl Noise for SmearedPerlinNoise {
    fn range(&self) -> Result<[f32; 2], NanRange> {
        SmearedPerlinNoise::range(self)
    }
    fn get2(&self, x: f64, z: f64) -> f32 {
        SmearedPerlinNoise::get2(self, x, z)
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        SmearedPerlinNoise::get3(self, x, y, z)
    }
    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) -> Result<(), DensityError> {
        SmearedPerlinNoise::add_to_volume(self, buffer, volume, xz_scale, y_scale, amplitude)
    }
}
impl Noise for SimplexNoise {
    fn range(&self) -> Result<[f32; 2], NanRange> {
        Ok(SimplexNoise::range(self))
    }
    fn get2(&self, x: f64, z: f64) -> f32 {
        SimplexNoise::get2(self, x, z)
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        SimplexNoise::get3(self, x, y, z)
    }
}

#[derive(Clone, Debug)]
struct TypedLayer<T> {
    noise: Arc<T>,
    frequency: f64,
    amplitude: f32,
}

/// Homogeneous devirtualized layer storage for stacks whose layers are all a
/// single concrete `PerlinNoise`, `SmearedPerlinNoise`, or `SimplexNoise`.
///
/// Concrete layers share the existing `Arc<T>` instances from `layers` via
/// `Arc::downcast::<T>()` — zero table cloning and no duplicate sampler tables
/// alongside `layers` (only an `Arc` refcount bump per layer). Mixed stacks and
/// custom wrapper `Noise` implementations fall back to `FastLayers::Dynamic`.
#[derive(Clone, Debug)]
enum FastLayers {
    Perlin(Box<[TypedLayer<PerlinNoise>]>),
    SmearedPerlin(Box<[TypedLayer<SmearedPerlinNoise>]>),
    Simplex(Box<[TypedLayer<SimplexNoise>]>),
    Dynamic,
}

/// Downcasts every layer to concrete type `T` using non-overridable [`Any`]
/// type identity, sharing the existing [`Arc`] allocations with zero cloning.
fn try_downcast_layers<T: Any + Send + Sync>(
    layers: &[NoiseLayer],
) -> Option<Box<[TypedLayer<T>]>> {
    if !layers.iter().all(|layer| {
        let any: &dyn Any = layer.noise.as_ref();
        any.is::<T>()
    }) {
        return None;
    }
    layers
        .iter()
        .map(|layer| {
            let cloned: Arc<dyn Noise> = Arc::clone(&layer.noise);
            let any: Arc<dyn Any + Send + Sync> = cloned;
            let noise = any.downcast::<T>().ok()?;
            Some(TypedLayer {
                noise,
                frequency: layer.frequency,
                amplitude: layer.amplitude,
            })
        })
        .collect::<Option<Box<[_]>>>()
}

#[derive(Clone, Debug)]
pub struct NoiseLayer {
    pub noise: Arc<dyn Noise>,
    pub frequency: f64,
    pub amplitude: f32,
}

#[derive(Clone, Debug)]
pub struct NoiseStack {
    layers: Vec<NoiseLayer>,
    fast: FastLayers,
    y0_fast_path: bool,
    range: [f32; 2],
}

impl NoiseStack {
    pub fn builder() -> NoiseStackBuilder {
        NoiseStackBuilder::default()
    }
    pub fn layers(&self) -> &[NoiseLayer] {
        &self.layers
    }
    pub fn get_layer(&self, index: usize) -> Option<&Arc<dyn Noise>> {
        self.layers.get(index).map(|layer| &layer.noise)
    }
}
impl Noise for NoiseStack {
    fn range(&self) -> Result<[f32; 2], NanRange> {
        Ok(self.range)
    }
    fn get2(&self, x: f64, z: f64) -> f32 {
        let mut value = 0.0;
        match &self.fast {
            FastLayers::Perlin(layers) => {
                for layer in layers {
                    value += layer.amplitude
                        * layer.noise.get2(x * layer.frequency, z * layer.frequency);
                }
            }
            FastLayers::SmearedPerlin(layers) => {
                for layer in layers {
                    value += layer.amplitude
                        * layer.noise.get2(x * layer.frequency, z * layer.frequency);
                }
            }
            FastLayers::Simplex(layers) => {
                for layer in layers {
                    value += layer.amplitude
                        * layer.noise.get2(x * layer.frequency, z * layer.frequency);
                }
            }
            FastLayers::Dynamic => {
                for layer in &self.layers {
                    value += layer.amplitude
                        * layer.noise.get2(x * layer.frequency, z * layer.frequency);
                }
            }
        }
        value
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        let mut value = 0.0;
        match &self.fast {
            FastLayers::Perlin(layers) => {
                if y == 0.0 && self.y0_fast_path {
                    for layer in layers {
                        value += layer.amplitude
                            * layer
                                .noise
                                .get3_y0(x * layer.frequency, z * layer.frequency);
                    }
                } else {
                    for layer in layers {
                        value += layer.amplitude
                            * layer.noise.get3(
                                x * layer.frequency,
                                y * layer.frequency,
                                z * layer.frequency,
                            );
                    }
                }
            }
            FastLayers::SmearedPerlin(layers) => {
                for layer in layers {
                    value += layer.amplitude
                        * layer.noise.get3(
                            x * layer.frequency,
                            y * layer.frequency,
                            z * layer.frequency,
                        );
                }
            }
            FastLayers::Simplex(layers) => {
                for layer in layers {
                    value += layer.amplitude
                        * layer.noise.get3(
                            x * layer.frequency,
                            y * layer.frequency,
                            z * layer.frequency,
                        );
                }
            }
            FastLayers::Dynamic => {
                for layer in &self.layers {
                    value += layer.amplitude
                        * layer.noise.get3(
                            x * layer.frequency,
                            y * layer.frequency,
                            z * layer.frequency,
                        );
                }
            }
        }
        value
    }
    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        match &self.fast {
            FastLayers::Perlin(layers) => {
                for layer in layers {
                    layer.noise.add_to_volume(
                        buffer,
                        volume,
                        xz_scale * layer.frequency,
                        y_scale * layer.frequency,
                        amplitude * layer.amplitude,
                    )?;
                }
            }
            FastLayers::SmearedPerlin(layers) => {
                for layer in layers {
                    layer.noise.add_to_volume(
                        buffer,
                        volume,
                        xz_scale * layer.frequency,
                        y_scale * layer.frequency,
                        amplitude * layer.amplitude,
                    )?;
                }
            }
            FastLayers::Simplex(layers) => {
                for layer in layers {
                    layer.noise.add_to_volume(
                        buffer,
                        volume,
                        xz_scale * layer.frequency,
                        y_scale * layer.frequency,
                        amplitude * layer.amplitude,
                    )?;
                }
            }
            FastLayers::Dynamic => {
                for layer in &self.layers {
                    layer.noise.add_to_volume(
                        buffer,
                        volume,
                        xz_scale * layer.frequency,
                        y_scale * layer.frequency,
                        amplitude * layer.amplitude,
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct NoiseStackBuilder {
    layers: Vec<NoiseLayer>,
}
impl NoiseStackBuilder {
    pub fn add(&mut self, noise: Arc<dyn Noise>, frequency: f64, amplitude: f32) -> &mut Self {
        self.layers.push(NoiseLayer {
            noise,
            frequency,
            amplitude,
        });
        self
    }
    /// Flattening rescales each existing layer before subsequent sampling.
    pub fn add_stack(&mut self, stack: &NoiseStack, frequency: f64, amplitude: f32) -> &mut Self {
        self.layers
            .extend(stack.layers.iter().map(|layer| NoiseLayer {
                noise: Arc::clone(&layer.noise),
                frequency: layer.frequency * frequency,
                amplitude: layer.amplitude * amplitude,
            }));
        self
    }
    pub fn build(&self) -> Result<NoiseStack, NanRange> {
        let mut range = [0.0; 2];
        for layer in &self.layers {
            let bounds = layer.noise.range()?;
            if layer.amplitude.is_nan() {
                return Err(NanRange);
            }
            let scaled = if bounds[0].is_nan() || bounds[1].is_nan() {
                [f32::NAN; 2]
            } else {
                // Interval.mulBound defines zero times infinity as positive zero.
                let product = |v: f32| {
                    if v == 0.0 || layer.amplitude == 0.0 {
                        0.0
                    } else {
                        v * layer.amplitude
                    }
                };
                let a = product(bounds[0]);
                let b = product(bounds[1]);
                if a < b {
                    [a, b]
                } else if a > b {
                    [b, a]
                } else if a == 0.0 {
                    [
                        f32::from_bits(a.to_bits() | b.to_bits()),
                        f32::from_bits(a.to_bits() & b.to_bits()),
                    ]
                } else {
                    [a, b]
                }
            };
            range = [range[0] + scaled[0], range[1] + scaled[1]];
            if range[0].is_nan() || range[1].is_nan() {
                range = [f32::NAN; 2];
            }
            // Interval.of interns [0,1], including a supplied negative zero.
            if range[0] == 0.0 && range[1] == 1.0 {
                range[0] = 0.0;
            }
        }
        let fast = if self.layers.is_empty() {
            FastLayers::Perlin(Box::new([]))
        } else if let Some(layers) = try_downcast_layers::<PerlinNoise>(&self.layers) {
            FastLayers::Perlin(layers)
        } else if let Some(layers) = try_downcast_layers::<SmearedPerlinNoise>(&self.layers) {
            FastLayers::SmearedPerlin(layers)
        } else if let Some(layers) = try_downcast_layers::<SimplexNoise>(&self.layers) {
            FastLayers::Simplex(layers)
        } else {
            FastLayers::Dynamic
        };
        let y0_fast_path = match &fast {
            FastLayers::Perlin(layers) => layers.iter().all(|layer| {
                layer.frequency.is_finite()
                    && layer.noise.table().offsets()[1].to_bits() != (-0.0_f64).to_bits()
            }),
            _ => false,
        };
        Ok(NoiseStack {
            layers: self.layers.clone(),
            fast,
            y0_fast_path,
            range,
        })
    }
}
