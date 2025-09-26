# 🫵 Kiko

[![GitHub](https://img.shields.io/badge/github-kiko-8da0cb?logo=GitHub)](https://github.com/SeedyROM/kiko)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A simple but elegant pointing poker app for agile teams
to estimate user stories and tasks.

## 📚 Meaning of Kiko
Kiko is a Hawaiian word meaning "dot," "point," or "spot" - referring to small marks or punctuation. In the context of this app, it refers to marking estimates for user stories and tasks during agile planning sessions.

## 👨‍💻 Development

### 📓 Prerequisite Dependencies
- [`rust`](https://www.rust-lang.org/tools/install) **(v1.88.0 or later)**
    - [`rustup`](https://rustup.rs/)
    - [`cargo-watch`](https://crates.io/crates/cargo-watch)
    - [`trunk`](https://crates.io/crates/trunk)
    - Install the wasm target `rustup target add wasm32-unknown-unknown`

- [`node`](https://nodejs.org/en/download/) **(v20.0.0 or later) for `npx`**
    - Install tailwind globally `npm i -g tailwindcss`

### 🏃‍♂️ Running Locally

#### `./bin/dev (--release)`
- This runs the entire app in development or release mode
- The frontend is served at `http://localhost:8080` and the backend at `http://localhost:3030` respectively
- The frontend and backend are reloaded automatically when you make changes to the code
    - The frontend is built using `trunk` and the backend is built using `cargo watch`, read the documentation on each tool for more details

### 📖 Documentation

To generate documentation for all workspace crates without external dependencies:

```bash
cargo doc --no-deps --document-private-items
```

Add `--open` to automatically open the docs in your browser after generation.

### 🔧 Pre-commit Hooks

Pre-commit hooks are configured to run `cargo fmt` and `cargo clippy` automatically on each commit:

```bash
# Install pre-commit hooks (one-time setup)
pre-commit install

# Run hooks manually on all files
pre-commit run --all-files
```

The hooks will format code and check for linting issues, treating warnings as errors.

### 🐳 Docker Deployment

For production deployment with optimized builds and compression:

```bash
# Development mode
docker-compose up

# Production mode with resource limits
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up

# Build all images
docker-compose build

# Build individual services
docker-compose build frontend
docker-compose build backend
```

#### Building and Pushing Images

Use the convenient build scripts to create and push Docker images:

```bash
# Build images locally (defaults to 'sleazyrom' username)
./bin/build-docker [username]

# Push images to Docker Hub
./bin/push-docker [username]

# Build and push in one command
./bin/build-and-push [username]
```

Examples:
```bash
# Build with custom username
./bin/build-docker myusername

# Build and push with default username
./bin/build-and-push
```

#### Docker Architecture
- **Multi-stage builds**: Optimized Dockerfiles for both services
- **Frontend**: Rust 1.88 builder → Alpine runtime with nginx and brotli compression
- **Backend**: Rust 1.88 builder → Debian slim runtime
- **Compression**: Brotli compression provides 70-85% size reduction for frontend assets
- **Security**: Non-root containers with health checks
