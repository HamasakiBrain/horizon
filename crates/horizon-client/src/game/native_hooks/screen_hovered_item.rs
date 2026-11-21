use super::*;

/// Helpers for native mods, kept as stable as the hook points.
// Native mods use these; nothing in Horizon does.
#[allow(dead_code)]
impl Game<'_> {
    /// The id of the item in the slot under the pointer on the open container screen
    /// (`AbstractContainerScreen.getHoveredSlot().getItem()`), None over no slot or an empty one.
    pub(in crate::game) fn screen_hovered_item(&self) -> Option<mod_api::ItemKey> {
        if self.mod_screen_under().is_some() {
            // A mod's own screen over a container screen (`mod_screen_open_over_container`)
            // hides that screen's slots: the pointer is over the mod's screen, not a slot.
            return None;
        }
        let size = self
            .renderer
            .as_ref()
            .map(|r| [r.config.width, r.config.height])?;
        let batch = HudBatch::new(size, self.settings.gui_scale_inputs());
        let slot = self.slot_at(&batch, self.cursor.0, self.cursor.1)?;
        mod_api::__private::item_key(&self.slot_view(slot)?.item)
    }
}
