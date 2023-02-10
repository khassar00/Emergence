//! Tiles can be selected, serving as a building block for clipboard, inspection and zoning operations.

use bevy::{prelude::*, utils::HashSet};
use emergence_macros::IterableEnum;
use hexx::shapes::hexagon;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::terrain::TerrainHandles, simulation::geometry::TilePos, terrain::Terrain,
};

use crate as emergence_lib;

use super::{cursor::CursorPos, InteractionSystem, PlayerAction};

/// Code and data for selecting groups of tiles
pub(super) struct TileSelectionPlugin;

impl Plugin for TileSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTiles>()
            .init_resource::<AreaSelection>()
            .init_resource::<LineSelection>()
            .add_system(
                select_tiles
                    .label(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            )
            .add_system(
                display_tile_interactions
                    .after(InteractionSystem::SelectTiles)
                    .after(InteractionSystem::ComputeCursorPos),
            );
    }
}

/// The set of tiles that is currently selected
#[derive(Resource, Debug, Default, Clone)]
pub struct SelectedTiles {
    /// Actively selected tiles
    selected: HashSet<TilePos>,
    /// Tiles that are hovered over
    hovered: HashSet<TilePos>,
}

impl SelectedTiles {
    /// Selects a single tile
    fn add_tile(&mut self, tile_pos: TilePos) {
        self.selected.insert(tile_pos);
    }

    /// Deselects a single tile
    fn remove_tile(&mut self, tile_pos: TilePos) {
        self.selected.remove(&tile_pos);
    }

    /// Selects a hexagon of tiles.
    fn select_hexagon(&mut self, center: TilePos, radius: u32, select: bool) {
        let hex_coord = hexagon(center.hex, radius);

        for hex in hex_coord {
            let target_pos = TilePos { hex };
            // Selection may have overflowed map
            match select {
                true => self.add_tile(target_pos),
                false => self.remove_tile(target_pos),
            }
        }
    }

    /// Clears the set of selected tiles.
    fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// The set of currently selected tiles.
    pub(super) fn selection(&self) -> &HashSet<TilePos> {
        &self.selected
    }

    /// Are any tiles selected?
    pub(super) fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Is the given tile in the selection?
    #[cfg(test)]
    fn contains_tile(&self, tile_pos: TilePos) -> bool {
        self.selected.contains(&tile_pos)
    }
}

/// How a given object is being interacted with by the player.
#[derive(PartialEq, Eq, Hash, Clone, Debug, IterableEnum)]
pub(crate) enum ObjectInteraction {
    /// Currently in the selection.
    Selected,
    /// Hovered over with the cursor.
    Hovered,
    /// Hovered over and simultaneously selected.
    ///
    /// This exists to allow easy visual distinction of this state,
    /// and should include visual elements of both.
    ///
    // TODO: this is silly and probably shouldn't exist, but we're using colors for everything for now so...
    // Tracked in https://github.com/Leafwing-Studios/Emergence/issues/263
    HoveredAndSelected,
}

impl ObjectInteraction {
    /// The material used by objects that are being interacted with.
    pub(crate) fn material(&self) -> StandardMaterial {
        let base_color = match self {
            ObjectInteraction::Selected => Color::DARK_GREEN,
            ObjectInteraction::Hovered => Color::YELLOW,
            ObjectInteraction::HoveredAndSelected => Color::YELLOW_GREEN,
        };

        StandardMaterial {
            base_color,
            ..Default::default()
        }
    }
}

/// The state needed by [`PlayerAction::Area`].
#[derive(Resource)]
struct AreaSelection {
    /// The central tile, where the area selection began.
    center: Option<TilePos>,
    /// The radius of the selection.
    radius: u32,
    /// The tiles selected at the start of this action.
    initial_selection: Option<SelectedTiles>,
}

impl AreaSelection {
    /// Set things up to start a line selection action.
    fn begin(&mut self, selected_tiles: &SelectedTiles, cursor_pos: TilePos) {
        self.center = Some(cursor_pos);
        self.initial_selection = Some(selected_tiles.clone());
    }

    /// Clean things up to conclude a line selection action.
    fn finish(&mut self) {
        self.center = None;
        self.initial_selection = None;
    }
}

impl Default for AreaSelection {
    fn default() -> Self {
        AreaSelection {
            center: None,
            radius: 1,
            initial_selection: None,
        }
    }
}

/// The state needed by [`PlayerAction::Line`].
#[derive(Resource, Default)]
struct LineSelection {
    /// The starting tile, where the line selection began.
    start: Option<TilePos>,
    /// The tiles selected at the start of this action.
    initial_selection: Option<SelectedTiles>,
}

impl LineSelection {
    /// Set things up to start a line selection action.
    fn begin(&mut self, selected_tiles: &SelectedTiles, cursor_pos: TilePos) {
        self.start = Some(cursor_pos);
        self.initial_selection = Some(selected_tiles.clone());
    }

    /// Clean things up to conclude a line selection action.
    fn finish(&mut self) {
        self.start = None;
        self.initial_selection = None;
    }

    /// Computes the set of hexagons between `self.start` and `end`, with a thickness determnind by `radius`.
    fn draw_line(&self, end: TilePos, radius: u32) -> HashSet<TilePos> {
        let start = self.start.unwrap();
        let line = start.line_to(end.hex);
        let mut tiles = HashSet::<TilePos>::new();

        for line_hex in line {
            let hexagon = hexagon(line_hex, radius);
            for hex in hexagon {
                tiles.insert(TilePos { hex });
            }
        }
        tiles
    }
}

