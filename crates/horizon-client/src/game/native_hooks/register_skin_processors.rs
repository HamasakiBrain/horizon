use super::*;

impl Game<'_> {
    /// Skins vanilla refuses: `register(processor)` adds the mod's own `SkinProcessor`, which the skin
    /// downloads ask for such a skin's 64×64 picture. Asked once at start on the client, before
    /// resources load.
    pub(in crate::game) fn native_register_skin_processors(
        context: &mod_api::RegistrationContext,
        processors: &mut dyn mod_api::SkinProcessors,
    ) {
        Self::fire_register_skin_processors(context, processors);
    }
}
