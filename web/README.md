# Rusty Torrents - Web Monitor

A lightweight, real-time web client for monitoring your torrent activity remotely.

## Features

- 🔴 **Real-time Monitoring**: Live updates of torrent status and statistics
- 📊 **Dashboard**: Overview of active torrents, upload/download speeds, and seeding stats
- 🌐 **Remote Access**: Access from any machine on your network
- ⚡ **Fast & Lightweight**: Built with React and Vite for optimal performance
- 🎨 **Professional UI**: Dark theme with consistent design
- 📱 **Responsive**: Works on desktop, tablet, and mobile devices

## Getting Started

### Prerequisites

- Node.js 16+ (npm 7+)
- Backend server running on `localhost:8080`

### Installation

```bash
# From the web directory
npm install
```

### Development

```bash
# Start web client on port 3000
npm run dev

# Or from the root directory
npm run web:dev

# Run both desktop app and web client
npm run dev:all
```

### Building

```bash
# Build for production
npm run build

# The output will be in the dist/ directory
```

## Architecture

### API Integration

The web client connects to the HTTP REST API running on port 8080:

```
Web Client (Port 3000)
    ↓
HTTP Proxy (Vite)
    ↓
REST API (Port 8080)
```

The proxy is configured in `vite.config.ts` to forward `/api` requests to the backend.

### Project Structure

```
web/
├── public/
│   └── index.html           # HTML entry point
├── src/
│   ├── components/
│   │   └── Header.tsx       # App header with connection status
│   ├── pages/
│   │   └── Dashboard.tsx    # Main monitoring dashboard
│   ├── services/
│   │   ├── api.ts          # HTTP API client
│   │   └── store.ts        # Zustand global state
│   ├── types/
│   │   └── index.ts        # TypeScript type definitions
│   ├── App.tsx             # Root component
│   ├── App.css             # Global styles
│   └── main.tsx            # React entry point
├── vite.config.ts          # Vite configuration (port 3000)
├── tsconfig.json           # TypeScript configuration
└── package.json            # Dependencies
```

## Services

### API Client (`services/api.ts`)

Type-safe HTTP client for backend communication:

```typescript
// Get current torrents
const torrents = await apiClient.getTorrents();

// Get statistics
const stats = await apiClient.getTorrentStats();

// Get seeding events
const events = await apiClient.getRecentSeedingEvents();

// Health check
const health = await apiClient.healthCheck();
```

### State Management (`services/store.ts`)

Zustand store for global state:

```typescript
const torrents = useAppStore((state) => state.torrents);
const isConnected = useAppStore((state) => state.isConnected);
const refreshInterval = useAppStore((state) => state.refreshInterval);
```

## Components

### Header (`components/Header.tsx`)

Displays app title and connection status with live indicator.

### Dashboard (`pages/Dashboard.tsx`)

Main monitoring interface with:
- Statistics cards (total torrents, seeding, uploading, bandwidth)
- Real-time torrent table with progress bars
- Auto-refresh functionality (configurable interval)
- Error handling and loading states

## Best Practices

### Type Safety

- ✅ Full TypeScript strict mode enabled
- ✅ Type-safe API client with proper interfaces
- ✅ Typed Zustand store
- ✅ No `any` types

### Performance

- ✅ Auto-refresh with configurable interval (default: 5 seconds)
- ✅ Parallel API requests for fast data loading
- ✅ Efficient re-renders with Zustand selectors
- ✅ CSS animations for smooth UX

### Code Organization

- ✅ Clear separation of concerns (components, services, types)
- ✅ Modular component structure
- ✅ Reusable API client with request/response handling
- ✅ Comprehensive error handling

### UI/UX

- ✅ Consistent dark theme (matches desktop app)
- ✅ Responsive design for all screen sizes
- ✅ Loading states and error messages
- ✅ Real-time connection indicator

## API Endpoints

The web client uses these endpoints from the backend:

```
GET  /api/health                  # Health check
GET  /api/torrents                # List all torrents
GET  /api/torrents/stats          # Statistics
GET  /api/torrents/prioritized    # Sorted by priority
GET  /api/seeding-events          # All seeding events
GET  /api/seeding-events/recent   # Last 100 events
```

## Development Workflow

### Running with Desktop App

Start both the desktop app and web client together:

```bash
npm run dev:all
```

This opens:
- Desktop app on Tauri window
- Web client at `http://localhost:3000`

### Type Checking

Check for TypeScript errors without building:

```bash
npm run type-check
```

### Production Build

```bash
npm run build
```

Output files in `dist/` directory ready for deployment.

## Network Access

To access the web client from another machine:

1. Find your computer's IP address:
   - **Linux/macOS**: `ifconfig | grep inet`
   - **Windows**: `ipconfig`

2. Access at `http://<your-ip>:3000`

3. Make sure port 3000 is open in your firewall/router

For production deployment, use a reverse proxy (like nginx) with HTTPS.

## Troubleshooting

### Cannot connect to backend

- Ensure the desktop app is running (`npm run dev`)
- Verify backend is listening on port 8080
- Check browser console for CORS errors

### Port 3000 already in use

Kill the process using the port or specify a different port:

```bash
# Kill process on port 3000
lsof -i :3000 | grep LISTEN | awk '{print $2}' | xargs kill -9
```

### TypeScript errors

Run type checker and fix issues:

```bash
npm run type-check
```

## License

MIT - Part of Rusty Torrents project
