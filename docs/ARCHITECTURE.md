# Cahnon Architecture

This document describes the technical architecture of Cahnon for developers and contributors.

## Overview

Cahnon is a desktop application built with Tauri v2, combining a Svelte frontend with a Rust backend.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri Shell                              │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Svelte Frontend                        │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │  │
│  │  │ Components  │  │   Stores    │  │   API Layer     │   │  │
│  │  │ (UI/Views)  │──│   (State)   │──│  (IPC Bridge)   │   │  │
│  │  └─────────────┘  └─────────────┘  └────────┬────────┘   │  │
│  └──────────────────────────────────────────────│────────────┘  │
│                                                 │                │
│                              Tauri IPC (invoke) │                │
│                                                 │                │
│  ┌──────────────────────────────────────────────│────────────┐  │
│  │                    Rust Backend              │            │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌────────▼────────┐   │  │
│  │  │  Database   │──│   Models    │──│    Commands     │   │  │
│  │  │  (SQLite)   │  │   (Data)    │  │   (Handlers)    │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Technology Stack

| Layer       | Technology            | Purpose                           |
| ----------- | --------------------- | --------------------------------- |
| Frontend    | Svelte 5 + TypeScript | Reactive UI components            |
| Framework   | Tauri v2              | Native shell, IPC, system dialogs |
| Backend     | Rust                  | Business logic, file I/O          |
| Database    | SQLite (rusqlite)     | Local data persistence            |
| Text Editor | TipTap + ProseMirror  | Rich text editing                 |
| Export      | docx, pdf-lib         | Document generation               |

## Directory Structure

```
cahnon/
├── src/                           # Frontend
│   ├── lib/
│   │   ├── api/
│   │   │   └── index.ts          # TypeScript types + Tauri invoke wrappers
│   │   ├── components/
│   │   │   ├── Layout.svelte     # Main app container
│   │   │   ├── Editor.svelte     # TipTap text editor
│   │   │   ├── Outline.svelte    # Chapter/scene tree
│   │   │   ├── Corkboard.svelte  # Card grid view
│   │   │   ├── BibleView.svelte  # Bible browser
│   │   │   ├── TimelineView.svelte
│   │   │   ├── ContextPanel.svelte
│   │   │   └── ...               # 30 components total
│   │   ├── stores/
│   │   │   └── index.ts          # Svelte stores + actions
│   │   └── utils/
│   │       └── index.ts          # Helper functions
│   └── routes/
│       └── +page.svelte          # Entry point
│
├── src-tauri/                     # Backend
│   ├── src/
│   │   ├── lib.rs                # App entry, command registration
│   │   ├── models.rs             # Data structures
│   │   ├── database.rs           # SQLite operations
│   │   ├── validation.rs         # Input validation
│   │   └── commands/             # Tauri command handlers
│   │       ├── project.rs        # Project CRUD, file locking
│   │       ├── chapter.rs        # Chapter management
│   │       ├── scene.rs          # Scene CRUD, split/merge
│   │       ├── bible.rs          # Bible entries
│   │       ├── association.rs    # Scene-Bible links
│   │       ├── arc.rs            # Plot arcs
│   │       ├── event.rs          # Timeline events
│   │       ├── export.rs         # Export handlers
│   │       └── ...               # 22 modules total
│   └── Cargo.toml
│
├── docs/
│   ├── Cahnon_Functional_Specification.md
│   └── ARCHITECTURE.md           # This file
│
└── tests/
    └── e2e/                      # WebdriverIO tests
```

## Data Flow

### Read operation

```
1. User clicks scene in Outline
2. Outline.svelte dispatches action
3. Store calls sceneApi.get(id)
4. API invokes Tauri command 'get_scene'
5. Rust command queries SQLite
6. Data returned through IPC
7. Store updates selectedScene
8. Editor.svelte reactively renders
```

### Write operation

```
1. User types in Editor
2. Editor debounces input (2s)
3. Editor calls actions.updateScene()
4. Store calls sceneApi.update()
5. API invokes 'update_scene' command
6. Rust validates input
7. SQLite UPDATE executed
8. Updated scene returned
9. Store updates scenes map
10. UI reflects changes
```

## Data Model

```
Project (1 per .cahnon file)
├── Chapters [ordered]
│   └── Scenes [ordered]
│       ├── Text (TipTap HTML)
│       ├── Metadata (status, POV, tags)
│       ├── Timeline info
│       └── Revision fields
├── Bible Entries
│   ├── Characters
│   ├── Locations
│   ├── Objects
│   ├── Factions
│   ├── Concepts
│   └── Glossary terms
├── Arcs (plot threads)
├── Events (timeline)
├── Templates (narrative structures)
├── Snapshots (point-in-time backups)
└── Settings (word targets, etc.)

Relationships (N:M):
- Scene ↔ Bible Entry (canonical associations)
- Scene ↔ Arc (arc membership)
- Scene ↔ Event (timeline links)
- Bible Entry ↔ Bible Entry (relationships)
```

## Key Patterns

### State Management

All application state lives in a class-based store using Svelte 5 runes (`src/lib/stores/app-state.svelte.ts`):

