//! Instructions to craft items.

use std::{fmt::Display, time::Duration};

use crate::{
    asset_management::manifest::{Id, ItemManifest, Recipe},
    organisms::energy::Energy,
    structures::crafting::{CraftTimer, InputInventory, OutputInventory},
};

use super::{inventory::Inventory, ItemCount};

// TODO: these should be read from disc
impl Id<Recipe> {
    /// The ID of the recipe for the leaf production of acacia plants.
    pub fn acacia_leaf_production() -> Self {
        Self::new("acacia_leaf_production")
    }

    /// The ID of the recipe for mushroom production of leuco mushrooms.
    pub fn leuco_chunk_production() -> Self {
        Self::new("leuco_chunk_production")
    }

    /// The ID of the recipe to make more ant eggs from leuco mushrooms.
    pub fn ant_egg_production() -> Self {
        Self::new("ant_egg_production")
    }

    /// The ID of the recipe to hatch ant eggs into adult ants.
    pub fn hatch_ants() -> Self {
        Self::new("hatch_ants")
    }
}

/// A recipe to turn a set of items into different items.
#[derive(Debug, Clone)]
pub(crate) struct RecipeData {
    /// The inputs needed to craft the recipe.
    inputs: Vec<ItemCount>,

    /// The outputs generated by crafting.
    outputs: Vec<ItemCount>,

    /// The time needed to craft the recipe.
    craft_time: Duration,

    /// The amount of [`Energy`] produced by making this recipe, if any.
    ///
    /// This is only relevant to living structures.
    energy: Option<Energy>,
}

impl RecipeData {
    /// Create a new recipe with the given inputs, outputs and craft time.
    pub(crate) fn new(
        inputs: Vec<ItemCount>,
        outputs: Vec<ItemCount>,
        craft_time: Duration,
        energy: Option<Energy>,
    ) -> Self {
        Self {
            inputs,
            outputs,
            craft_time,
            energy,
        }
    }

    /// The inputs needed to craft the recipe.
    pub(crate) fn inputs(&self) -> &Vec<ItemCount> {
        &self.inputs
    }

    /// The outputs generated by crafting.
    pub(crate) fn outputs(&self) -> &Vec<ItemCount> {
        &self.outputs
    }

    /// The time needed to craft the recipe.
    pub(crate) fn craft_time(&self) -> Duration {
        self.craft_time
    }

    /// The timer used in this recipe.
    pub(crate) fn craft_timer(&self) -> CraftTimer {
        CraftTimer::new(self.craft_time)
    }

    /// An inventory with empty slots for all of the inputs of this recipe.
    pub(crate) fn input_inventory(&self, item_manifest: &ItemManifest) -> InputInventory {
        let mut inventory = Inventory::new(self.inputs.len());
        for item_count in &self.inputs {
            inventory.add_empty_slot(item_count.item_id, item_manifest);
        }
        InputInventory { inventory }
    }

    /// An inventory with empty slots for all of the outputs of this recipe.
    pub(crate) fn output_inventory(&self, item_manifest: &ItemManifest) -> OutputInventory {
        let mut inventory = Inventory::new(self.outputs.len());
        for item_count in &self.outputs {
            inventory.add_empty_slot(item_count.item_id, item_manifest);
        }
        OutputInventory { inventory }
    }

    /// The amount of energy produced by crafting the recipe, if any.
    pub(crate) fn energy(&self) -> &Option<Energy> {
        &self.energy
    }
}

// TODO: Remove this once we load recipes from asset files
impl RecipeData {
    /// An acacia plant producing leaves.
    pub(crate) fn acacia_leaf_production() -> Self {
        RecipeData::new(
            Vec::new(),
            vec![ItemCount::one(Id::acacia_leaf())],
            Duration::from_secs(3),
            Some(Energy(20.)),
        )
    }

    /// A leuco mushroom processing acacia leaves
    pub(crate) fn leuco_chunk_production() -> Self {
        RecipeData::new(
            vec![ItemCount::one(Id::acacia_leaf())],
            vec![ItemCount::one(Id::leuco_chunk())],
            Duration::from_secs(2),
            Some(Energy(40.)),
        )
    }

    /// An ant hive producing eggs.
    pub(crate) fn ant_egg_production() -> Self {
        RecipeData::new(
            vec![ItemCount::one(Id::leuco_chunk())],
            vec![ItemCount::one(Id::ant_egg())],
            Duration::from_secs(5),
            None,
        )
    }

    /// An ant hive producing eggs.
    pub(crate) fn hatch_ants() -> Self {
        RecipeData::new(
            vec![ItemCount::one(Id::ant_egg())],
            vec![],
            Duration::from_secs(2),
            None,
        )
    }
}

impl Display for RecipeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input_strings: Vec<String> =
            self.inputs.iter().map(|input| format!("{input}")).collect();
        let input_str = input_strings.join(", ");

        let output_strings: Vec<String> = self
            .outputs
            .iter()
            .map(|output| format!("{output}"))
            .collect();
        let output_str = output_strings.join(", ");

        let duration_str = format!("{:.2}", self.craft_time().as_secs_f32());

        write!(f, "[{input_str}] -> [{output_str}] | {duration_str} s")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_display_inputs_outputs_craft_time() {
        let recipe = RecipeData {
            inputs: Vec::new(),
            outputs: vec![ItemCount::one(Id::acacia_leaf())],
            craft_time: Duration::from_secs(1),
            energy: Some(Energy(20.)),
        };

        assert_eq!(format!("{recipe}"), "[] -> [acacia_leaf (1)] | 1.00 s")
    }
}
