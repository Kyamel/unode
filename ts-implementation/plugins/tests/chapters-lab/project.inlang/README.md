This local inlang project powers Sherlock tooling for this plugin's runtime JSON catalogs.

Use `const t = ctx.i18n.t` inside `load()` and `render()` so the editor can match `t('key')`
against `./messages/{locale}.json` without changing the `unode` runtime model.
