# Project Structure

## Overview
This is a **Desktop Rusty Torrents** built with Rust, Tauri, and React. The project follows best practices for desktop application development with a clear separation of concerns between frontend and backend.

## Directory Structure

### Frontend (`src/`)
```
src/
├── components/          # Reusable React components
│   ├── ConnectionStatus.tsx     # Displays backend connection status
│   ├── ConnectionStatus.css
│   ├── Header.tsx               # App header with title
│   ├── Header.css
│   ├── TorrentItem.tsx          # Individual torrent list item
│   └── TorrentItem.css
├── pages/              # Page components (main views)
│   ├── HomePage.tsx            # Main downloads page
│   ├── HomePage.css
│   ├── TestPage.tsx            # Backend connectivity test page
│   └── TestPage.css
├── services/           # Business logic and API calls
│   ├── api.ts                  # Tauri IPC command wrappers
│   └── store.ts                # Zustand global state management
├── hooks/              # Custom React hooks
│   └── useConnectionTest.ts    # Initial connection test hook
├── types/              # TypeScript interfaces and types
│   └── index.ts                # Shared type definitions
├── styles/             # Global styles (if needed)
├── App.tsx             # Root component
├── App.css             # App layout styles
├── main.tsx            # React entry point
├── index.css           # Global styles
└── vite-env.d.ts       # Vite environment types
```

### Backend (`src-tauri/src/`)
```
src-tauri/
├── src/
│   ├── modules/               # Core application modules
│   │   ├── mod.rs             # Module declarations
│   │   ├── config.rs          # Configuration management
│   │   └── torrent.rs         # BitTorrent functionality
│   ├── commands/              # Tauri IPC command handlers
│   │   ├── mod.rs             # Command module declarations
│   │   ├── test.rs            # Connectivity test commands
│   │   └── torrent.rs         # Torrent operation commands
│   ├── main.rs                # Application entry point
│   └── lib.rs                 # Library initialization & command registration
├── Cargo.toml                 # Rust dependencies
├── build.rs                   # Build configuration
├── tauri.conf.json            # Tauri configuration
├── icons/                     # App icons
└── capabilities/              # Security capabilities
```

## Key Components

### Frontend
- **React Router**: Navigation between pages
- **Zustand**: Lightweight state management
- **Tauri API**: IPC communication with backend

### Backend
- **Tauri**: Desktop framework and IPC bridge
- **Tokio**: Async runtime for concurrent operations
- **Serde**: Serialization/deserialization

## Communication Pattern

Frontend → React Components → Services (api.ts) → Tauri invoke() → Rust Commands → Backend Logic

## Module Breakdown

### Components
- `ConnectionStatus`: Visual indicator of backend connection
- `Header`: Application header with title and status
- `TorrentItem`: Reusable component for displaying torrent information

### Pages
- `HomePage`: Displays list of active torrents with controls
- `TestPage`: Interactive test page to verify backend connectivity

### Services
- `api.ts`: Wrapper functions for all Tauri commands
- `store.ts`: Zustand store for application state

### Rust Modules
- `config`: Application configuration management
- `torrent`: Torrent session and download management

### Rust Commands
- `test_connection`: Verify backend is responding
- `get_server_info`: Retrieve server information
- `get_torrents`: Fetch active torrent sessions
- `add_torrent`: Add new torrent from file
- `start_torrent`: Start/resume download
- `pause_torrent`: Pause download
- `remove_torrent`: Remove torrent session

## Best Practices Implemented

1. **Separation of Concerns**: Frontend and backend are clearly separated
2. **Type Safety**: TypeScript on frontend, Rust on backend
3. **Modular Design**: Both frontend and backend use modular architecture
4. **State Management**: Centralized state with Zustand
5. **Error Handling**: Proper error handling in services and commands
6. **Component Reusability**: Components are designed to be reusable
7. **Responsive Design**: CSS media queries for mobile compatibility
8. **Documentation**: Inline comments and documentation
9. **Testing**: Dedicated test page for connectivity verification
