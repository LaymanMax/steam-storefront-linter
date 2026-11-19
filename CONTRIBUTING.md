# Contributing to Steam Storefront Linter

We welcome contributions of all kinds to improve Steam Storefront Linter! Whether you are reporting bugs, improving documentation, expanding audit rules, or adding benchmark capabilities, here is how to get involved.

---

## Development Setup

1. **Fork and Clone the Repository**:

   ```bash
   git clone https://github.com/LaymanMax/steam-storefront-linter.git
   cd steam-storefront-linter
   ```

2. **Install Node & Rust Toolchains**:
   - Rust: `1.89+` (stable)
   - Node: `v20+` or `v22+`
   - Package manager: `pnpm` (`corepack enable && corepack prepare pnpm@latest --activate`)

3. **Install Dependencies & Seed Sample Benchmark DB**:
   ```bash
   pnpm install
   cargo run -p linter-cli -- dataset sample
   ```

---

## Contribution Workflow

1. **Create a Topic Branch**:

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Run Tests & Linters Locally**:
   Before opening a pull request, ensure all tests, formatting, and linters pass:

   ```bash
   # Check Rust formatting and clippy
   cargo fmt --check
   cargo clippy --workspace --tests

   # Run test suites
   cargo test --lib --workspace
   cargo test --tests --workspace
   pnpm --filter web test

   # Type-check and build frontend
   pnpm --filter web lint
   pnpm --filter web build
   ```

3. **Commit Your Changes**:
   Write clear, conventional commit messages:

   ```bash
   git commit -m "feat(rules): add missing audio language parity check"
   ```

4. **Submit a Pull Request**:
   Push your branch and open a PR against the `main` branch. Provide a concise summary of the motivation, changes, and any testing performed.

---

## Code Style & Architectural Principles

- **Deterministic Auditing**: All audit rules must be rule-based, deterministic, and versioned. Do not introduce LLM or non-deterministic external dependencies into core auditing logic.
- **Descriptive Benchmarking**: Never predict or imply commercial success, revenue, sales, or review scores. Completeness audits are purely descriptive.
- **Handle Missing Data Explicitly**: Never convert an unknown or missing field into zero. Preserve `known: false` and provenance tracking.
- **Shared Business Logic**: Implement domain and audit rules in `crates/linter-core` so both the CLI (`linter-cli`) and HTTP API (`linter-server`) share identical behavior.

---

## Reporting Issues & Security

- **Bug Reports & Feature Requests**: Please use [GitHub Issues](https://github.com/LaymanMax/steam-storefront-linter/issues) with detailed steps to reproduce.
- **Security Vulnerabilities**: Review [docs/security.md](./docs/security.md) for vulnerability disclosure procedures.
