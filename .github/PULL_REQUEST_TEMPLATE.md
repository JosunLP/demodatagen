<!--
Thanks for contributing to demodatagen!
Please fill out the sections below. PRs that pass CI (fmt, clippy -D warnings, test)
and include a clear description are reviewed fastest.
-->

## Summary

<!-- Provide a clear, concise description of what this PR does and why. -->



<!-- Link any related issues, e.g. "Closes #123" or "Relates to #456". -->
Closes #

## Type of change

<!-- Check all that apply. -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] New output format
- [ ] Breaking change (fix or feature that changes existing behavior or public API)
- [ ] Performance improvement
- [ ] Refactor (no functional change)
- [ ] Documentation only
- [ ] Build, CI, or tooling change

## How was this tested?

<!--
Describe the tests you ran and how to reproduce them. Include exact commands,
schemas, seeds, and the environment (OS, Rust version) where relevant.
-->



## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes with no warnings
- [ ] `cargo test` passes
- [ ] Documentation (README and/or doc comments) updated where needed
- [ ] CHANGELOG updated with a user-facing entry

### If this PR adds a new output format

<!-- Delete this section if it does not apply. New formats must be wired in all 7 places. -->

- [ ] 1. Registered in the format enum / type definition
- [ ] 2. Mapped from the CLI / file-extension parsing
- [ ] 3. Generator/writer implementation added
- [ ] 4. Dispatched from the central format-dispatch logic
- [ ] 5. Tests added covering the new format (including deterministic seeding)
- [ ] 6. README and format list / documentation updated
- [ ] 7. CHANGELOG entry added for the new format

## Additional notes

<!-- Anything else reviewers should know: trade-offs, follow-ups, screenshots, etc. -->
