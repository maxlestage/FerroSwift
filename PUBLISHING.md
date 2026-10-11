# Publishing on crates.io

FerroSwift is published on [crates.io](https://crates.io/crates/ferroswift) as four crates,
in this order: `ferroswift-macros`, `ferroswift-transpiler`, `ferroswift` and
`ferroswift-cli` (the `ferroswift` command). All share the workspace version and the
[proprietary license](LICENSE): the source is visible there so that it can be audited and
built, and anyone may depend on it.

## One release = one tag

1. Bump `version` in the root [`Cargo.toml`](Cargo.toml) (`[workspace.package]`) and the
   `version = "…"` of the inter-crate dependencies in `crates/*/Cargo.toml`, then commit.
2. Open **Releases → Draft a new release**, create the tag `vX.Y.Z` (the same version with a
   `v` in front) and tap **Publish release** — or push the tag.

The [`publish.yml`](.github/workflows/publish.yml) workflow checks that the tag matches the
version, runs the tests and publishes the four crates in order, skipping a crate whose version
is already online. It works from the GitHub website on a phone; nothing is installed locally.
From the **Actions** tab it can also be started by hand, as a dry run by default (the packages
are built and checked, nothing is uploaded).

## One-time setup

- A crates.io account with a verified e-mail address.
- A token from [crates.io/settings/tokens](https://crates.io/settings/tokens) with the scopes
  **publish-new** and **publish-update**, saved as the repository secret
  `CARGO_REGISTRY_TOKEN` (GitHub → Settings → Secrets and variables → Actions).

## From a computer

```sh
cargo login                              # paste the token, once
cargo publish --workspace --dry-run      # packages and builds the four crates
cargo publish --workspace                # publishes them in dependency order
```

## The ecosystem, in order

The six projects by Maxime Nathan Lestage depend on each other through crates.io, so they
are published in this order, each one after the previous one is online:

1. **Active** — `active`
2. **Vitesse** — `vitesse`
3. **Securite** — `securite`
4. **Memoire** — `memoire-orm-derive`, then `memoire-orm`
5. **Paquets** — `paquets-macros`, then `paquets` (depends on the four above)
6. **FerroSwift** — `ferroswift-macros`, `ferroswift-transpiler`, `ferroswift`, `ferroswift-cli`
   (this repository)
