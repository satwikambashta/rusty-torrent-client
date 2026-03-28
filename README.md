# Rusty Torrents - README

A powerful, modern desktop Rusty Torrents built with **Rust**, **Tauri**, and **React**.

## Features

### Current Version (0.1.0)
- ✅ Cross-platform desktop application (macOS, Windows, Linux)
- ✅ Modern React UI with responsive design
- ✅ Backend-Frontend IPC communication
- ✅ Test connectivity verification
- ✅ Component-based architecture
- ✅ Type-safe with TypeScript and Rust

### Planned Features
- 🔄 Torrent parsing and metadata extraction
- 🔄 DHT (Distributed Hash Table) support
- 🔄 Peer discovery and connection management
- 🔄 Multi-file torrent support
- 🔄 Download progress tracking
- 🔄 Upload/seeding capabilities
- 🔄 Configurable speed limits
- 🔄 Advanced search and filtering

## Quick Start

### Prerequisites
- Node.js 16+ (npm 7+)
- Rust 1.70+

### Installation

1. **Clone or navigate to the project**:
```bash
cd /path/to/tauri-app
```

2. **Install dependencies**:
```bash
npm install
cd src-tauri && cargo build && cd ..
```

3. **Start development**:
```bash
npm run dev
```

4. **Test connectivity**:
   - Open the "Test Connection" tab in the app
   - Click "Test Connection" button
   - Verify the response

## Project Structure

```
tauri-app/
├── src/                    # React frontend
│   ├── components/         # Reusable UI components
│   ├── pages/             # Page components
│   ├── services/          # API & state management
│   ├── hooks/             # Custom React hooks
│   ├── types/             # TypeScript definitions
│   ├── App.tsx            # Root component
│   └── main.tsx           # Entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── modules/       # Core business logic
│   │   ├── commands/      # IPC handlers
│   │   ├── lib.rs         # Initialization
│   │   └── main.rs        # Entry point
│   └── Cargo.toml         # Rust dependencies
├── package.json           # npm dependencies
├── PROJECT_STRUCTURE.md   # Detailed structure guide
├── TEST_GUIDE.md          # Testing instructions
├── DEVELOPMENT.md         # Development guidelines
└── README.md              # This file
```

## Available Commands

### Frontend
```bash
npm run dev          # Start development server with hot reload
npm run build        # Build optimized production version
npm run preview      # Preview production build
npm run type-check   # Check TypeScript errors
npm run tauri dev    # Run with Tauri dev tools
```

### Backend (from `src-tauri/`)
```bash
cargo build          # Build debug version
cargo build --release # Build optimized release
cargo test           # Run tests
cargo check          # Check for compilation errors
cargo clean          # Clean build artifacts
```

## Architecture

### Frontend Stack
- **React 19**: UI library
- **React Router v7**: Routing
- **Zustand**: State management
- **TypeScript**: Type safety
- **Vite**: Build tool

### Backend Stack
- **Tauri**: Desktop framework
- **Rust**: Systems programming
- **Tokio**: Async runtime
- **Serde**: Serialization

### Communication
```
React Component
    ↓
API Service (invoke)
    ↓
Tauri IPC Bridge
    ↓
Rust Command Handler
    ↓
Business Logic Module
```

## Testing

### Backend-Frontend Connection
1. Navigate to the "Test Connection" tab
2. Click "Test Connection" button
3. View response in the results panel

See [TEST_GUIDE.md](./TEST_GUIDE.md) for detailed testing instructions.

## Development

See [DEVELOPMENT.md](./DEVELOPMENT.md) for:
- Adding new components
- Creating new pages
- Adding Tauri commands
- State management patterns
- Error handling
- Performance tips

## Building for Release

### macOS
```bash
npm run build
# Output: src-tauri/target/release/bundle/macos/
```

### Windows
```bash
npm run build
# Output: src-tauri/target/release/
```

### Linux
```bash
npm run build
# Output: src-tauri/target/release/bundle/deb/
```

## Documentation

- [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md) - Detailed project organization
- [TEST_GUIDE.md](./TEST_GUIDE.md) - Backend connectivity testing
- [DEVELOPMENT.md](./DEVELOPMENT.md) - Development guidelines and best practices

## Troubleshooting

### Build Errors
```bash
# Clean and rebuild
cargo clean
npm run build
```

### Connection Test Fails
1. Check browser console (F12)
2. Verify backend built: `cd src-tauri && cargo check`
3. Restart dev server: `npm run dev`

### Module Import Errors
```bash
# Reinstall dependencies
rm -rf node_modules
npm install
```

## Key Features Explained

### IPC Communication
The app uses Tauri's IPC (Inter-Process Communication) to securely communicate between the React frontend and Rust backend.

### Module Architecture
- **config.rs**: Configuration settings
- **torrent.rs**: Torrent session management
- **test.rs**: Connectivity testing (frontend & backend)
- **torrent.rs (commands)**: Torrent operation endpoints

### State Management
Global state is managed with Zustand, providing:
- Connection status
- Torrent list
- Loading states
- Error messaging

## Implementation Roadmap

This section outlines the development plan for the Rusty Torrents, organized by priority and complexity.

### Phase 1: Foundation (Current)
- ✅ Project structure and architecture
- ✅ Frontend-backend IPC communication
- ✅ Connection testing framework
- ⏳ Unit tests for core modules

**Estimated Duration**: 1-2 weeks
**Dependencies**: None

### Phase 2: Torrent File Parsing
- ⏳ Bencode parser implementation
- ⏳ Torrent file metadata extraction
- ⏳ Info hash calculation (SHA-1)
- ⏳ File list parsing and validation

