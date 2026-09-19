---
name: extract-paths-from-text
description: Extract real (path, line?, col?) matches from arbitrary text via loose regex + filesystem validation, not a strict regex alone
source: auto-skill
extracted_at: '2026-09-18T03:38:55.754Z'
---

# Extract file paths + line numbers from arbitrary text

Use when building a parser that pulls `(path, Option<line>, Option<col>)` out of
unstructured text (grep output, compiler errors, logs) — especially when the
matches will be highlighted/clickable. Core lesson: **a regex alone cannot tell
`mission.txt` (a word) from `main.rs` (a file). Only the disk knows.** So use
two stages, and let the filesystem be the judge.

## Stage 1 — loose candidate regex

The `regex` crate REJECTS lookaround (`(?=...)`, `(?<=...)`) at runtime — a
regex that compiles in PCRE/Python will panic on `LazyLock` init there. Write
alternatives without it:

```rust
r"(?P<path>[^\s:/]*/[^\s:]*|(?:[A-Za-z0-9_+~-]+\.)+[A-Za-z0-9_+~-]+)(?::(?P<line>[0-9]+)(?::(?P<col>[0-9]+))?)?"
```

- `[^\s:/]*/[^\s:]*` — "token containing at least one `/`" (the no-lookahead
  way). Covers `/abs`, `./rel`, `../up`, `~/home`, `src/main.rs`.
- `(?:[A-Za-z0-9_+~-]+\.)+[A-Za-z0-9_+~-]+` — bare `name.ext`, no trailing dot
  (so `sentence.` and `e.g.` yield `e.g` at worst).
- `:42` or `:42:10` suffix → line / line+col (rustc, grep -n, vimgrep formats).
- Keep `:` and whitespace out of path char classes; don't try to make this
  regex reject junk — that's stage 2's job.

## Stage 2 — validate against the filesystem

For each candidate:

1. Reject all-slash tokens (`/`, `//`) — root dirs exist and would be junk hits.
2. Expand `~/` via `$HOME` before checking.
3. `Path::new(candidate).exists()`; relative paths resolve against cwd.
4. If it doesn't exist, trim ONE trailing punctuation char
   (`.` `,` `;` `:` `)` `]` `}` `>` `"` `'`) and retry — real text glues
   punctuation onto paths (`src/main.rs,`), real filenames almost never end so.
5. Only keep the match — and its `:line` — if a prefix exists on disk. If the
   token needed trimming, DROP any parsed `:line` suffix (it was glued to
   punctuation, not the path).

Also return byte `start`/`end` spans per match (from the regex captures,
adjusted for trimming) — highlighting/click handlers need offsets, not strings.

Trade-off to state: a false-positive highlight is worse than a miss when
clicking opens the file, so validating on disk is the right bias. Dirs match
too (`Path::exists`) — acceptable/useful.

## Fixture and test rules that fall out of disk validation

- Fixtures must reference files that REALLY exist in the repo (e.g.
  `src/main.rs`, `Cargo.toml`) — a fixture naming `/Users/foo/bar.rs` produces
  zero matches by design.
- Absolute-path cases can't live in static fixtures (not portable/real); build
  them in-test with `current_dir().join("src/main.rs")`. `cargo test` runs with
  cwd = package root, so repo-relative fixtures work.
- A "prose, expect nothing" fixture only proves something if it includes the
  traps: full stops after words, `e.g.`, version numbers (`3.14`), URLs.
- One caller-facing test should slice the input with the returned span and
  assert it equals the expected `path:line` substring.
