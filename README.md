# 🫵 Kiko

A simple but elegant pointing poker app for agile teams
to estimate user stories and tasks.

## Development

### Prerequisite Dependencies
- [`rust`](https://www.rust-lang.org/tools/install) **(v1.85.0 or later)**
    - [`cargo-watch`](https://crates.io/crates/cargo-watch)
    - [`trunk`](https://crates.io/crates/trunk)
- [`node`](https://nodejs.org/en/download/) **(v20.0.0 or later) for `npx`**

### Running Locally

- `./bin/dev`
   - This runs the entire app in development mode.
   - The frontend is served on `localhost:8080` and the backend on `localhost:8000`.
   - The frontend and backend are reloaded automatically when you make changes to the code.
   - The frontend is built using `trunk` and the backend is built using `cargo watch`.