/// Integrates user input into tile selection actions to let other systems handle what happens to a selected tile
#[allow(clippy::too_many_arguments)]
fn select_tiles(
    cursor: Res<CursorPos>,
    mut selected_tiles: ResMut<SelectedTiles>,
    actions: Res<ActionState<PlayerAction>>,
    mut area_selection: ResMut<AreaSelection>,
    mut line_selection: ResMut<LineSelection>,
) {
    if let Some(cursor_pos) = cursor.maybe_tile_pos() {
        let select = actions.pressed(PlayerAction::Select);
        let deselect = actions.pressed(PlayerAction::Deselect);

        let multiple = actions.pressed(PlayerAction::Multiple);
        let area = actions.pressed(PlayerAction::Area);
        let line = actions.pressed(PlayerAction::Line);
        let simple_area = area & !multiple & !line;
        let simple_deselect = deselect & !area & !multiple & !line;

        // Cache the starting state to make selections reversible
        if simple_area & area_selection.initial_selection.is_none() {
            area_selection.begin(&selected_tiles, cursor_pos);
        }

        if line & line_selection.initial_selection.is_none() {
            line_selection.begin(&selected_tiles, cursor_pos);
        }

        // Clean up state from area and line selections
        if !simple_area {
            area_selection.finish();
        }

        if !line {
            line_selection.finish();
        }

        // Compute the center and radius
        let (center, radius) = if area {
            let center = if !simple_area {
                cursor_pos
            } else {
                area_selection.center.unwrap()
            };

            if simple_area {
                area_selection.radius = cursor_pos.unsigned_distance_to(center.hex);
            }

            (center, area_selection.radius)
        } else {
            (cursor_pos, 0)
        };

        // Record which tiles should have the "hovered" effect
        selected_tiles.hovered.clear();
        if simple_area {
            selected_tiles.hovered.insert(center);
            let ring = center.hex.ring(radius);
            for hex in ring {
                selected_tiles.hovered.insert(TilePos { hex });
            }
        } else if line {
            let line_hexes = line_selection.draw_line(cursor_pos, radius);
            selected_tiles.hovered.extend(line_hexes);
        } else {
            selected_tiles.hovered.insert(cursor_pos);
        }

        // Don't attempt to handle conflicting inputs.
        if select & deselect {
            return;
        }

        // Clear the selection
        if simple_deselect | (select & !multiple) {
            selected_tiles.clear_selection()
        }

        // Actually select tiles
        if line {
            if actions.just_released(PlayerAction::Select) {
                let line_hexes = line_selection.draw_line(cursor_pos, radius);
                selected_tiles.selected.extend(line_hexes);
                line_selection.start = Some(cursor_pos);
            } else if actions.just_released(PlayerAction::Deselect) {
                let line_hexes = line_selection.draw_line(cursor_pos, radius);
                for tile_pos in line_hexes {
                    selected_tiles.selected.remove(&tile_pos);
                }
                line_selection.start = Some(cursor_pos);
            }
        } else {
            if select {
                selected_tiles.select_hexagon(center, radius, true);
            }

            if deselect {
                selected_tiles.select_hexagon(center, radius, false);
            }
        }
    }
}

/// Shows which tiles are being hovered and selected.
fn display_tile_interactions(
    selected_tiles: Res<SelectedTiles>,
    mut terrain_query: Query<(&mut Handle<StandardMaterial>, &Terrain, &TilePos)>,
    materials: Res<TerrainHandles>,
) {
    if selected_tiles.is_changed() {
        // PERF: We should probably avoid a linear scan over all tiles here
        for (mut material, terrain, &tile_pos) in terrain_query.iter_mut() {
            let hovered = selected_tiles.hovered.contains(&tile_pos);
            let selected = selected_tiles.selected.contains(&tile_pos);

            *material = materials.get_material(terrain, hovered, selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectedTiles;
    use crate::simulation::geometry::TilePos;

    #[test]
    fn simple_selection() {
        let mut selected_tiles = SelectedTiles::default();
        let tile_pos = TilePos::default();

        selected_tiles.add_tile(tile_pos);
        assert!(selected_tiles.contains_tile(tile_pos));
        assert!(!selected_tiles.is_empty());
        assert_eq!(selected_tiles.selected.len(), 1);

        selected_tiles.remove_tile(tile_pos);
        assert!(!selected_tiles.contains_tile(tile_pos));
        assert!(selected_tiles.is_empty());
        assert_eq!(selected_tiles.selected.len(), 0);
    }

    #[test]
    fn multi_select() {
        let mut selected_tiles = SelectedTiles::default();

        selected_tiles.add_tile(TilePos::new(1, 1));
        // Intentionally doubled
        selected_tiles.add_tile(TilePos::new(1, 1));
        selected_tiles.add_tile(TilePos::new(2, 2));
        selected_tiles.add_tile(TilePos::new(3, 3));

        assert_eq!(selected_tiles.selected.len(), 3);
    }

    #[test]
    fn clear_selection() {
        let mut selected_tiles = SelectedTiles::default();
        selected_tiles.add_tile(TilePos::new(1, 1));
        selected_tiles.add_tile(TilePos::new(2, 2));
        selected_tiles.add_tile(TilePos::new(3, 3));

        assert_eq!(selected_tiles.selected.len(), 3);
        selected_tiles.clear_selection();
        assert_eq!(selected_tiles.selected.len(), 0);
    }
}
