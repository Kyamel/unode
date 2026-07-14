# unode Host Bridge

## Why the bridge exists

`unode` is domain-agnostic. It knows nothing about works, chapters, users, or
manga. The bridge is where that knowledge lives.

The bridge answers: "what can a plugin do in *this* app?"

---

## Bridge responsibilities

- Typed domain API exposed to plugins (`MugenHostApi`)
- Domain permission strings (`catalog.read`, `library.write`)
- Permission metadata mapping each API method to its required permission
- Domain models (`WorkSummary`, `ChapterSummary`, `StaffCredit`)
- Domain-specific UI sugar built from unode core AST primitives
- Locale provider: exposes the app's current locale to plugins via `locale_get`

---

## Bridge structure

```
mugens-sdk/
  src/
    lib.rs
    api/
      catalog.rs    — CatalogApi trait + implementations
      library.rs    — LibraryApi trait
      reader.rs     — ReaderApi trait
    models/
      work.rs       — WorkSummary, WorkDetail
      chapter.rs    — ChapterSummary
      staff.rs      — StaffCredit
    host_functions/
      web.rs        — JS-facing host function registration
      tui.rs        — Wasmtime host function registration
    permissions.rs  — HostApiMeta: method → required permission
    locale.rs       — locale_get host function implementation
    sugar/
      work_banner.rs   — workBanner() DSL helper
      chapter_list.rs  — chapterList() DSL helper
```

---

## Domain API traits

```rust
#[async_trait]
pub trait CatalogApi: Send + Sync {
    async fn get_work(&self, id: &str) -> Result<WorkDetail>;
    async fn list_chapters(&self, work_id: &str) -> Result<Vec<ChapterSummary>>;
    async fn search_works(&self, query: &str) -> Result<SearchResults>;
}

#[async_trait]
pub trait LibraryApi: Send + Sync {
    async fn is_favorited(&self, work_id: &str) -> Result<bool>;
    async fn add_favorite(&self, work_id: &str) -> Result<()>;
    async fn remove_favorite(&self, work_id: &str) -> Result<()>;
}
```

---

## Permission metadata

```rust
pub struct HostFnMeta {
    pub name: &'static str,
    pub required_permission: &'static str,
}

pub const HOST_FN_META: &[HostFnMeta] = &[
    HostFnMeta { name: "catalog_get_work",      required_permission: "catalog.read" },
    HostFnMeta { name: "catalog_list_chapters", required_permission: "catalog.read" },
    HostFnMeta { name: "catalog_search_works",  required_permission: "catalog.read" },
    HostFnMeta { name: "library_is_favorited",  required_permission: "library.read" },
    HostFnMeta { name: "library_add_favorite",  required_permission: "library.write" },
    HostFnMeta { name: "library_remove_favorite", required_permission: "library.write" },
];
```

At instantiation, the renderer iterates `HOST_FN_META`, checks
`guard.has(required_permission)` for each entry, and only registers the
functions whose permission was granted. Plugins cannot call un-registered functions.

---

## Locale provider

The bridge implements the `locale_get` host function:

```rust
// In tui.rs
linker.func_wrap("unode", "locale_get", |mut caller: Caller<'_, TuiState>| {
    let locale = caller.data().preferences.locale.clone(); // e.g. "pt-BR"
    write_string(&mut caller, &locale)
})?;
```

The locale value is a BCP 47 tag. Plugins call `ctx.locale()` which invokes
this host function. Plugins use the returned string to select from their own
catalogs — the bridge does not translate for plugins.

---

## Domain sugar (UI component helpers)

Bridge-level sugar builds rich UI layouts from unode core primitives.
Plugins import these helpers instead of building the layout by hand.

```rust
// mugens-sdk/src/sugar/work_banner.rs
pub fn work_banner(view_model: &WorkBannerViewModel) -> UiNode {
    ui::stack(Some(Gap::Sm), vec![
        ui::media(MediaNode {
            ref_: view_model.cover_ref.clone(),
            media_kind: MediaKind::Cover,
            alt: view_model.cover_alt.clone(),
            aspect_ratio: Some(AspectRatio::Poster),
            ..Default::default()
        }),
        ui::stack(Some(Gap::Xs), vec![
            ui::text(view_model.title.clone())
                .role(TextRole::Title)
                .emphasis(Emphasis::Strong),
            ui::inline(Some(Gap::Xs)).wrap(true).children(
                view_model.badges.iter().map(|b|
                    ui::badge(b.label.clone()).tone(b.tone)
                ).collect()
            ),
        ]),
    ])
}
```

This sugar is Mugen-specific and lives in the bridge crate, not in unode.

---

## What should NOT move into unode core

These are clearly app-specific and must stay in the bridge:

- `workBanner`, `chapterList`, `workMetadata` — domain sugar
- `WorkSummary`, `ChapterSummary` — domain models
- `catalog.read`, `library.write` — domain permission strings
- Route tabs chrome (`screen.meta.routeTabs`) — app navigation pattern
- Any reference to manga, anime, chapters, or works

If another app wanted to use unode, it would write its own bridge with its own
domain APIs. unode would be unchanged.

---

## What belongs in unode core vs the bridge

| Concept | Location | Reason |
|---|---|---|
| AST node types | unode | Universal across all apps |
| StateStore | unode | Generic reactive state |
| ExprResolver | unode | Generic binding tracking |
| PermissionGuard types | unode | Generic permission model |
| `locale_get` contract | unode | Interface only |
| `locale_get` implementation | Bridge | App owns locale state |
| i18n catalog registry | unode | Generic, plugin-owned catalogs |
| `WorkSummary` model | Bridge | Mugen-specific |
| `catalog_get_work` host fn | Bridge | Mugen-specific |
| Navigation items, commands | unode | Generic plugin registration |
| Route tab chrome | Bridge (or app shell) | Mugen-specific navigation pattern |
