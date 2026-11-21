//! `menu_result_take` through a real `MenuPart` on a server's guest: a take from a mod menu's
//! result slot reaches the part after the block entity holds what the click left.

use horizon_client::{
    chunk::Chunk,
    containers::{BlockEntity, ModBlockEntity, ModMenuType},
    items::ItemStack,
    lan::menu::{Slots, asks_still_valid},
};
use horizon_protocol::packets::play::{ContainerClick, Serverbound};
use mod_api::{Hooks, MenuPart, MenuResultTakeEvent, ModEntry, Mods, Registrar, Tag};
use std::sync::{Arc, Mutex};

/// Each take the part heard: the slot, the count taken, and the count the block entity's
/// result slot held when the part read it.
static HEARD: Mutex<Vec<(u32, u32, i64)>> = Mutex::new(Vec::new());

struct Machine;
#[mod_api::horizon_part]
impl MenuPart for Machine {
    fn menu_result_take(
        &self,
        host: &mut dyn mod_api::ServerHost,
        event: &mut MenuResultTakeEvent<'_>,
    ) {
        let level = host.level(host.dimension()).unwrap();
        let data = host
            .block_entity(level, event.pos)
            .unwrap()
            .data(&*host)
            .unwrap();
        let left = match data.get(&"Items".into()) {
            Some(Tag::List(items)) => items
                .iter()
                .filter_map(|item| match item {
                    Tag::Compound(item) => Some(item),
                    _ => None,
                })
                .filter(|item| matches!(item.get(&"Slot".into()), Some(Tag::Byte(1))))
                .map(|item| match item.get(&"count".into()) {
                    Some(Tag::Int(n)) => i64::from(*n),
                    _ => 1,
                })
                .sum(),
            _ => 0,
        };
        HEARD
            .lock()
            .unwrap()
            .push((event.slot, event.stack.count(), left));
    }
}
struct Mod;
#[mod_api::__subscriptions]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &mod_api::RegistrationContext) {
        r.menu_part("test:press", Machine);
    }
}

#[test]
#[ignore = "daily"]
fn a_guests_take_reaches_the_part_after_the_machine_holds_what_it_left() {
    let mods: Mods = vec![ModEntry::new("press", Box::new(Mod))].into();
    crate::game::api_values::install_parts(&mods).unwrap();
    crate::entry::install_client_hooks();
    let press: &'static ModMenuType = Box::leak(Box::new(ModMenuType {
        id: "test:press",
        slots: 2,
        data_slots: 0,
        results: vec![1],
        crafting: None,
        links: None,
    }));
    mod_platform::testing::with_mods(mods, || {
        super::super::native_guest_turn_tests::with_guests(|game| {
            game.world
                .chunks
                .insert((0, 0), Arc::new(Chunk::empty(0, 0)));
            let eye = game.lan.as_ref().unwrap().hub.guests[0].position;
            let pos = (eye[0] as i32, eye[1] as i32, eye[2] as i32 + 2);
            let state = game
                .pack
                .assets
                .registry
                .default_state("minecraft:barrel")
                .unwrap();
            game.world.set(pos.0, pos.1, pos.2, state).unwrap();
            let mut entity = ModBlockEntity::new("test:press".into(), Default::default());
            entity.items = vec![None, Some(ItemStack::new("minecraft:iron_ingot", 3))];
            game.world
                .block_entities
                .insert(pos, BlockEntity::Mod(Box::new(entity)));
            let host = game.lan.as_mut().unwrap();
            host.hub.open_container(
                12,
                "minecraft:generic_9x3",
                Tag::String("press".into()),
                pos,
                Slots::Block(pos),
                vec![None; 2],
                None,
            );
            host.hub.guests.update(0, |guest| {
                let menu = guest.menu.as_mut().unwrap();
                menu.registry_id = "test:press".into();
                menu.mod_type = Some(press);
                menu.asks_still_valid = asks_still_valid(menu.mod_type, &menu.registry_id);
            });
            let window = game.lan.as_ref().unwrap().hub.guests[0]
                .menu
                .as_ref()
                .unwrap()
                .window;
            HEARD.lock().unwrap().clear();
            // Pick up the whole result stack.
            game.lan_guest_packet(
                12,
                Serverbound::ContainerClick(ContainerClick {
                    container: window,
                    state: 0,
                    slot: 1,
                    button: 0,
                    input: 0,
                    changed: Vec::new(),
                    carried: None,
                }),
            );
            assert_eq!(
                *HEARD.lock().unwrap(),
                vec![(1, 3, 0)],
                "the part heard the take of 3 with the result slot already empty"
            );
            let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&pos) else {
                panic!("the press is gone");
            };
            assert_eq!(entity.items[1], None);
        });
    });
}
