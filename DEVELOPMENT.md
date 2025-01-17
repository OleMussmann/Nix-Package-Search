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

1. Document future changes in the [CHANGELOG.md](./CHANGELOG.md) under "Unreleased". Check if the `pre-push` hooks pass - apart from tags.

    ```bash
    ./hooks/pre-push
    ```

1. Do a dry-run with

    ```bash
    cargo release [LEVEL|VERSION]
    ```

    and review the changes. Possible choices for `LEVEL` are `beta`, `alpha` or `rc` for development (pre-) releases and `major`, `minor`, `patch` or `release` (removes the pre-release extension) for production releases.

1. Execute the `cargo release`. This will run the tests, tag the release and push to GitHub.

    ```bash
    cargo release [LEVEL|VERSION] --execute --no-publish
    ```

1. Create a pull request for the `development` branch into `main`. If all pre-checks succeed, conclude the pull request. A release draft will is created from [CHANGELOG.md](CHANGELOG.md).

1. Review the release draft under "Releases" and publish the release.
