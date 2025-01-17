# Development Instructions
## Tests
Most tests run quick, execute them with:

```bash
cargo test
```

Refreshing the cache needs a working internet connection and might take a while.
These tests are by default disabled. Include them with

```bash
cargo test -- --include-ignored
```

Use `tarpaulin` to check for code coverage. Make sure to `--include-ignored` tests and include the integration tests with `--follow-exec`. This can take a long time. To generate a HTML report, run the check with

```bash
cargo tarpaulin --all-features --workspace --timeout 360 --out Html --follow-exec -- --include-ignored
```

## Benchmarking
First build `nps` in release mode.

```bash
cargo build --release
```

Then run benchmarks on the produced executable with:

```bash
hyperfine './target/release/nps -e neovim'
```

## Git Hooks
Review the hooks in the [hooks](./hooks) folder and use them with

```bash
git config core.hooksPath hooks
```

## Release

Document future changes in the [CHANGELOG.md](./CHANGELOG.md) under "Unreleased". Check if the `pre-push` hooks pass - apart from tags.

```bash
./hooks/pre-push
```

Do a dry-run with

```bash
cargo release [LEVEL|VERSION]
```

and review the changes. Possible choices for `LEVEL` are `beta`, `alpha` or `rc` for development (pre-) releases and `major`, `minor`, `patch` or `release` (removes the pre-release extension) for production releases. Then execute the release. This will run the tests, tag the release and push to GitHub.

```bash
cargo release [LEVEL|VERSION] --execute --no-publish
```

