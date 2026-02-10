# Cahnon — Functional Specification (Final)

**Tagline: _Write freely. Stay consistent._**

---

## 1. Objective

A desktop application for fiction writers (novel, series, narrative project) to:

- write and organize scenes freely,
- centralize the story universe (bible),
- maintain consistency (timeline, conflicts),
- revise with annotations and tracking,
- export clean manuscripts or full working backups.

**Cahnon** is **modular** and **non-prescriptive**: start writing immediately, structure later.

---

## 2. Core Principles

1. **Writing first**: no configuration required to start.
2. **Progressive enrichment**: bible, timeline, arcs enabled when needed.
3. **Source of truth**: canonical associations, not detected mentions.
4. **Non-blocking alerts**: always dismissible, never interrupt flow.
5. **Two modes**: Writing (minimal) and Revision (analytical).
6. **Local-only**: your data stays on your machine.
7. **Finished product**: no plugins, no cloud, no endless expansion.

---

## 3. Data Model

### 3.1 Entity Hierarchy

```
Project
  ├── Bible
  │     ├── Character
  │     ├── Location
  │     ├── Object
  │     ├── Faction
  │     ├── Concept/Rule
  │     └── Glossary Term
  ├── Timeline (Events)
  ├── Arcs
  └── Manuscript
        └── Chapters
              └── Scenes
```

### 3.2 Cardinalities

| Relationship              | Cardinality |
| ------------------------- | ----------- |
| Project → Manuscript      | 1:1         |
| Project → Bible           | 1:1         |
| Project → Timeline        | 1:1         |
| Manuscript → Chapter      | 1:N         |
| Chapter → Scene           | 1:N         |
| Scene → Arc               | N:M         |
| Scene → Bible Entry       | N:M         |
| Scene → Event             | N:M         |
| Bible Entry → Bible Entry | N:M         |

### 3.3 Storage

- **Format**: SQLite database
- **Project file**: single `.cahnon` file
- **Backup export**: JSON + Markdown (human-readable)

### 3.4 Data Safety

- **Autosave**: every 30 seconds and on window blur
- **Undo/Redo**: unlimited within session
- **Soft delete**: 30-day trash before permanent deletion
- **Snapshots**: manual named snapshots, kept indefinitely

### 3.5 Multi-Device Workflow

Project file can be stored in any cloud-synced folder (Dropbox, iCloud, Google Drive, OneDrive, Syncthing).

**Lock file**: when project is open, create `.cahnon.lock` containing machine name and timestamp. On open:

- If lock exists and recent: warn user ("Project may be open on [machine]")
- User can proceed or cancel

**External modification detection**:

- On focus: check if file changed since last read
- If changed: prompt to reload, keep current, or compare

**Sync conflict detection**:

- Detect conflict files created by sync services (e.g., `Project (conflict).cahnon`)
- Prompt user to compare and choose

---

## 4. Manuscript

### 4.1 Chapter

| Attribute | Description                                    |
| --------- | ---------------------------------------------- |
| Title     | Chapter title                                  |
| Summary   | Optional, 1-3 sentences                        |
| Status    | planned / in_progress / draft / revised / done |
| Notes     | Private notes                                  |

### 4.2 Scene

| Attribute   | Description                                              |
| ----------- | -------------------------------------------------------- |
| Title       | Scene title                                              |
| Summary     | 1-3 sentences                                            |
| Text        | Main content (rich text)                                 |
| Status      | planned / to_write / draft / in_revision / done / to_cut |
| POV         | Single character (optional)                              |
| Tags        | Free-form tags                                           |
| Notes       | Private notes                                            |
| TODO        | Task list                                                |
| Word target | Optional word count goal                                 |

### 4.3 Scene Time

- **Point** (default) or **Duration** (start + end)
- **On timeline** or **Off timeline** (dream, flashback, undefined)

Time format: free text ("Day 3", "March 15", "Before the war") or simple date.

### 4.4 Scene Operations

| Operation | Behavior                                |
| --------- | --------------------------------------- |
| Move      | Drag-and-drop; associations follow      |
| Duplicate | Full copy or structure only             |
| Split     | Choose point; user assigns associations |
| Merge     | Text concatenated; associations merged  |
| Delete    | Moves to trash; restorable for 30 days  |

---

## 5. Work Modes

### 5.1 Writing Mode

- Minimal interface, maximum writing space
- Alerts hidden (except critical)
- Focus options:
  - Fullscreen
  - Typewriter scroll (current line centered)
  - Dim surroundings (focus current paragraph)
- Customizable: font, size, line height, text width

