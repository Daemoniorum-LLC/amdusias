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
