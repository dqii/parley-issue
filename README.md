# fontique `fallback_families` cache bug (parley 0.7.0)

[![CI](https://github.com/dqii/parley-issue/actions/workflows/test.yml/badge.svg)](https://github.com/dqii/parley-issue/actions/workflows/test.yml)

Minimal reproduction for a bug in [fontique](https://github.com/linebender/parley/tree/main/fontique) where `FallbackData::fallback_families` caches its result and doesn't invalidate when font attributes (`FontWeight`, `FontStyle`) change between runs. This causes bold/italic to silently fail on later text ranges.

## The Bug

When using `ranged_builder` to apply **bold** to a _later_ range in the text (not the first run), the styled run gets the same font as the default run — no weight change is applied.

**Root cause**: `FallbackData::fallback_families()` caches its result on first call and returns the cached value on subsequent calls, even when the `attrs` argument has changed (different weight/style).

### Test results on stock parley 0.7.0

| Test | Result | Notes |
|------|--------|-------|
| `test_bold_first_word_changes_width` | PASS | Bold on first range works correctly |
| `test_bold_second_word_changes_width` | **FAIL** | Bold on second range has no effect (the bug) |
| `test_underline_works_on_both_ranges` | PASS | Non-font-selection styles unaffected |
| `test_font_size_works_on_both_ranges` | PASS | Non-font-selection styles unaffected |

Italic (`FontStyle`) has the same root cause but is harder to test cross-platform since italic Inter has the same advance widths as regular Inter.

## Running the tests

```bash
# Stock parley 0.7.0 — bold second-word test fails (the bug)
cargo test

# With patched fontique — all tests pass
cat >> Cargo.toml << 'EOF'

[patch.crates-io]
fontique = { git = "https://github.com/dqii/parley", branch = "fix/fontique-fallback-cache" }
EOF
cargo test
```

## The Fix

A 3-line addition in `fontique/src/collection/query.rs` — when attributes change, also clear the cached fonts on `fallback_families` (not just `families`):

```diff
--- a/fontique/src/collection/query.rs
+++ b/fontique/src/collection/query.rs
@@ impl<'a> Query<'a>
             for family in &mut self.state.families {
                 family.clear_fonts();
             }
+            for family in &mut self.state.fallback_families {
+                family.clear_fonts();
+            }
             self.attributes = attributes;
```

**Fix branch**: [`dqii/parley#fix/fontique-fallback-cache`](https://github.com/dqii/parley/tree/fix/fontique-fallback-cache)

## CI

The [GitHub Actions workflow](.github/workflows/test.yml) runs two jobs:

- **Stock parley 0.7.0** — demonstrates the bug (expected to fail, `continue-on-error: true`)
- **Patched fontique** — applies the fix via `[patch.crates-io]` (expected to pass)
