# Quick Start Guide

## 🚀 Starting the Dev Server

**From the project root directory ONLY** (not `src-tauri`):

```bash
npm run dev
```

This command:
1. ✅ Starts Vite on http://localhost:5173/
2. ✅ Compiles and runs Rust backend
3. ✅ Enables hot-reload for frontend changes
4. ✅ Watches for backend code changes

## ⚙️ Port Information

- **Frontend**: http://localhost:5173/ (Vite dev server)
- **Backend**: Running as Tauri desktop app
- **IPC**: Automatic communication between frontend and backend

## 🛑 Stopping the Dev Server

Press `Ctrl+C` in the terminal where the dev server is running.

## 🔴 Common Issues & Fixes

### "Port 5173 is already in use"

**Cause**: A previous vite process is still running or another app is using the port.

**Fix**:
```bash
# Kill any process on port 5173
lsof -i :5173 | grep LISTEN | awk '{print $2}' | xargs kill -9

# Then restart
npm run dev
```

### "npm ERR! code ENOENT"

**Cause**: Running command from wrong directory

**Fix**:
```bash
# Make sure you're in the project root
cd /home/satwik/Projects/rust/tauri-app
npm run dev
```

### Compilation Errors

**Fix**:
```bash
# Clean rebuild
cargo clean --manifest-path ./src-tauri/Cargo.toml
npm run dev
```

## ✅ Verifying Everything Works

1. **Dev server starts**: You see "ready in XXX ms" and "Local: http://localhost:5173/"
2. **Backend compiles**: You see "Finished `dev` profile"
3. **App window opens**: Desktop window appears with Rusty Torrents UI
4. **Test connection**:
   - Click "Test Connection" tab
   - Click "Test Connection" button
   - Verify response shows success ✓

## 📂 Development Workflow

```
Edit React/TypeScript files → Auto-reload in browser (http://localhost:5173/)
Edit Rust files → Auto-recompile backend
Changes reflected immediately in dev server
```

## 🗂️ Project Structure Reminder

```
npm run dev              # Frontend + Backend (from root only)
src/                     # React/TypeScript frontend
src-tauri/               # Rust backend
```

## 📝 Useful Commands

```bash
npm run dev              # Start development (CORRECT WAY)
npm run vite:dev         # Just frontend Vite server
npm run type-check       # Check TypeScript errors
npm run build            # Build for production
cargo build              # Build Rust backend only
```

## 🎯 Next Steps

1. ✅ Dev server running
2. Run the app and test connectivity via /test page
3. Start implementing features from the roadmap in [README.md](./README.md)

---

**Remember**: Always run `npm run dev` from the **project root**, not from `src-tauri` directory!