```typescript
class AppState {
  // Reactive state with $state
  project = $state<Project | null>(null);
  chapters = $state<Chapter[]>([]);
  scenes = new SvelteMap<string, Scene[]>();
  selectedSceneId = $state<string | null>(null);

  // Derived values via getters
  get selectedScene(): Scene | null {
    if (!this.selectedSceneId) return null;
    for (const sceneList of this.scenes.values()) {
      const found = sceneList.find(s => s.id === this.selectedSceneId);
      if (found) return found;
    }
    return null;
  }

  // Actions as methods
  async updateScene(id: string, data: Partial<Scene>) { ... }
  toggleOutline() { ... }
}

export const appState = new AppState();
```

Components access state directly via `appState`. No direct API calls from components.

### API Layer

TypeScript wrappers provide type safety over Tauri IPC:

```typescript
// src/lib/api/index.ts
export const sceneApi = {
	get: (id: string) => invoke<Scene>('get_scene', { id }),

	update: (id: string, request: Partial<Scene>) => invoke<Scene>('update_scene', { id, request }),
};
```

### Tauri Commands

Each command is a Rust function decorated with `#[tauri::command]`:

```rust
// src-tauri/src/commands/scene.rs
#[tauri::command]
pub fn update_scene(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSceneRequest,
) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_scene(&id, request)
}
```

Commands are registered in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::scene::update_scene,
    // ... 75+ commands
])
```

### Database Layer

`Database` wraps rusqlite with schema management:

```rust
// src-tauri/src/database.rs
impl Database {
    pub fn create(path: &Path) -> Result<Self, String> { ... }
    pub fn open(path: &Path) -> Result<Self, String> { ... }

    pub fn create_scene(&self, req: CreateSceneRequest) -> Result<Scene, String> { ... }
    pub fn update_scene(&self, id: &str, req: UpdateSceneRequest) -> Result<Scene, String> { ... }
}
```

Schema is initialized on project creation and migrated on open.

### Component Architecture

Components follow a consistent Svelte 5 pattern with runes:

```svelte
<!--
  Brief description of component purpose.
  Key features and behaviors.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { appState } from '$lib/stores';

	// Props with $props() rune
	interface Props {
		sceneId: string;
		onSave?: () => void;
	}
	let { sceneId, onSave }: Props = $props();

	// Derived values from appState for proper reactivity in templates
	// Use $derived to wrap frequently accessed appState properties
	let selectedScene = $derived(appState.selectedScene);
	let chapters = $derived(appState.chapters);

	// Local state with $state
	let isEditing = $state(false);

	// Computed values with $derived (use 90% of the time)
	let scene = $derived(appState.scenes.get(sceneId));
	let wordCount = $derived(scene?.text?.split(/\s+/).length ?? 0);

	// For complex derived computations, use $derived.by()
	let filteredItems = $derived.by(() => {
		if (!searchQuery) return [];
		return items.filter((item) => item.name.includes(searchQuery));
	});

	// Side effects with $effect - ONLY for side effects (API calls, DOM manipulation)
	// DON'T use $effect to synchronize state - use $derived instead
	$effect(() => {
		if (selectedScene?.id) {
			loadRelatedData(selectedScene.id); // API call = side effect
		}
	});

	// One-time initialization: use onMount instead of $effect
	onMount(() => {
		loadInitialData();
	});

	// Event handlers - call appState methods directly
	function handleSave() {
		appState.updateScene(sceneId, { text: newText });
		onSave?.();
	}
</script>

<!-- Event handlers use native attributes: onclick, onchange, etc. -->
<div class="component-name">
	<button onclick={handleSave}>Save</button>
</div>

<!-- Use derived values in templates for proper reactivity -->
{#if selectedScene}
	<h1>{selectedScene.title}</h1>
{/if}

<style>
	/* Scoped styles */
</style>
```

**Key Svelte 5 Patterns:**

| Pattern               | When to Use                                                   |
| --------------------- | ------------------------------------------------------------- |
| `$state()`            | Mutable local state that changes over time                    |
| `$derived()`          | Computed values from other state (90% of use cases)           |
| `$derived.by()`       | Complex derived computations with logic                       |
| `$effect()`           | Side effects only: API calls, DOM manipulation, subscriptions |
| `onMount()`           | One-time initialization when component mounts                 |
| `SvelteMap/SvelteSet` | Reactive Map/Set collections                                  |

**Anti-patterns to Avoid:**

- Don't use `$effect` to synchronize state → use `$derived` instead
- Don't access `appState.selectedScene` directly in templates without `$derived` wrapper
- Don't use empty `$effect(() => { loadData() })` → use `onMount()` instead

## Error Handling

- Rust commands return `Result<T, String>`
- Frontend catches errors and displays via toast notifications
- Validation happens at the Rust layer before database operations
- User-facing errors are descriptive; internal errors are logged

## Testing

| Type        | Tool        | Location                 |
| ----------- | ----------- | ------------------------ |
| Unit (TS)   | Vitest      | `src/lib/**/*.test.ts`   |
| Unit (Rust) | cargo test  | `src-tauri/src/tests.rs` |
| E2E         | WebdriverIO | `tests/e2e/*.spec.ts`    |

## Performance Considerations

- Scenes are loaded per-chapter to avoid loading entire manuscript
- Editor debounces saves (2 seconds)
- Word counts are calculated on-demand, not stored
- SQLite indexes on foreign keys and frequently queried columns
- TipTap document is serialized to HTML only on save