**Estimated Duration**: 2-3 weeks
**Dependencies**: Phase 1
**Key Files**: `src-tauri/src/modules/torrent.rs`, `src-tauri/src/modules/parser.rs`

### Phase 3: DHT & Peer Discovery
- ⏳ DHT (Distributed Hash Table) client implementation
- ⏳ Peer discovery via DHT
- ⏳ Announce to trackers (HTTP/UDP)
- ⏳ Peer connection pool management

**Estimated Duration**: 3-4 weeks
**Dependencies**: Phase 2
**Key Files**: `src-tauri/src/modules/dht.rs`, `src-tauri/src/modules/tracker.rs`

### Phase 4: Download Engine
- ⏳ Piece selection algorithm
- ⏳ Peer communication protocol (BitTorrent Protocol)
- ⏳ Concurrent piece downloading
- ⏳ Download progress tracking
- ⏳ Local file I/O and storage management

**Estimated Duration**: 3-4 weeks
**Dependencies**: Phase 3
**Key Files**: `src-tauri/src/modules/download.rs`, `src-tauri/src/modules/peer.rs`

### Phase 5: UI Enhancements
- ⏳ Real-time progress updates via Tauri events
- ⏳ Torrent adding workflow (file dialog)
- ⏳ Download speed graphs
- ⏳ Peer information display
- ⏳ Advanced filtering and sorting

**Estimated Duration**: 2-3 weeks
**Dependencies**: Phase 4
**Key Files**: `src/pages/HomePage.tsx`, `src/components/` (new)

### Phase 6: Seeding & Upload
- ⏳ Seeding functionality
- ⏳ Upload rate limiting
- ⏳ Peer serving from local storage
- ⏳ Choking algorithm implementation
- ⏳ Seed statistics tracking

**Estimated Duration**: 2-3 weeks
**Dependencies**: Phase 4
**Key Files**: `src-tauri/src/modules/seeder.rs`

### Phase 7: Configuration & Settings
- ⏳ Settings UI page
- ⏳ Configuration file management
- ⏳ Download directory selection
- ⏳ Speed limit configuration
- ⏳ Persistence management

**Estimated Duration**: 1-2 weeks
**Dependencies**: Phase 2
**Key Files**: `src/pages/SettingsPage.tsx`, Save/load in `modules/config.rs`

### Phase 8: Testing & Polish
- ⏳ Integration tests
- ⏳ Performance benchmarking
- ⏳ Error recovery mechanisms
- ⏳ User experience improvements
- ⏳ Documentation updates

**Estimated Duration**: 2-3 weeks
**Dependencies**: All previous phases

### Phase 9: Release Preparation
- ⏳ Cross-platform build optimization
- ⏳ Auto-update mechanism
- ⏳ Release notes and versioning
- ⏳ Installer creation (Windows/macOS/Linux)
- ⏳ Security audit

**Estimated Duration**: 2 weeks
**Dependencies**: Phase 8

### Quick Wins (Can be done anytime)
- 📝 Add keyboard shortcuts
- 📝 Implement clipboard torrent URL handling
- 📝 Add dark mode
- 📝 Create system tray integration
- 📝 Add torrent search integration
- 📝 Implement paused torrents resume

### Total Estimated Timeline
- **Minimum Viable Product (MVP)**: Phases 1-4 = 9-13 weeks
- **Feature Complete**: Phases 1-7 = 13-18 weeks
- **Production Ready**: Phases 1-9 = 17-25 weeks

### Dependencies & Tech Stack Requirements

**Already Included**:
- ✅ Tauri 2.0 (IPC & window management)
- ✅ React 19 with Router (UI)
- ✅ Tokio (async runtime)
- ✅ Serde (serialization)
- ✅ SHA-1 hashing

**To Add Later**:
- 🔄 `nom` or `winnow` (parsing library if bencode insufficient)
- 🔄 `quinn` (QUIC protocol for faster transfers)
- 🔄 `reqwest` (HTTP client for trackers)
- 🔄 Socket2 (UDP for DHT)
- 🔄 Ring or `rustls` (TLS for secure connections)

## Contributing

To extend the application:

1. ✅ Check the roadmap above for planned work
2. ✅ Add new components in `src/components/`
3. ✅ Add new pages in `src/pages/`
4. ✅ Extend Rust modules in `src-tauri/src/modules/`
5. ✅ Add new commands in `src-tauri/src/commands/`
6. ✅ Update documentation and roadmap

### Development Guidelines
- Follow the coding standards in [DEVELOPMENT.md](./DEVELOPMENT.md)
- Keep commits atomic and well-documented
- Test changes with the `/test` page before committing
- Update comments with area-wise explanations (not line-by-line)
- Follow Rust naming conventions and style


## Performance

- React components are optimized with proper memoization
- Backend uses async/await with Tokio for concurrent operations
- Minimal overhead for IPC communication
- Efficient state management with Zustand

## Security

- All backend operations are behind Tauri's secured commands
- Type-safe Rust prevents memory issues
- Frontend is sandboxed within Tauri framework
- No network access without explicit commands

## Browser Support

This is a desktop application built with Tauri. It works on:
- ✅ macOS 10.13+
- ✅ Windows 7+
- ✅ Linux (Ubuntu 16.04+, Fedora 26+, etc.)

## License

MIT License - See LICENSE file for details

## Support

- 📧 Issues: Check the issue tracker
- 📖 Docs: See PROJECT_STRUCTURE.md, TEST_GUIDE.md, DEVELOPMENT.md
- 🔗 Tauri: https://tauri.app/
- 🔗 React: https://react.dev/

---

**Happy coding!** 🚀

Start with the test page to verify everything is working, then dive into development!

