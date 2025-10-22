use std::collections::HashSet;

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::level::{LevelAssets, LevelConfig};

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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionSystems;

#[derive(Resource, Default)]
pub struct CollisionMap {
    pub tile_size: Vec2,
    pub origin: Vec2,
    pub solids: HashSet<IVec2>,
}

impl CollisionMap {
    pub fn clear(&mut self) {
        self.solids.clear();
    }

    pub fn is_solid(&self, tile: IVec2) -> bool {
        self.solids.contains(&tile)
    }
}

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
            LevelEvent::Spawned(_) => {
                needs_rebuild = true;
            }
            LevelEvent::Despawned(_) => {
                should_clear = true;
            }
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

    for (coords, cell, _) in &int_cells {
        if cell.value <= 0 {
            continue;
        }

        map.solids.insert(IVec2::new(coords.x, coords.y));
    }

    if map.solids.is_empty() {
        warn!(
            "Collision map is empty. Ensure your LDtk IntGrid layer marks solid tiles with a non-zero value."
        );
    }
}
