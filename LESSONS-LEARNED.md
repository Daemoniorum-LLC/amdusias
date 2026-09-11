# Lessons Learned

Organizational memory for the Amdusias audio engine. Document mistakes, discoveries, and successful patterns here so future agents don't repeat failures or miss proven approaches.

## Format

Each entry should follow this structure:

```
## [Date] - [Session/Feature Name]

### Context
What were we trying to do?

### What Happened
What went wrong or right?

### Root Cause
Why did this happen?

### Lesson
What should future agents know?

### Prevention
How do we avoid this in future?
```

---

## Entries

*Add new entries above this line, newest first.*

---

## 2026-09-11 - LARES-346: sigil-parser CI pin

### Context
`ci.yml` ran `cargo install sigil-parser` unpinned. amdusias#13's first-ever CI run resolved to
v0.3.0, a compiler three weeks older than amdusias itself, and failed almost entirely for
toolchain reasons unrelated to the DSP code. Asked to pin the compiler to a version that accepts
`Sigil.toml`, takes a directory argument for `check`, and parses `.sg`.

### What Happened
No version of `sigil-parser` — published (0.1.1, 0.3.0, 0.4.0-rc.4) or unreleased
`origin/develop` HEAD — makes `sigil check <directory>` work. `check` has always taken exactly one
file argument (`fs::read_to_string(path)`, no `is_dir()` branch), at every point in the project's
git history. `ci.yml`'s `sigil check .` step therefore cannot pass under any pin.

### Root Cause
A CLI capability gap in `sigil-parser` itself, not a version-skew problem. `lint`, `build`,
`build_workspace`, and `test` all walk directories; `check` never gained that support.

### Lesson
Don't assume "pin an older CI failure to a newer/different version" is always the fix — verify
the actual command the CI step runs is even *capable* of succeeding under any version before
choosing one. Read the CLI's own source at the candidate refs rather than trusting behavior
inferred from one failing run.

### Prevention
Filed upstream as sigil-lang#178. Until `sigil check` supports a directory argument,
`ci.yml`'s Type Check job cannot be made green by pinning alone — full findings in the Lares doc
trail for LARES-346 and LARES-343 (`~/development/resources/tickets/LARES-346/findings.md`).
No change was made to `ci.yml` in this session, per the ticket's own instruction not to invent a
workaround that would bury this finding.

## 2026-02-11 - Initial Repository Setup

### Context
Extracting amdusias from monorepo (~/dev2/workspace/nyx/amdusias) to standalone repository for Sigil migration.

### What Happened
Created infrastructure: CONCLAVE.sigil, LESSONS-LEARNED.md, .claude/CLAUDE.md, methodology docs.

### Root Cause
Repository extracted without Daemoniorum's current best practice infrastructure.

### Lesson
When extracting projects from monorepo, always add methodology infrastructure before starting work.

### Prevention
Checklist for repo extraction:
- [ ] .claude/CLAUDE.md
- [ ] CONCLAVE.sigil
- [ ] LESSONS-LEARNED.md
- [ ] docs/methodologies/
- [ ] docs/sessions/
