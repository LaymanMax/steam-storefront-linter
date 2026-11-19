# Local Development Guide

## Prerequisites

- **Rust 1.89+** (includes `cargo` and `rustc`)
- **Node.js 20+**
- **pnpm 9+** or **10+**

---

## Workspace Setup

Clone the repository and install frontend dependencies:

```bash
git clone https://github.com/LaymanMax/steam-storefront-linter.git
cd steam-storefront-linter

# Install Node dependencies
pnpm install
```

---

## Running Locally

### Option 1: Full-stack Development Mode

In one terminal, start the Rust API backend:

```bash
cargo run -p linter-server
# Server listens on http://127.0.0.1:3000
```

In a second terminal, start the Vite frontend development server:

```bash
pnpm --filter web dev
# Web app runs on http://localhost:5173 with auto-reload and proxy to :3000
```

### Option 2: Self-hosted Production Mode

Build the frontend bundle and start the Rust server (the server automatically serves `apps/web/dist` directly):

```bash
pnpm --filter web build
cargo run --release -p linter-server
# Open http://localhost:3000 in your browser
```

---

## CLI Development

Run CLI commands directly via Cargo:

```bash
# Check CLI help
cargo run -p linter-cli -- --help

# Initialize sample benchmark database
cargo run -p linter-cli -- dataset sample

# Audit a fixture
cargo run -p linter-cli -- audit --input data/fixtures/valid_indie_game.json

# Audit a live Steam AppID
cargo run -p linter-cli -- audit 2539430

# Export standalone HTML report
cargo run -p linter-cli -- audit --input data/fixtures/valid_indie_game.json --format html --output report.html
```

---

## Testing

Run unit and integration test suites:

```bash
# Run Rust unit & integration tests
cargo test --lib --workspace
cargo test --tests --workspace

# Run frontend Vitest tests
pnpm --filter web test
```

---

## Code Quality & Formatting

```bash
# Rust formatter
cargo fmt --check

# Rust linter
cargo clippy --workspace --tests

# Frontend typecheck
pnpm --filter web lint
```