### 5.2 Revision Mode

- Full metadata visible
- Annotations panel open
- All alerts active
- Side panel: issues, associations, timeline

---

## 6. Bible (Knowledge Base)

### 6.1 Entry Types

| Type         | Examples                                 |
| ------------ | ---------------------------------------- |
| Character    | People, creatures                        |
| Location     | Places, buildings, regions               |
| Object       | Artifacts, vehicles, important items     |
| Faction      | Organizations, families, nations         |
| Concept/Rule | Magic systems, world rules, technologies |
| Glossary     | Invented terms, languages                |

### 6.2 Common Attributes

| Attribute   | Description                        |
| ----------- | ---------------------------------- |
| Name        | Primary name                       |
| Aliases     | Alternative names                  |
| Description | Short (tooltip) + full (rich text) |
| Status      | draft / canon / TBD                |
| Tags        | Free-form                          |
| Image       | Optional avatar/reference          |
| Notes       | Private                            |
| TODO        | Task list                          |
| Color       | Visual label                       |

### 6.3 Custom Fields

User-defined fields per entry type:

- Text (short/long)
- Number
- Date (free text)
- Boolean
- Dropdown (predefined options)
- Link (to another entry)

### 6.4 Type-Specific Defaults

**Character**: Role (protagonist / antagonist / supporting / minor), Voice notes

**Location**: Parent location (hierarchy)

**Faction**: Type (government, guild, family...), Members (links), Headquarters

**Glossary**: Pronunciation, Etymology, Language

### 6.5 Canonical Association vs Detected Mention

| Concept                   | Description                                                     |
| ------------------------- | --------------------------------------------------------------- |
| **Canonical association** | User confirms: "This entity is in this scene" — source of truth |
| **Detected mention**      | System suggests: "This name appears, associate?" — optional     |

Consistency checks use **only canonical associations**.

### 6.6 Mention Handling

For each detected mention:

- **Accept**: create canonical association
- **Ignore**: dismiss, may reappear if text changes
- **Ignore always**: never suggest this pair again
- **Create new**: make new Bible entry from selection

---

## 7. Relationships (Bible ↔ Bible)

### 7.1 Structure

```
Relationship:
  - source: Entry
  - target: Entry
  - type: predefined or custom
  - note: free text
  - status: active / ended
```

### 7.2 Predefined Types

**Character ↔ Character**: parent of, child of, sibling of, spouse of, friend of, enemy of, mentor of, ally of, knows, killed, saved

**Character ↔ Location**: lives in, born in, works at, owns

**Character ↔ Faction**: member of, leader of, founder of

**Character ↔ Object**: owns, created, seeks

**Location ↔ Location**: contains, part of, near

**Faction ↔ Faction**: ally of, enemy of, parent of

User can add custom relationship types.

### 7.3 View

List view per entry, filterable by type. No graph visualization.

---

## 8. Arcs

### 8.1 Attributes

| Attribute   | Description                                    |
| ----------- | ---------------------------------------------- |
| Name        | Arc name                                       |
| Description | What this arc is about                         |
| Stakes      | What's at risk                                 |
| Characters  | Key characters involved                        |
| Status      | setup / active / climax / resolved / abandoned |
| Color       | Visual label                                   |

### 8.2 Rules

- A scene belongs to 0..N arcs
- Flat structure (no sub-arcs)
- View: arc presence across chapters/scenes

---

## 9. Timeline

### 9.1 Event

| Attribute   | Description                           |
| ----------- | ------------------------------------- |
| Title       | Event name                            |
| Description | Details                               |
| Time        | Point or interval (free text or date) |
| Type        | plot / backstory / historical         |
| Importance  | minor / moderate / major              |
| Links       | Scenes and Bible entries              |

### 9.2 Dual View

- **Chronological order**: when events happen
- **Narrative order**: when reader discovers them

Simple dual-track visualization with connecting lines.

### 9.3 Evaluable Scenes

Timeline conflict detection applies only to scenes with:

- A canonical character
- A canonical location
- A defined time

Otherwise: "non-evaluable" (no alerts, suggestion to complete).

---

## 10. Narrative Templates

### 10.1 Built-in Templates

| Template       | Beats                                                                  |
| -------------- | ---------------------------------------------------------------------- |
| Three-Act      | Setup, Confrontation, Resolution                                       |
| Save the Cat   | 15 beats (Opening Image → Final Image)                                 |
| Hero's Journey | 12 stages                                                              |
| Seven-Point    | Hook, Plot Turn 1, Pinch 1, Midpoint, Pinch 2, Plot Turn 2, Resolution |

