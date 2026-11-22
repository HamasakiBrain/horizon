//! The `register_skin_processors` hook: declarations the mod API generator reads (see `hooks/mod.rs`).

pub trait Hooks {
    /// Player skins the game refuses (`SkinTextureDownloader.processLegacySkin` takes only 64×64 and
    /// 64×32): `register(processor)` adds the mod's own `SkinProcessor`, which the skin downloads ask
    /// for such a skin's 64×64 picture. Asked once at start on the client, before resources load.
    #[notes = r#"## Skin processors

The game shows a downloaded player skin only when it is 64×64 or the older 64×32, as vanilla does; any other size
leaves the player in the default skin. A skin server that hands out larger skins (HD skins of 128×128 to 1024×1024)
needs a client mod to say how to show them: `Hooks::register_skin_processors` registers the mod's `SkinProcessor`
once at start, and every skin the game refuses is offered to it. Its answer is the 64×64 RGBA the skin's slot
holds, taken as it is (alpha included); None leaves the skin refused. Vanilla-sized skins never reach it.

```rust,ignore
struct HdSkins;
impl SkinProcessor for HdSkins {
    fn process(&self, width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
        if width != height || width % 64 != 0 || width == 64 {
            return None;
        }
        let step = (width / 64) as usize;
        let mut out = vec![0u8; 64 * 64 * 4];
        for y in 0..64 {
            for x in 0..64 {
                let s = ((y * step) * width as usize + x * step) * 4;
                out[(y * 64 + x) * 4..][..4].copy_from_slice(&rgba[s..s + 4]);
            }
        }
        Some(out)
    }
}

fn register_skin_processors(&self, _context: &RegistrationContext, event: &mut RegisterSkinProcessorsEvent<'_>) {
    event.processors.register(Arc::new(HdSkins));
}
```

Processors run on the skin download workers (and when an evicted skin is read again from the disk cache), so they
get no `Host` and keep no mutable state; a setting the mod reads elsewhere goes to them through an atomic. The first
processor to answer wins, in registration order. One that panics is asked no more."#]
    #[hook(side = "client", contract = "listen", context = "registration")]
    fn register_skin_processors();
}
