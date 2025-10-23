//! Tilemap collision extraction. Converts LDtk IntGrid layers into a hash-set of solid tiles that
//! the movement system queries. The data lives in a Bevy resource so it can be accessed by any
//! system without copying large structures.

use std::collections::HashSet;

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::level::{LevelAssets, LevelConfig};

/// Registers the collision map resource and rebuild system. Bevy keeps plugin state in its ECS
/// world; no manual allocation or freeing is necessary.
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionMap>().add_systems(
            PostUpdate,
            rebuild_collision_map
                .after(crate::level::sync_level_spatial)
                .in_set(CollisionSystems),
        );
    }
}

/// Marker system set so movement can depend on collision map freshness if required.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionSystems;

/// Runtime collision data. Stores the LDtk tile size, world origin, and a hash-set of solid cell
/// coordinates. The hash-set grants O(1) `is_solid` queries while remaining compact in memory.
#[derive(Resource, Default)]
pub struct CollisionMap {
    pub tile_size: Vec2,
    pub origin: Vec2,
    pub solids: HashSet<IVec2>,
    pub tile_values: std::collections::HashMap<IVec2, i32>,
}

impl CollisionMap {
    /// Clears the hash-set. Memory is retained by the `HashSet` allocation for reuse in the next
    /// rebuild, avoiding repeated heap allocations.
    pub fn clear(&mut self) {
        self.solids.clear();
        self.tile_values.clear();
    }

    /// Returns whether the given tile coordinate is flagged as solid.
    pub fn is_solid(&self, tile: IVec2) -> bool {
        self.solids.contains(&tile)
    }

    /// Returns the IntGrid value at the given tile coordinate, or None if no tile exists.
    pub fn get_tile_value(&self, tile: IVec2) -> Option<i32> {
        self.tile_values.get(&tile).copied()
    }
}

/// Regenerates the solid tile cache whenever LDtk emits level spawn/despawn events. The ECS query
/// iterates over freshly spawned `IntGridCell` entities, copying only the coordinates we care about
/// into the `HashSet`. All intermediate data is stack-allocated and dropped after the system runs.
fn rebuild_collision_map(
    mut events: EventReader<LevelEvent>,
    int_cells: Query<(&GridCoords, &IntGridCell, &Parent)>,
    config: Res<LevelConfig>,
    level_assets: Res<LevelAssets>,
    mut map: ResMut<CollisionMap>,
) {
    let mut needs_rebuild = false;
    let mut should_clear = false;

    for event in events.read() {
        match event {
            LevelEvent::Spawned(_) => needs_rebuild = true,
            LevelEvent::Despawned(_) => should_clear = true,
            _ => {}
        }
    }

    if should_clear {
        map.clear();
    }

    if !needs_rebuild {
        return;
    }

    map.tile_size = Vec2::splat(config.tile_size);
    map.origin = level_assets.level_origin.unwrap_or(Vec2::ZERO);
    map.solids.clear();
    map.tile_values.clear();

    let mut value_2_count = 0;
    for (coords, cell, _) in &int_cells {
        let tile_pos = IVec2::new(coords.x, coords.y);

        if cell.value > 0 {
            // Value 1 = solid collision block
            // Value 2 = non-solid trigger (for level transitions)
            if cell.value == 1 {
                map.solids.insert(tile_pos);
            }

            // Store all non-zero values in the tile_values map
            map.tile_values.insert(tile_pos, cell.value);

            if cell.value == 2 {
                value_2_count += 1;
                info!("Found value 2 tile (trigger) at grid coords: {:?}", tile_pos);
            }
        }
    }

    info!("Collision map rebuilt: {} solid tiles, {} trigger tiles", map.solids.len(), value_2_count);

    if map.solids.is_empty() {
        warn!(
            "Collision map is empty. Ensure your LDtk IntGrid layer marks solid tiles with a non-zero value."
        );
    }
}