### 10.2 Template Structure

```
Template:
  - name
  - steps[]:
      - name
      - description
      - story_percentage (%)
      - color
```

### 10.3 Rules

- Project has 0 or 1 active template
- Scene has 0 or 1 step assignment
- User can rename, add, remove, reorder steps
- Changing template: previous assignments kept in history, user remaps

---

## 11. Navigation & Organization

### 11.1 Outline

- Tree view: Chapters → Scenes
- Drag-and-drop reordering
- Status indicators (color-coded)
- Keyboard navigation

### 11.2 Quick Open

`Cmd+K` / `Ctrl+K`: jump to any scene, character, location by typing name.

### 11.3 Filters

Filter scenes by:

- Status
- POV
- Arc
- Tag
- Character (canonical)
- Location (canonical)

Combine multiple filters. Save filter combinations as named collections.

### 11.4 Corkboard

- One card per scene: title, summary, status, POV, arc colors, word count
- Drag-and-drop reordering
- Grouping: by chapter, by arc, by status, by POV
- Multi-select for bulk operations

---

## 12. Context Panel

When editing a scene, side panel shows:

- Canonical associations (characters, locations, objects)
- Linked timeline events
- Linked arcs
- Template step (if assigned)
- Scene notes and TODO
- Word count (scene / chapter / total)

---

## 13. Revision & Annotations

### 13.1 Text Annotations

- Attach comment to text selection
- Type: comment / question / TODO
- Status: open / resolved
- Filter by status

### 13.2 Review Grid

Spreadsheet view of all scenes:

| Column    | Type         |
| --------- | ------------ |
| Scene     | link         |
| POV Goal  | text         |
| Conflict? | yes/no       |
| Change?   | yes/no       |
| Tension   | low/med/high |
| Setup for | scene link   |
| Payoff of | scene link   |
| Notes     | text         |

### 13.3 Revision Checklist

Generic checklist (per scene or global):

- [ ] Scene has conflict
- [ ] Something changes
- [ ] POV consistent
- [ ] Enters late, exits early
- [ ] No info dumps

---

## 14. Issues

### 14.1 Issue Types

| Type                                        | Auto-detected    |
| ------------------------------------------- | ---------------- |
| Timeline conflict (character in two places) | Yes              |
| TBD in done scene                           | Yes              |
| Orphan mention (name without entry)         | Yes (suggestion) |
| Bible contradiction                         | Manual           |
| Continuity error                            | Manual           |

### 14.2 Issue Attributes

| Attribute       | Description                 |
| --------------- | --------------------------- |
| Type            | See above                   |
| Title           | Short description           |
| Description     | Details                     |
| Severity        | info / warning / error      |
| Status          | open / resolved / ignored   |
| Affected        | Scenes and entries involved |
| Resolution note | How it was resolved         |

### 14.3 Issue View

Unified list:

- Filter by type, status, severity
- Sort by location in manuscript
- Click to jump to affected scene/entry

### 14.4 Alert Behavior

| Mode     | Alerts shown  |
| -------- | ------------- |
| Writing  | Critical only |
| Revision | All           |

---

## 15. Progress Tracking

### 15.1 Word Count

- Total manuscript
- Per chapter / per scene
- Session count (since app opened)
- Today's count

### 15.2 Goals

- Daily word target (optional)
- Manuscript target (optional)
- Visual progress bar

### 15.3 Status Overview

- Scenes by status (pie chart or bar)
- Completion percentage

---

## 16. History & Versions

### 16.1 Snapshots

| Type               | Trigger             | Retention |
| ------------------ | ------------------- | --------- |
| Manual             | User creates        | Forever   |
| Pre-bulk-operation | Before mass changes | 30 days   |

Snapshot = full project state (manuscript, Bible, timeline, arcs, associations).

### 16.2 Operations

- View snapshot (read-only)
- Compare two versions (diff)
- Restore full project from snapshot
- Restore single scene from snapshot

### 16.3 Scene History

- Every save creates a version
- View any previous version
- Restore any version
- Diff between versions

### 16.4 Cut Library

- Deleted/cut text preserved
- Searchable
- Restore to original scene or insert elsewhere
- Permanent delete after confirmation

---

## 17. Search

### 17.1 Scope

Search across:

- Scene text, titles, summaries
- Bible entries (name, aliases, description, fields)
- Timeline events
- Notes and TODO
- Annotations
- Cuts

### 17.2 Features

- Instant results as you type
- Filter by type
- Snippet preview with highlights
- Click to navigate

### 17.3 Find & Replace

