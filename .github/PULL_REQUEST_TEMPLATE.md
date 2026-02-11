## Summary

<!-- Brief description of what this PR does -->

## Changes

<!-- List the key changes made -->

-

## Testing

<!-- How was this tested? -->

- [ ] Unit tests pass (`cargo test --workspace`)
- [ ] Clippy passes (`cargo clippy --workspace --all-targets`)
- [ ] Formatted (`cargo fmt --check`)
- [ ] WASM builds (`wasm-pack build crates/amdusias-web`)

## Audio Testing

<!-- If this affects audio processing: -->

- [ ] Tested with real audio (not just unit tests)
- [ ] Verified no audio glitches or artifacts
- [ ] Tested at multiple sample rates (44.1k, 48k, 96k)
- [ ] Tested at multiple buffer sizes (128, 256, 512, 1024)

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) guide
- [ ] My code follows the project's style guidelines
- [ ] I have added tests that prove my fix/feature works
- [ ] New and existing tests pass locally
- [ ] I have updated documentation as needed
- [ ] This change does not introduce security vulnerabilities
- [ ] DSP code is real-time safe (no allocations, locks, or blocking)

## Related Issues

<!-- Link any related issues: Fixes #123, Relates to #456 -->
