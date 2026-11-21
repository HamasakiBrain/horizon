use super::Game;
use horizon_client::containers::ModBlockEntity;
use horizon_core::position::BlockPos;

impl Game<'_> {
    /// A callback removed or replaced its holder. Keep that mutation, and drop
    /// only the original container's contents, never the rejected proposal.
    /// A block whose mod keeps its container's stacks (`block_container_keeps_contents`)
    /// scatters them here too: the holder was out of the level for the call, so the
    /// mod's `block_drops` saw an empty placeholder and could not carry them.
    pub(super) fn drop_unreturned_mod_contents(
        &mut self,
        pos: (i32, i32, i32),
        mut taken: ModBlockEntity,
    ) {
        if taken.items.iter().any(Option::is_some) {
            let lost = std::mem::take(&mut taken.items);
            let at = BlockPos {
                x: pos.0,
                y: pos.1,
                z: pos.2,
            };
            self.with_level(|_, level, _| level.drop_lost_contents(at, lost));
            self.world.modified.insert((pos.0 >> 4, pos.2 >> 4));
        }
    }
}
