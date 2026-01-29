# Cahnon

**Write freely. Stay consistent.**

A desktop application for fiction writers to organize scenes, maintain story consistency, and export clean manuscripts. Built with Svelte and Tauri.

## About

Cahnon helps novelists and series writers manage complex narratives by combining a distraction-free writing environment with powerful organizational tools. Write scenes in any order, build your story bible, track timeline events, and let Cahnon help you maintain consistency across your work.

**Cahnon is modular and non-prescriptive**: start writing immediately, structure later.

## Features

- **Scene-based writing** - Organize your manuscript into chapters and scenes with drag-and-drop reordering
- **Story Bible** - Centralize characters, locations, objects, factions, and concepts with relationships and custom fields
- **Timeline tracking** - Place scenes and events chronologically, detect conflicts automatically
- **Plot arcs** - Track narrative threads across scenes with visual progress indicators
- **Corkboard view** - Visualize scenes as cards, filter by status/POV/arc, drag to reorganize
- **Revision tools** - Annotations, review grid, consistency issues, setup/payoff tracking
- **Focus modes** - Typewriter mode, dim surroundings, fullscreen for distraction-free writing
- **Version history** - Scene snapshots with comparison and restoration
- **Flexible export** - Markdown, plain text, PDF, DOCX, JSON backup
- **Local-only** - Your data stays on your machine in a single `.cahnon` file (SQLite)

## Installation

### Prerequisites

- Node.js 18+
- pnpm
- Rust toolchain (rustc, cargo)

### Build from source

```bash
# Clone the repository
git clone https://github.com/your-username/cahnon.git
cd cahnon

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

The built application will be in `src-tauri/target/release/`.

## Usage

1. **Create a project** - Choose a location for your `.cahnon` file
2. **Start writing** - Create chapters and scenes, or just start with a single scene
3. **Build your bible** - Add characters and locations as you introduce them
4. **Link scenes to bible entries** - Create canonical associations to track who/what appears where
5. **Use the corkboard** - Step back and see your story structure at a glance
6. **Review and revise** - Switch to revision mode for analytical tools

### Keyboard shortcuts

| Action               | Shortcut               |
| -------------------- | ---------------------- |
| Quick Open           | `Cmd/Ctrl + K`         |
| New Scene            | `Cmd/Ctrl + N`         |
| Save                 | `Cmd/Ctrl + S`         |
| Toggle Outline       | `Cmd/Ctrl + \`         |
| Toggle Context Panel | `Cmd/Ctrl + Shift + \` |
| Focus Mode           | `Cmd/Ctrl + Shift + F` |
| Find                 | `Cmd/Ctrl + F`         |
| Find & Replace       | `Cmd/Ctrl + H`         |

## Documentation

- [Functional Specification](docs/Cahnon_Functional_Specification.md) - Complete feature documentation
- [Architecture](docs/ARCHITECTURE.md) - Technical architecture and code organization
- [Contributing](CONTRIBUTING.md) - Development setup and contribution guidelines

## Technology

| Layer     | Technology           |
| --------- | -------------------- |
| Frontend  | Svelte 5, TypeScript |
| Framework | Tauri v2             |
| Backend   | Rust                 |
| Database  | SQLite               |
| Editor    | TipTap (ProseMirror) |

## License

GPL-3.0 - See [LICENSE](LICENSE) for details.
