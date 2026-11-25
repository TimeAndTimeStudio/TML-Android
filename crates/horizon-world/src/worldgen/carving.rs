//! Apply ordered carving-mask intervals to pre-lighting terrain.
use super::{
    material_condition::{
        CompiledSurfaceRule, MaterialConditions, MaterialDensitySources, MaterialRule,
        MaterialSurfaceQueries,
    },
    material_stage::{SurfaceError, TopMaterialInputs, top_material_cell},
    terrain::{TerrainAquifer, TerrainBlockStates, TerrainChunk, TerrainError},
};
use crate::{
    heightmap::HeightmapState,
    registry::{BlockRegistry, BlockStateId},
};
use horizon_core::{
    carving::CarvingMask, density::SamplerContext, noise::DensityVolume, position::BlockPos,
    random::RandomSource,
};
use std::collections::BTreeSet;

/// Per-state predicates resolved once from the world's block registry and
/// `minecraft:uncarvable` tag. Keep these bound to the same registry as the chunk.
pub struct CarvingBlocks {
    flags: Vec<u8>,
}
impl CarvingBlocks {
    pub fn from_registry(
        registry: &BlockRegistry,
        uncarvable: &BTreeSet<String>,
    ) -> Result<Self, TerrainError> {
        for name in uncarvable.iter().map(String::as_str).chain([
            "minecraft:grass_block",
            "minecraft:mycelium",
            "minecraft:dirt",
        ]) {
            if registry.default_state(name).is_none() {
                return Err(TerrainError::InvalidState);
            }
        }
        let flags = (0..registry.len())
            .map(|index| {
                let name = &registry.state(BlockStateId(index as u32)).unwrap().name;
                u8::from(uncarvable.contains(name))
                    | (u8::from(matches!(
                        name.as_str(),
                        "minecraft:grass_block" | "minecraft:mycelium"
                    )) << 1)
                    | (u8::from(name == "minecraft:dirt") << 2)
            })
            .collect();
        Ok(Self { flags })
    }
    fn flags(&self, state: BlockStateId) -> Result<u8, TerrainError> {
        self.flags
            .get(state.0 as usize)
            .copied()
            .ok_or(TerrainError::InvalidState)
    }
}

/// Visit mask runs in packed order and each interval top to bottom. The filter
/// receives local X/Z and absolute Y before any block or aquifer lookup. A run
/// remembers grass/mycelium even if its aquifer returns no replacement. Exposed
/// dirt below subsequent carved cells is restored through `top_material`.
/// Earlier writes, aquifer state and fluid postprocessing survive later errors.
pub fn apply_carving_mask<R: RandomSource, S: MaterialSurfaceQueries, B: TerrainBlockStates>(
    chunk: &mut TerrainChunk<'_, B>,
    mask: &CarvingMask,
    rule: MaterialRule,
    inputs: TopMaterialInputs<'_, R, S>,
    aquifer: &mut (impl TerrainAquifer + ?Sized),
    blocks: &CarvingBlocks,
    filter: impl FnMut([i32; 3]) -> bool,
) -> Result<(), SurfaceError>
where
    R::Positional: Clone,
{
    apply_carving_mask_shared(
        chunk, mask, &rule, &mut None, inputs, aquifer, blocks, filter,
    )
}
/// `apply_carving_mask` with the rule compiled once for the chunks `shared`
/// serves (`MaterialConditions::add_shared_rule`).
pub fn apply_carving_mask_shared<
    R: RandomSource,
    S: MaterialSurfaceQueries,
    B: TerrainBlockStates,
