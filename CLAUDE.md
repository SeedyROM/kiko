# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Starting the Application
- `./bin/dev` - Start both frontend and backend in development mode with hot reload
- `./bin/dev --release` - Start both services in release mode
- `./bin/dev -be=http://0.0.0.0:3030 -fe=http://0.0.0.0:8080` - Custom addresses
- Backend runs on port 3030, frontend on port 8080

### Prerequisites
- Rust 1.85.0+ with `cargo-watch` and `trunk` installed
- WASM target: `rustup target add wasm32-unknown-unknown`
- Node.js v20.0.0+ for `npx` and TailwindCSS: `npm i -G tailwindcss`

### Building and Testing
- `cargo doc --no-deps --document-private-items` - Generate documentation for workspace crates
- `cargo doc --no-deps --document-private-items --open` - Generate and open docs
- `./bin/e2e` - Run end-to-end tests headlessly
- `./bin/e2e --headed` - Run E2E tests with browser visible
- `./bin/e2e --debug` - Run E2E tests in debug mode
- `./bin/e2e --ui` - Run E2E tests in interactive UI mode
- `./bin/e2e --show-report` - Show existing test report in browser

### Individual Services
- Backend: `cd crates/kiko-backend && cargo run` (dev) or `cargo run --release`
- Frontend: `cd crates/kiko-frontend && trunk serve` (dev) or `trunk serve --release`

## Architecture

### Workspace Structure
This is a Rust workspace with three main crates:
- `kiko` - Shared library with common types, APIs, and utilities
- `kiko-backend` - Axum-based REST API and WebSocket server
- `kiko-frontend` - Yew-based WebAssembly frontend

### Core Architecture Pattern
- **Shared Types**: All data structures and message types are defined in the `kiko` crate and shared between frontend and backend
- **Real-time Communication**: WebSocket-based messaging system for live session updates
- **Session Management**: In-memory session storage with participant tracking and pointing

### Key Components

#### Backend (Axum + WebSockets)
- **AppState**: Shared application state containing SessionService and PubSub messaging
- **REST API**: `/api/v1/` endpoints for session CRUD operations
- **WebSocket**: `/api/v1/ws` endpoint for real-time session updates
- **Services**: `SessionServiceInMemory` for session storage, `PubSub` for message broadcasting
- **Handlers**: Organized by version (`v1/`) with health, session, and websocket modules

#### Frontend (Yew + WebAssembly)
- **Yew Components**: React-like component system with hooks
- **Router**: Client-side routing with `yew-router`
- **WebSocket Hooks**: Custom `use_websocket` hook for real-time communication
- **Pages**: `home.rs` for session creation, `session.rs` for session management
- **API Provider**: HTTP client for REST API communication

#### Shared Library (`kiko`)
- **Data Types**: `Session`, `Participant`, message enums in `data.rs`
- **IDs**: Type-safe ID system with `SessionId`, `ParticipantId`
- **API**: Request/response types for REST endpoints
- **Errors**: Centralized error handling with `Report` type
- **Logging**: Unified logging setup for both frontend and backend

### Data Flow
1. Sessions are created via REST API (`POST /api/v1/session`)
2. Participants join sessions via WebSocket connection
3. Real-time updates (pointing, topic changes) broadcast via PubSub system
4. Frontend components subscribe to WebSocket for live session state updates

### Build Process
- Frontend uses Trunk for WASM compilation with TailwindCSS integration
- Backend uses standard Cargo compilation
- Both services support development mode with hot reload via `cargo-watch` and `trunk serve`