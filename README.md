# 🫵 Kiko

[![GitHub](https://img.shields.io/badge/github-kiko-8da0cb?logo=GitHub)](https://github.com/SeedyROM/kiko)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

<!-- [![Tests](https://github.com/SeedyROM/kiko/actions/workflows/rust.yml/badge.svg)](https://github.com/SeedyROM/kiko/actions) -->

A simple but elegant pointing poker app for agile teams
to estimate user stories and tasks.

## 📚 Meaning of Kiko
Kiko is a Hawaiian word meaning "dot," "point," or "spot" - referring to small marks or punctuation. In the context of this app, it refers to marking estimates for user stories and tasks during agile planning sessions.

## 👨‍💻 Development

### 📓 Prerequisite Dependencies
- [`rust`](https://www.rust-lang.org/tools/install) **(v1.85.0 or later)**
    - [`cargo-watch`](https://crates.io/crates/cargo-watch)
    - [`trunk`](https://crates.io/crates/trunk)
- [`node`](https://nodejs.org/en/download/) **(v20.0.0 or later) for `npx`**

### 🏃‍♂️ Running Locally

- `./bin/dev`
   - This runs the entire app in development mode.
   - The frontend is served on `localhost:8080` and the backend on `localhost:8000`.
   - The frontend and backend are reloaded automatically when you make changes to the code.
   - The frontend is built using `trunk` and the backend is built using `cargo watch`.