- Scope: entire manuscript, chapter, or scene
- Options: match case, whole words
- Preview all changes before applying
- Single undo for entire operation

---

## 18. Name Registry

### 18.1 Purpose

Detect spelling inconsistencies and variants.

### 18.2 Features

- Centralized list of all proper nouns
- Detect similar spellings (fuzzy match)
- Detect case inconsistencies

### 18.3 Actions

- Merge entries
- Add as alias
- Create new Bible entry
- Mark as intentional (ignore)

---

## 19. Import / Export

### 19.1 Import

| Format            | Support                    |
| ----------------- | -------------------------- |
| Plain text (.txt) | Full                       |
| Markdown (.md)    | Full                       |
| Word (.docx)      | Basic formatting preserved |

Import process:

1. Select file
2. Preview detected structure
3. Configure chapter/scene detection (by heading, separator, or manual)
4. Confirm
5. Report of imported items

### 19.2 Export Profiles

**Reading Export** (clean manuscript):

- Text only, no notes/annotations
- Formats: PDF, DOCX, Markdown, plain text
- Options: title page, chapter headings, scene separators

**Working Export** (full backup):

- Everything: manuscript, Bible, timeline, arcs, issues
- Format: JSON + Markdown files
- Importable back into Cahnon

**Partial Exports**:

- Outline only (Markdown, text)
- Bible only (Markdown, CSV)
- Timeline only (CSV)

### 19.3 Export Options

- Scope: full or selected chapters
- Formatting: font, margins, line spacing
- Scene separators: blank line, \*, #, custom

---

## 20. Interface

### 20.1 Layout

```
┌─────────────────────────────────────────────────────────┐
│  Toolbar                                                │
├──────────┬─────────────────────────────┬───────────────┤
│          │                             │               │
│  Outline │      Editor                 │  Context      │
│          │                             │  Panel        │
│          │                             │               │
├──────────┴─────────────────────────────┴───────────────┤
│  Status bar (word count, save status, mode)            │
└─────────────────────────────────────────────────────────┘
```

- Outline: collapsible
- Context panel: collapsible
- Editor: main focus

### 20.2 Views

- **Editor**: write and edit scenes
- **Corkboard**: visual scene organization
- **Timeline**: chronological view
- **Bible**: browse and edit entries
- **Issues**: consistency problems
- **Review Grid**: revision spreadsheet

### 20.3 Theme

- Light and dark mode
- System preference detection
- Font and size customization in editor

### 20.4 Visual Style

- **Minimalist**: clean, distraction-free, focus on content
- **Inspiration**: iA Writer, Ulysses, Typora
- **Language**: English only

---

## 21. Keyboard Shortcuts

### 21.1 Navigation

| Action         | Mac  | Windows/Linux |
| -------------- | ---- | ------------- |
| Quick Open     | ⌘K   | Ctrl+K        |
| Next scene     | ⌘↓   | Ctrl+↓        |
| Previous scene | ⌘↑   | Ctrl+↑        |
| Toggle outline | ⌘\   | Ctrl+\        |
| Toggle context | ⌘⇧\  | Ctrl+Shift+\  |

### 21.2 Editing

| Action         | Mac | Windows/Linux |
| -------------- | --- | ------------- |
| Save           | ⌘S  | Ctrl+S        |
| Undo           | ⌘Z  | Ctrl+Z        |
| Redo           | ⌘⇧Z | Ctrl+Shift+Z  |
| Find           | ⌘F  | Ctrl+F        |
| Find & Replace | ⌘H  | Ctrl+H        |

### 21.3 Views

| Action     | Mac | Windows/Linux |
| ---------- | --- | ------------- |
| Editor     | ⌘1  | Ctrl+1        |
| Corkboard  | ⌘2  | Ctrl+2        |
| Timeline   | ⌘3  | Ctrl+3        |
| Bible      | ⌘4  | Ctrl+4        |
| Issues     | ⌘5  | Ctrl+5        |
| Dashboard  | ⌘6  | Ctrl+6        |
| Fullscreen | F11 | F11           |
| Focus mode | ⌘⇧F | Ctrl+Shift+F  |

All shortcuts customizable in settings.

---

## 22. Error Handling

### 22.1 Save Failures

- Retry automatically
- Alert if persistent
- Never lose data (keep in memory until saved)

### 22.2 File Corruption

- Detect on open (checksum)
- Attempt recovery
- Load last snapshot if available
- Clear error message with options

### 22.3 Crash Recovery

- On next launch: detect incomplete session
- Offer to restore unsaved changes

### 22.4 Validation

