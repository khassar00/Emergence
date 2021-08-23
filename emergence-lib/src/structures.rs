use bevy::prelude::*;

use crate::graphics::sprite_bundle_from_position;
use crate::id::ID;
use crate::organisms::{Composition, OrganismBundle};
use crate::position::Position;

#[derive(Bundle, Default)]
pub struct StructureBundle {
    structure: Structure,
    #[bundle]
    organism_bundle: OrganismBundle,
}

// TODO: replace with better defaults
#[derive(Clone, Default)]
pub struct Structure {
    upkeep_rate: f32,
    starting_mass: f32,
    despawn_mass: f32,
}

#[derive(Clone, Default)]
pub struct Plant {
    photosynthesis_rate: f32,
}

#[derive(Bundle, Default)]
pub struct PlantBundle {
    plant: Plant,
    #[bundle]
    structure_bundle: StructureBundle,
}

impl PlantBundle {
    pub fn new(position: Position, material: Handle<ColorMaterial>) -> Self {
        Self {
            structure_bundle: StructureBundle {
                organism_bundle: OrganismBundle {
                    sprite_bundle: sprite_bundle_from_position(position, material),
                    id: ID::Plant,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Clone, Default)]
pub struct Fungi;

#[derive(Bundle, Default)]
pub struct FungiBundle {
    fungi: Fungi,
    #[bundle]
    structure_bundle: StructureBundle,
}

impl FungiBundle {
    pub fn new(position: Position, material: Handle<ColorMaterial>) -> Self {
        Self {
            structure_bundle: StructureBundle {
                organism_bundle: OrganismBundle {
                    sprite_bundle: sprite_bundle_from_position(position, material),
                    id: ID::Fungus,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub struct StructuresPlugin;
impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(photosynthesize.system())
            .add_system(upkeep.system())
            .add_system(cleanup.system());
    }
}

fn photosynthesize(time: Res<Time>, mut query: Query<(&Plant, &mut Composition)>) {
    for (plant, mut comp) in query.iter_mut() {
        comp.mass += plant.photosynthesis_rate * time.delta_seconds() * comp.mass.powf(2.0 / 3.0);
    }
}

fn upkeep(time: Res<Time>, mut query: Query<(&Structure, &mut Composition)>) {
    for (structure, mut comp) in query.iter_mut() {
        comp.mass -= structure.upkeep_rate * time.delta_seconds() * comp.mass;
    }
}

fn cleanup(mut commands: Commands, query: Query<(&Structure, Entity, &Composition)>) {
    for (structure, ent, comp) in query.iter() {
        if comp.mass <= structure.despawn_mass {
            commands.entity(ent).despawn();
        }
    }
}