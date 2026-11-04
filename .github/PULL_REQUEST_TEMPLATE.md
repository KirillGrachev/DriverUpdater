## Summary

<!-- What changed and why? Keep it user-visible. -->

## Related issue

<!-- Fixes #123, refs #456, or "none". -->

## Verification

- [ ] `npm run typecheck`
- [ ] `npm test`
- [ ] `npm run build`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`

## UI changes

<!-- Screenshots for light and dark themes, plus the minimum window size when layout changed. -->

## Checklist

- [ ] The change is focused and does not include unrelated formatting churn.
- [ ] Comments follow the JSDoc/Javadoc convention and indentation is four spaces.
- [ ] New behavior has a regression test where practical.
- [ ] New or changed strings exist in the Russian and English reference dictionaries first.
- [ ] No generated files (`dist`, `node_modules`, `src-tauri/target`) are committed.
- [ ] The PR does not contain secrets, personal paths or proprietary assets.