| Field          | Rule                    |
| -------------- | ----------------------- |
| Scene title    | Required, max 200 chars |
| Character name | Required, max 100 chars |
| Aliases        | Max 20 per entry        |

---

## 23. License & Business Model

- **License**: GPL v3
- **Price**: Free
- **Revenue**: Donations (GitHub Sponsors, Ko-fi)
- **Trademark**: "Cahnon" name and logo protected

---

## 24. Platform & Technology

### 24.1 Target

- macOS (Apple Silicon + Intel)
- Windows (10+)
- Linux (AppImage or similar)

Desktop only. No web, no mobile.

### 24.2 Technology Stack

```
┌─────────────────────────────────────────┐
│         Svelte + TypeScript             │  ← UI, views, local state
├─────────────────────────────────────────┤
│              Tauri IPC                  │  ← Async communication
├─────────────────────────────────────────┤
│              Rust Backend               │  ← SQLite, files, export
└─────────────────────────────────────────┘
```

| Layer       | Technology          | Role                                  |
| ----------- | ------------------- | ------------------------------------- |
| Frontend    | Svelte + TypeScript | UI, editor, navigation, state         |
| Framework   | Tauri v2            | Native shell, IPC, system access      |
| Backend     | Rust                | SQLite, file I/O, PDF/DOCX generation |
| Database    | SQLite              | Local persistence                     |
| Text Editor | TipTap              | Rich text editing                     |
| Spell-check | Native OS           | System dictionaries                   |

**Why Tauri**: lightweight (~15 MB), native webview, Rust performance, single codebase.

**Why Svelte**: reactive, minimal boilerplate, compiles to vanilla JS, fast.

### 24.3 Performance

| Metric                        | Target               |
| ----------------------------- | -------------------- |
| Launch                        | < 2 seconds          |
| Scene switch                  | < 100ms              |
| Search                        | < 200ms              |
| Autosave                      | < 500ms (background) |
| Large manuscript (150k words) | No degradation       |

---

## 25. Prioritization

### MVP

**Writing**:

- Project with chapters and scenes
- Scene attributes (title, summary, text, status, POV, tags, notes)
- Autosave, undo/redo
- Writing mode with focus features

**Organization**:

- Outline with drag-and-drop
- Corkboard view
- Quick Open
- Basic filters

**Bible**:

- All entry types
- Basic attributes (name, aliases, description, status, notes)
- Canonical associations (manual)

**Search**:

- Global search
- Find & Replace

**Export**:

- Reading export (PDF, DOCX)
- Working export (JSON)

### Post-MVP

**Timeline**:

- Events with scene links
- Dual view (chronological vs narrative)
- Conflict detection

**Templates**:

- Built-in templates (4)
- Step assignment
- Template editing

**Bible enhanced**:

- Detected mentions
- Typed relationships
- Custom fields

**Arcs**:

- Arc management
- Scene-arc linking
- Arc visualization

**Revision**:

- Text annotations
- Review grid
- Issues view

**History**:

- Snapshots
- Scene history
- Cut library

**Polish**:

- Name registry
- Import (DOCX, MD)
- Saved filter collections
- Keyboard shortcut customization
- Dark mode

---

## 26. Success Criteria

The application is complete when a writer can:

- [ ] Start writing immediately without setup
- [ ] Organize scenes in chapters with drag-and-drop
- [ ] Track characters, locations, objects, factions in the Bible
- [ ] Link Bible entries to scenes (canonical associations)
- [ ] See timeline conflicts and consistency issues
- [ ] Use narrative templates to structure the story
- [ ] Track multiple plot arcs
- [ ] Annotate text for revision
- [ ] Review scenes systematically
- [ ] Search across all content
- [ ] Find and replace text
- [ ] Export a clean manuscript for reading
- [ ] Export a full backup for safety
- [ ] Never lose work (autosave, crash recovery)
- [ ] Work entirely offline

---

## Glossary

| Term                  | Definition                                                                |
| --------------------- | ------------------------------------------------------------------------- |
| Canonical association | User-confirmed link between scene and Bible entry                         |
| Detected mention      | System-suggested link based on name match                                 |
| Evaluable scene       | Scene with enough data (character, location, time) for consistency checks |
| Issue                 | Consistency problem to resolve                                            |
| Arc                   | Plot thread spanning multiple scenes                                      |
| Snapshot              | Point-in-time backup of entire project                                    |
| Bible                 | Knowledge base of story world                                             |
| Template              | Narrative structure (e.g., Three-Act, Save the Cat)                       |
| Step                  | Single beat within a template                                             |
