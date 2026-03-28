# Backend-Frontend Connectivity Test Guide

## Overview

The project includes a dedicated **Test Page** to verify that your Rust backend and React frontend are properly connected. This is essential for development and debugging.

## Accessing the Test Page

1. Start the development server: `npm run dev` from the project root
2. Navigate to the "Test Connection" tab in the application
3. Or directly visit: `http://localhost:5173/test` (after app is running)

## Available Tests

### 1. Test Connection (`test_connection`)
**Purpose**: Verify basic IPC communication between frontend and backend

**What it does**:
- Sends a request from React to the Rust backend
- Rust processes the request and responds
- React displays the response with timestamp and version info

**Expected Response**:
```json
{
  "status": "success",
  "message": "Backend is connected and responding!",
  "timestamp": "2026-03-28T10:30:45+00:00",
  "backend_version": "0.1.0"
}
```

**Success Indicators**:
- ✓ Response received within 1 second
- ✓ Status shows "success"
- ✓ Message is clearly visible
- ✓ Communication status shows "Established"

### 2. Get Server Info (`get_server_info`)
**Purpose**: Retrieve information about the backend server

**What it does**:
- Requests server metadata from the Rust backend
- Returns version and status information
- Useful for debugging backend state

**Expected Response**:
```json
{
  "status": "success",
  "message": "Rusty Torrents Server v0.1.0",
  "timestamp": "2026-03-28T10:30:45+00:00",
  "backend_version": "0.1.0"
}
```

## How to Use the Test Page

### Step 1: Click "Test Connection"
```
1. Navigate to the Test Connection page
2. Click the "Test Connection" button
3. Wait for response (usually < 1 second)
```

### Step 2: View Results
```
The page displays:
- Response JSON in a collapsible details panel
- Connection status indicator
- Timestamp of the test
- Backend version information
```

### Step 3: Check System Status
```
At the bottom of the page, see:
- Frontend Status: ✓ Running (always true if page loaded)
- Backend Status: ✓ Running (after successful test)
- Communication: ✓ Established (after successful test)
```

## Troubleshooting

### Test Button Does Nothing
**Problem**: Clicking the button doesn't produce any response

**Solutions**:
1. Check if the backend started: `cargo build --release` in `src-tauri`
2. Verify Tauri is running in dev mode: `npm run dev`
3. Check browser console for errors (F12)

### "Connection test failed" Error
**Problem**: Error message appears instead of response

**Solutions**:
1. Ensure backend compiled successfully: `cd src-tauri && cargo check`
2. Check if the `test_connection` command is registered in `lib.rs`
3. Review browser console (F12) for detailed error
4. Check Tauri console output in terminal

### Timestamp is Wrong
**Problem**: Timestamp doesn't match current time

**Solutions**:
1. This is usually just a timezone issue - response is still valid
2. Verify system time is correct
3. Check the backend's chrono configuration

### Connection Shows "Not tested"
**Problem**: Status shows connection hasn't been tested yet

**Solutions**:
1. You haven't clicked a test button yet - click one!
2. Test failed silently - check browser console
3. Refresh page and try again

## Expected Workflow

### Development Setup
```bash
1. npm install                 # Install frontend dependencies
2. cargo build                 # Build backend
3. npm run dev                 # Start development server
4. Navigate to /test           # Open test page
5. Click "Test Connection"     # Run test
```

### If Test Passes ✓
```
- Backend and frontend are communicating
- Ready to test other features
- Can proceed with development
```

### If Test Fails ✗
```
1. Check console errors (F12)
2. Verify Cargo.toml has all commands registered
3. Check lib.rs invoke_handler includes test_connection
4. Rebuild: cargo clean && cargo build
5. Restart dev server: npm run dev
```

## Code References

### Frontend Test Service (`src/services/api.ts`)
```typescript
export async function testConnection(): Promise<TestResponse> {
  return invoke("test_connection");
}
```

### Backend Test Command (`src-tauri/src/commands/test.rs`)
```rust
#[tauri::command]
pub fn test_connection() -> TestResponse {
    TestResponse {
        status: "success".to_string(),
        message: "Backend is connected and responding!".to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
        backend_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
```

### Command Registration (`src-tauri/src/lib.rs`)
```rust
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::test_connection,
            commands::get_server_info,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Advanced Testing

### Testing Torrent Commands
Once connectivity is verified, test torrent-related commands:

1. **Get Torrents**: Fetch list of active torrents
2. **Add Torrent**: Upload a .torrent file
3. **Control Downloads**: Start/pause/remove torrents

### Performance Testing
```
1. Run network profiler (F12 > Network tab)
2. Monitor IPC communication
3. Check message size and latency
4. Optimize as needed
```

### Error Simulation
To test error handling:

1. Stop the backend while test is running
2. Modify a torrent command to return an error
3. Verify error is properly displayed in UI
4. Check error handling in try/catch blocks

## Next Steps

After verifying connectivity:

1. ✓ Test Connection works
2. → Implement torrent parsing module
3. → Add DHT (Distributed Hash Table) support
4. → Implement peer discovery
5. → Create download progress tracking
6. → Add configuration UI
7. → Build package for distribution

---

**Need Help?**
- Check [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md) for code organization
- Review [DEVELOPMENT.md](./DEVELOPMENT.md) for development guidelines
- Check Tauri docs: https://tauri.app/develop/