>(
    chunk: &mut TerrainChunk<'_, B>,
    mask: &CarvingMask,
    rule: &MaterialRule,
    shared: &mut Option<CompiledSurfaceRule<R>>,
    inputs: TopMaterialInputs<'_, R, S>,
    aquifer: &mut (impl TerrainAquifer + ?Sized),
    blocks: &CarvingBlocks,
    mut filter: impl FnMut([i32; 3]) -> bool,
) -> Result<(), SurfaceError>
where
    R::Positional: Clone,
{
    if mask.is_empty() {
        return Ok(());
    }
    // Most carved columns never expose dirt below grass/mycelium. When `shared`
    // is already compiled and has no eager density bindings, defer constructing
    // the per-cell top-material worker until an exposed dirt block actually
    // needs repair.
    //
    // Deferring in that case cannot hide a rule-compile, noise-binding, or
    // density-sampling error: `CompiledSurfaceRule` is only populated by
    // `MaterialConditions::add_shared_rule` after `CompiledRule::compile`
    // validates the rule and resolves every noise, and once populated
    // `add_shared_rule` reuses the already-bound rule and noise handles
    // directly without re-compiling `rule` or calling `bind_noise`. When
    // `shared` is `None` or has eager density bindings, initialize the worker
    // up front at `[0, 0, 0]` before any terrain write so invalid rules and
    // eager origin sampling errors still fail before mutating the chunk.
    let TopMaterialInputs {
        generation,
        root,
        surface,
        preliminary_surface,
        density_context,
    } = inputs;
    let defer_worker = shared
        .as_ref()
        .is_some_and(|compiled| !compiled.has_eager_density_bindings());
    let mut top_worker = None;
    let mut top_inputs = if defer_worker {
        Some((generation, root, surface, preliminary_surface))
    } else {
        let volume =
            DensityVolume::new([1; 3], [0; 3], [1; 3]).map_err(|_| SurfaceError::InvalidVolume)?;
        let sources = MaterialDensitySources::new(
            surface,
            volume,
            preliminary_surface,
            SamplerContext::default(),
        )?;
        let mut worker =
            MaterialConditions::<R, _>::with_possible_biomes(generation, root, sources, None);
        let rule_id = worker.add_shared_rule(rule, shared)?;
        top_worker = Some((worker, rule_id));
        None
    };
    let cx = chunk.position().x.wrapping_shl(4);
    let cz = chunk.position().z.wrapping_shl(4);
    for column in mask.columns() {
        let mut found_surface = false;
        let x = cx.wrapping_add(column.x);
        let z = cz.wrapping_add(column.z);
        for y in (column.min_y..=column.max_y).rev() {
            if !filter([column.x, y, column.z]) {
                continue;
            }
            let flags = blocks.flags(chunk.block(x, y, z))?;
            if flags & 1 != 0 {
                continue;
            }
            found_surface |= flags & 2 != 0;
            let Some(state) = aquifer
                .compute_substance(density_context, [x, y, z], 0.)
                .map_err(TerrainError::from)?
            else {
                continue;
            };
            chunk.set_block_worldgen(x, y, z, state)?;
            if aquifer.should_schedule_fluid_update() && chunk.block_properties(state).has_fluid() {
                chunk.mark_postprocessing(x, y, z);
            }
            let below = y.wrapping_sub(1);
            if found_surface && blocks.flags(chunk.block(x, below, z))? & 4 != 0 {
                let submerged = chunk.block_properties(state).has_fluid();
                let (worker, rule_id) = match &mut top_worker {
                    Some((worker, rule_id)) => {
                        worker.retarget_surface_densities([x, below, z])?;
                        (worker, *rule_id)
                    }
                    None => {
                        let (generation, root, surface, preliminary_surface) =
                            top_inputs.take().expect("top material inputs");
                        let volume = DensityVolume::new([1; 3], [x, below, z], [1; 3])
                            .map_err(|_| SurfaceError::InvalidVolume)?;
                        let sources = MaterialDensitySources::new(
                            surface,
                            volume,
                            preliminary_surface,
                            SamplerContext::default(),
                        )?;
                        let mut worker = MaterialConditions::<R, _>::with_possible_biomes(
                            generation, root, sources, None,
                        );
                        let rule_id = worker.add_shared_rule(rule, shared)?;
                        let (worker, rule_id) = top_worker.insert((worker, rule_id));
                        (worker, *rule_id)
                    }
                };
                let replacement = top_material_cell(
                    worker,
                    chunk,
                    rule_id,
                    BlockPos::new(x, below, z),
                    submerged,
                )?;
                if let Some(replacement) = replacement {
                    chunk.set_block_worldgen(x, below, z, replacement)?;
                    if chunk.block_properties(replacement).has_fluid() {
                        chunk.mark_postprocessing(x, below, z);
                    }
                }
            }
        }
    }
    Ok(())
}
