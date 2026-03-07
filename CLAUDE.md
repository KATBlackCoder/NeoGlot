# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**NeoGlot** is a desktop application for translating open-source RPG games. It extracts text from game files, translates via local AI (Ollama), and re-injects translations into the original file format.

## Tech Stack

| Layer | Technology |
|---|---|
| UI | Vue 3 + TypeScript + Vite (`<script setup lang="ts">`) |
| Desktop | Tauri v2 (Rust backend) |
| UI components | shadcn-vue + Tailwind CSS v4 |
| Logic backend | Rust (Tauri commands) — parsing, SQLite, Ollama HTTP, sous-processus |
| AI | Ollama only (`localhost:11434`) via `ollama-rs` — no cloud APIs |
| Database | SQLite via `rusqlite` (Rust, bundled) — PAS de `tauri-plugin-sql` |

**Ollama is a hard requirement.** Verify at startup on `localhost:11434`. If not running, show a clear error with install link. No fallback to other AI services.

**No Python backend.** All logic runs in Rust (Tauri commands). The frontend communicates via `invoke()` IPC only — no HTTP server between layers.

## Supported Game Engines

1. RPG Maker MV/MZ — JSON (`rvpacker-txt-rs-lib` Rust crate)
2. RPG Maker XP/VX/VXAce — Ruby Marshal binary (`marshal-rs` + `rpgmad-lib`)
3. Wolf RPG Editor — binary `.dat`/`.mps` (WolfTL + UberWolf CLI tools via `std::process::Command`)
4. RPG Bakin — TBD (lowest priority)

## Project Structure

```
docs/          # Planning documents (architecture, DB schema, workflow, UI)
tasks/         # Ordered task backlog (T01 to T10)
reviews/       # Analysis of reference projects (00-SYNTHESE.md = master synthesis)
src/           # Vue 3 frontend
src-tauri/     # Rust/Tauri backend (parsing, SQLite, Ollama, Wolf RPG)
```

## Key Documents

- `docs/01-architecture.md` — full system architecture
- `docs/02-database-schema.md` — SQLite schema (6 tables)
- `docs/03-translation-workflow.md` — 13-step translation pipeline
- `docs/04-game-engines.md` — engine detection, formats, tools
- `docs/05-ui-structure.md` — UI wireframes and Vue 3 file structure
- `reviews/00-SYNTHESE.md` — master synthesis of all research
- `tasks/T01-install-plugins.md` — start here (plugins + shadcn setup)

## Development Rules

- Build and test **one module at a time** before moving to the next.
- **Never modify original game source files** — always work on copies (`work_path`).
- Code comments in **French**.
- Must be compatible with **Linux (CachyOS/Arch) and Windows**.
- Task order: T01 → T02 → T03 → T04 → T05 → T06 → T07 → T08 → T09 → T10.

## Commands

```bash
# Développement Tauri
pnpm tauri dev

# Build production
pnpm tauri build
```

## Skills Available

This project has agent skills pre-installed in `.agents/skills/`:
- `tauri-v2` — Tauri v2 IPC patterns, capabilities, Rust commands
- `vercel-react-best-practices` — React/performance best practices
- `shadcn` — shadcn/ui component usage

Use the relevant skill when working on those layers.
