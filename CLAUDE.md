# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development
pnpm tauri dev              # Run full app with hot reload (frontend + backend)

# Testing
pnpm test                   # Frontend unit tests (Vitest)
pnpm test:watch             # Frontend tests in watch mode
pnpm test:all               # Frontend + Rust tests
cargo test --manifest-path src-tauri/Cargo.toml  # Rust tests only
pnpm test:e2e               # E2E tests (builds app first, requires tauri-driver)

# Code Quality
pnpm check                  # TypeScript/Svelte type checking
pnpm lint                   # ESLint (TypeScript + Svelte)
pnpm lint:fix               # ESLint with auto-fix
pnpm format                 # Prettier formatting
pnpm format:check           # Check Prettier formatting
cargo fmt                   # Format Rust code
cargo clippy                # Lint Rust code

# Build
pnpm tauri build            # Production binary → src-tauri/target/release/
```

## Architecture

Cahnon is a Tauri v2 desktop app: **Svelte 5 frontend** ↔ **Tauri IPC** ↔ **Rust backend** ↔ **SQLite**.

```
UI Component → Svelte Store → API Layer (invoke) → Tauri IPC → Rust Command → SQLite
```

### Frontend (`src/`)

- **`lib/api/`** - TypeScript types and Tauri invoke wrappers, split into domain modules (project, bible, timeline, health, etc.) with `index.ts` re-exporting all APIs
- **`lib/stores/`** - Svelte 5 runes-based `AppState` class (`app-state.svelte.ts`) + types, recovery utilities
- **`lib/components/`** - 39 Svelte components (Layout, Editor, Outline, Corkboard, BibleView, Dashboard, etc.) + `ui/` subdir with 12 reusable primitives (Button, Icon, Dialog, EmptyState, etc.)

### Backend (`src-tauri/src/`)

- **`lib.rs`** - AppState, plugin setup, command registration via `generate_handler![]`
- **`models.rs`** - All data structures and request/response types
- **`database.rs`** - SQLite operations (schema init, migrations, CRUD)
- **`validation.rs`** - Input validation
- **`commands/`** - 20 modules: project, chapter, scene, bible, arc, event, export, export_csv, annotation, association, cut, history, import, issue, relationship, search, snapshot, template, trash, analytics

### Data Model

Projects are single `.cahnon` SQLite files containing:

- Chapters → Scenes (with text, status, POV, tags, timeline info, cached word count)
- Bible entries (character, location, object, faction, concept, glossary)
- Arcs, Events, Templates, Snapshots, Annotations
- Facts, Writing Sessions, Name Registry, Cuts, Issues, Saved Filters
- N:M relationships: Scene↔BibleEntry, Scene↔Arc, Scene↔Event, BibleEntry↔BibleEntry, Arc↔Character, Issue↔Scene, Issue↔BibleEntry

## Key Patterns

### Adding a Full-Stack Feature

1. Define data model in `src-tauri/src/models.rs`
2. Add database operations in `src-tauri/src/database.rs`
3. Create Tauri commands in `src-tauri/src/commands/`
4. Register commands in `src-tauri/src/lib.rs` (`generate_handler![]`)
5. Add TypeScript types in `src/lib/api/types/index.ts` and API wrapper in `src/lib/api/` (create a domain module or add to existing one, re-export from `index.ts`)
6. Update stores if needed in `src/lib/stores/app-state.svelte.ts`
7. Build UI components in `src/lib/components/`

### State Management

Components never call APIs directly. Use appState:

```typescript
import { appState } from '$lib/stores';

// In components, access state directly (reactive via Svelte 5 runes)
// appState.selectedScene is automatically reactive in templates

await appState.updateScene(id, { text: '...' });

// Toggle UI state
appState.toggleOutline();

// Set values directly
appState.selectedSceneId = sceneId;
```

### Tauri Commands

```rust
#[tauri::command]
pub fn update_scene(state: State<'_, AppState>, id: String, request: UpdateSceneRequest) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_scene(&id, &request)
}
```

### API Layer

```typescript
export const sceneApi = {
	update: (id: string, request: Partial<Scene>) => invoke<Scene>('update_scene', { id, request }),
};
```

## Code Standards

- **Rust**: Format with `cargo fmt`, lint with `cargo clippy`, use `Result<T, String>` for commands
- **TypeScript**: Run `pnpm check` and `pnpm lint`, avoid `any` types
- **Svelte**: Add comment block at top of complex components describing purpose/features
- **Formatting**: Run `pnpm format` to apply Prettier formatting
- **Commits**: Use conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`)

## Documentation

- **[docs/Cahnon_Functional_Specification.md](docs/Cahnon_Functional_Specification.md)** - Complete feature specification
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Technical architecture details
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development setup and contribution guidelines
