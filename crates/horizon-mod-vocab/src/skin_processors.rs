//! Player skins vanilla refuses, as a client mod registers it in `register_skin_processors`.
//!
//! The game takes a downloaded skin only when it is 64×64 or the older 64×32
//! (`SkinTextureDownloader.processLegacySkin`); a skin of another size is dropped and the player
//! shows the default skin. A mod's processor is asked for such a skin and may answer the 64×64
//! RGBA the game's skin slot holds: a skin server's HD skins (128×128 to 1024×1024), say. It runs on
//! the skin download workers, so it gets no `Host` and keeps no mutable state.
use std::sync::Arc;

/// A mod's processor for skins the game refuses.
pub trait SkinProcessor: Send + Sync + 'static {
    /// The 64×64 RGBA (16384 bytes, row by row) to show for a downloaded skin of `width`×`height`
    /// RGBA pixels the game refuses (not 64×64 or 64×32), or None to leave it refused. Taken as it
    /// is, alpha included; an answer of another length is ignored. Asked on a skin download worker;
    /// a processor that panics is asked no more.
    fn process(&self, width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>>;
}

/// The processor registry `register_skin_processors` hands out.
pub trait SkinProcessors {
    /// `processor` is asked for every skin the game refuses, in registration order; the first
    /// answer is the skin.
    fn register(&mut self, processor: Arc<dyn SkinProcessor>);
}

/// The game's [`SkinProcessors`]: the processors in registration order.
#[derive(Default)]
// exhaustive: its field is private; the game builds it
// no wire: process-local processor storage; processors run on the skin download workers, not serialized.
pub struct SkinProcessorRegistry {
    processors: Vec<Arc<dyn SkinProcessor>>,
}

impl SkinProcessors for SkinProcessorRegistry {
    fn register(&mut self, processor: Arc<dyn SkinProcessor>) {
        self.processors.push(processor);
    }
}

impl SkinProcessorRegistry {
    /// The processors, in registration order.
    pub fn into_processors(self) -> Vec<Arc<dyn SkinProcessor>> {
        self.processors
    }
}
