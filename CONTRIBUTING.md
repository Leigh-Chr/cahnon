# Contributing to Cahnon

Thank you for your interest in contributing to Cahnon! This document provides guidelines for development setup, code standards, and the contribution process.

## Getting Started

### Development Setup

1. **Install prerequisites**
   - Node.js 18+
   - pnpm (`npm install -g pnpm`)
   - Rust toolchain via [rustup](https://rustup.rs/)

2. **Clone and install**

   ```bash
   git clone https://github.com/your-username/cahnon.git
   cd cahnon
   pnpm install
   ```

3. **Run in development mode**

   ```bash
   pnpm tauri dev
   ```

   This starts both the Vite dev server (frontend) and the Tauri app (backend).

4. **Run tests**
   ```bash
   pnpm test          # Frontend unit tests (Vitest)
   pnpm test:all      # Frontend + Rust tests
   pnpm test:e2e      # End-to-end tests (WebdriverIO)
   ```

### Project Structure

```
cahnon/
├── src/                        # Frontend (Svelte/TypeScript)
│   ├── lib/
│   │   ├── api/               # Tauri IPC wrappers, split into domain modules
│   │   │   ├── types/index.ts # Shared TypeScript types
│   │   │   └── *.ts           # Domain modules (9 total: project, bible, timeline, etc.)
│   │   ├── components/        # Svelte components (39 main + 12 ui = 51 total)
│   │   │   └── ui/            # Reusable UI primitives (Button, Icon, Dialog, etc.)
│   │   ├── stores/            # Svelte 5 runes-based state management
│   │   └── utils/             # Utility functions
│   └── routes/                # SvelteKit routes
├── src-tauri/                 # Backend (Rust)
│   ├── src/
│   │   ├── commands/          # Tauri command handlers (20 modules)
│   │   ├── database/           # SQLite operations (modular, split into submodules)
│   │   ├── models.rs          # Data structures
│   │   ├── validation.rs      # Input validation
│   │   └── lib.rs             # App entry and command registration
│   └── Cargo.toml
├── docs/                      # Documentation
└── tests/                     # E2E tests
```

## Code Standards

### Rust

- Format with `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Document public functions and types with doc comments
- Use `Result<T, String>` for Tauri command return types
- Keep commands focused on a single responsibility

```rust
/// Creates a new chapter in the manuscript.
///
/// Returns the created chapter with generated ID and timestamps.
#[tauri::command]
pub fn create_chapter(
    request: CreateChapterRequest,
    state: State<AppState>,
) -> Result<Chapter, String> {
    // ...
}
```

### TypeScript/Svelte

- Run `pnpm check` to verify TypeScript and Svelte
- Run `pnpm lint` to check for linting issues (ESLint)
- Run `pnpm format` to apply consistent formatting (Prettier)
- Use TypeScript strictly (no `any` types without justification)
- Prefer composition over inheritance
- Keep components focused and reusable

```typescript
// API wrapper pattern
export const chapterApi = {
	create: (title: string) => invoke<Chapter>('create_chapter', { request: { title } }),
};
```

### Svelte Components

- Add a comment block at the top of complex components explaining purpose and features
- Use typed props with TypeScript
- Keep template logic minimal; extract to functions when complex

```svelte
<!--
  Scene card for the corkboard view.
  Displays title, summary, status badge, and word count.
  Supports drag-and-drop reordering.
-->
<script lang="ts">
	// ...
</script>
```

### Commits

- Write clear, descriptive commit messages
- Use conventional commit style when appropriate:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation
  - `refactor:` for code restructuring
  - `test:` for test additions/changes

## Pull Requests

### Before submitting

1. **Run the full test suite**

   ```bash
   pnpm check && pnpm lint && pnpm test:all
   ```

2. **Format your code**

   ```bash
   pnpm format         # Format TypeScript/Svelte with Prettier
   cargo fmt           # Format Rust
   ```

3. **Test your changes manually** in the app

### PR Guidelines

- Keep PRs focused on a single feature or fix
- Update documentation if you change behavior
- Add tests for new functionality
- Reference any related issues

### Branch naming

- `feat/short-description` for features
- `fix/short-description` for bug fixes
- `docs/short-description` for documentation

## Reporting Issues

### Bug reports

When reporting bugs, please include:

- Steps to reproduce
- Expected behavior
- Actual behavior
- OS and version
- Cahnon version (or commit hash if building from source)
- Relevant error messages or logs

### Feature requests

For feature requests:

- Check the [Functional Specification](docs/Cahnon_Functional_Specification.md) first; it may already be planned
- Describe the use case and why this would be valuable
- Consider how it fits with Cahnon's principles (writing-first, non-prescriptive, local-only)

## Architecture Notes

### Data Flow

```
UI Component → Svelte Store → API Layer (invoke) → Tauri IPC → Rust Command → SQLite
```

All state mutations go through this flow. The frontend never accesses the database directly.

### Adding a new feature

1. **Define the data model** in `src-tauri/src/models.rs`
2. **Add database operations** in `src-tauri/src/database/` (create or extend a submodule)
3. **Create Tauri commands** in `src-tauri/src/commands/`
4. **Register commands** in `src-tauri/src/lib.rs`
5. **Add TypeScript types and API wrapper** in `src/lib/api/` (create or extend a domain module, re-export from `index.ts`)
6. **Update stores** if needed in `src/lib/stores/app-state.svelte.ts`
7. **Build the UI** in `src/lib/components/`

## Questions?

If you have questions about contributing, feel free to open an issue for discussion.
