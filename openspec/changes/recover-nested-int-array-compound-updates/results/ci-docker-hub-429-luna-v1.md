# Supply-chain CI image pull fallback

The exact prior CI run `37991578328` failed before checkout while pulling the DockerHub `rust:1.85`
image (HTTP 429). The root agent traced that failure to the supply-chain job's `cargo-deny-action`
container. This patch removes that action from only the supply-chain job and installs the pinned
`cargo-deny 0.20.2` binary after checkout with stable Rust:

```sh
cargo install cargo-deny --version 0.20.2 --locked
```

The existing root and fuzz check names remain. Their checks now invoke the official CLI directly
with the same config, all-features, workspace, and locked settings, selecting the respective
manifest. No lockfile, dependency, feature, deny policy, cache, or install helper changed.

The official cargo-deny CLI documentation describes installation with `cargo install --locked`,
common `--manifest-path`/`--config`/`--all-features`/`--workspace`/`--locked` options, and `check` as
the command that runs configured checks:
[CLI overview](https://embarkstudios.github.io/cargo-deny/cli/index.html),
[common options](https://embarkstudios.github.io/cargo-deny/cli/common.html),
[check command](https://embarkstudios.github.io/cargo-deny/cli/check.html).

Luna did not execute either check. Root owns local verification with its already available
`cargo-deny 0.20.2` and will freeze workflow metadata after those checks complete.
