# FerroSwift website

The presentation site, documentation and live playground of FerroSwift, in French and English, written in Rust with [active](https://maxlestage.github.io/Active/).

- Pages are rendered by Rust (`src/server/pages.rs`) into static HTML with complete SEO tags (title, description, canonical URL, `hreflang` alternates, Open Graph, JSON-LD).
- The **playground** (`src/islands.rs`) is an active island: the server renders it with the first example already translated, and the browser hydrates it with the same code compiled to WebAssembly, where `ferroswift_transpiler::transpile` runs on every keystroke.
- The Rust/Swift comparisons of the home page are produced by the transpiler when the site is generated, so they always show its real output.
- Pages without an island (docs, 404) load no WebAssembly. The home page starts the bundle when the playground comes near the viewport.

## Layout

```text
site/
├── src/
│   ├── lib.rs           shared by the server and the browser
│   ├── routes.rs        /{fr,en}/, /{lang}/playground/, /{lang}/docs/{slug}/, base path
│   ├── highlight.rs     Rust, Swift, shell and TOML syntax highlighter (escapes HTML)
│   ├── examples.rs      the playground's examples (examples/*.rs, include_str!)
│   ├── islands.rs       the playground island
│   ├── labels.rs        the island's texts, French and English
│   ├── client/          browser only (wasm32): hydration, editor keys, panic guard
│   ├── server/          native only: pages, docs (Markdown), export, dev server
│   └── main.rs          `ferroswift-site [serve]` and `ferroswift-site export <dir>`
├── docs/{fr,en}/*.md    documentation pages (front matter: title, description, order)
├── examples/*.rs        playground examples (not cargo examples)
├── static/              app.css, favicon, og.png, fonts (Geist, SIL OFL)
├── pkg/                 the browser bundle, generated (git-ignored)
└── tests/               every route, the export, the island, the examples
```

## Build the browser bundle

The playground needs the WebAssembly bundle in `site/pkg`. From the workspace root:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # same version as in Cargo.toml

cargo build -p ferroswift-site --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --out-dir site/pkg \
  ${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/wasm-release/ferroswift_site.wasm
```

The bundle is about 1.1 MB (390 kB gzipped): it contains the whole transpiler. Without it the site still works; the playground then shows the server's translation and is not editable live.

## Develop

```sh
cargo run -p ferroswift-site            # or `-- serve`; http://localhost:3000
PORT=8080 cargo run -p ferroswift-site  # another port (HOST=0.0.0.0 to reach it from a phone)
```

The development server has no dependencies (std only). It reloads the docs and the static files on every page, so editing Markdown, CSS or rebuilding the bundle needs no restart; Rust changes do.

## Export the static site

```sh
cargo run -p ferroswift-site -- export dist
```

writes every page as `dist/<path>/index.html`, plus `index.html` (sends visitors to their language, French by default), `404.html`, `sitemap.xml`, `robots.txt`, `.nojekyll`, `static/` and `pkg/`.

- `BASE_PATH` prefixes every link and asset, for a site published under a sub-path: `BASE_PATH=/FerroSwift` on GitHub Pages (`https://maxlestage.github.io/FerroSwift/`).
- `SITE_URL` is the public address used for absolute URLs (canonical, Open Graph, sitemap); the base path is appended when missing.

```sh
BASE_PATH=/FerroSwift SITE_URL=https://maxlestage.github.io \
  cargo run --release -p ferroswift-site -- export dist
python3 -m http.server --directory dist 8000   # http://localhost:8000 (with BASE_PATH empty)
```

### GitHub Pages

`.github/workflows/pages.yml` publishes the site after every successful CI run on `master`: it builds the bundle as above and runs the export with `BASE_PATH` and `SITE_URL` given by `actions/configure-pages`. One-time setup: Settings → Pages → Build and deployment → Source: GitHub Actions.

## Test

```sh
cargo test -p ferroswift-site
cargo clippy -p ferroswift-site --all-targets -- -D warnings
cargo clippy -p ferroswift-site --lib --target wasm32-unknown-unknown -- -D warnings
```

The tests render every route in both languages, export the site behind `/FerroSwift`, render the island for every example and check that each example translates without errors — so a change of the transpiler that breaks an example fails here.

## Write documentation

Add `docs/en/<slug>.md` and `docs/fr/<slug>.md`:

```markdown
---
title: Previews
description: One sentence, used in the docs index and as the page's meta description.
order: 6
---
```

Pages are sorted by `order`; a page missing in one language shows the other one. Code blocks are highlighted for `rust`, `swift`, `sh` and `toml`; `> **Note:**`, `> **Tip:**` / `> **Astuce :**` and `> **Warning:**` / `> **Attention :**` make callouts. Link other pages relatively (`../cli/`), so links work behind any base path.

## The playground's examples

`examples/*.rs` are listed in `src/examples.rs`. They are plain Rust using the `ferroswift` crate and must translate without error diagnostics (a test checks it).
