//! The global `dispenser_acts` hook from a real dispenser: a mod's answer for any item wins over
//! the faced block's and the item's parts, and no answer leaves vanilla's behaviour alone.
use crate::game::Game;
use horizon_client::{chunk::Chunk, items::ItemStack};
use horizon_core::geometry::Direction;
use mod_api::*;
use std::sync::Arc;

const DISPENSER: [i32; 3] = [5, 64, 5];

/// Empties the slot of any stack of sticks and answers nothing for other items.
struct Everything;
#[mod_api::__subscriptions]
impl Hooks for Everything {
    fn dispenser_acts(
        &self,
        _: &mut dyn ServerHost,
        event: &mut DispenserActsEvent<'_>,
    ) -> Option<Dispensed> {
        (mod_api::__private::item_key_id(event.stack.item()) == Some("minecraft:stick")
            && event.facing == Direction::East)
            .then_some((None, 1001))
    }
}

fn dispense(item: &str, count: u32) -> Option<ItemStack> {
    let mods: Mods = vec![ModEntry::new("everything", Box::new(Everything))].into();
    mod_platform::testing::with_mods(mods, || {
        let mut result = None;
        crate::game::native_entity_refs::tests::with_game(|game: &mut Game<'_>| {
            game.options.dedicated = Some(Vec::new());
            game.world
                .chunks
                .insert((0, 0), Arc::new(Chunk::empty(0, 0)));
            game.native_lifecycle
                .begin_open(mod_api::Authority::IntegratedServer);
            let at = (DISPENSER[0], DISPENSER[1], DISPENSER[2]);
            result = game
                .dispense_item(at, Direction::East, ItemStack::new(item, count))
                .unwrap();
        });
        result
    })
}

#[test]
#[ignore = "daily"]
fn a_mods_global_answer_replaces_vanillas_dispense_and_no_answer_keeps_it() {
    // The mod empties the slot of an item it claims, with nothing dropped.
    assert_eq!(dispense("minecraft:stick", 3), None);
    // An item it leaves alone is dispensed one at a time, as before.
    let left = dispense("minecraft:cobblestone", 3).unwrap();
    assert_eq!(left.count, 2);
}
