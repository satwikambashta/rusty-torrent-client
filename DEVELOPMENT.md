# Development Guide

This document provides guidelines for developing the Rusty Torrents application.

## Getting Started

### Prerequisites
- Node.js 16+ (npm 7+)
- Rust 1.70+
- Tauri CLI: `npm install -g @tauri-apps/cli`

### Initial Setup

```bash
# Install dependencies
npm install

# Build the backend
cd src-tauri
cargo build
cd ..

# Start development server
npm run dev
```

## Development Workflow

### Frontend Development

#### File Structure Best Practices
```
src/
├── components/    # Presentational components (no business logic)
├── pages/         # Full-page components
├── services/      # API calls and Tauri integration
├── hooks/         # Custom React hooks
├── types/         # TypeScript interfaces
└── styles/        # Global and layout styles
```

#### Adding a New Component

1. **Create component file** in `src/components/MyComponent.tsx`:
```typescript
import "./MyComponent.css";

interface MyComponentProps {
  title: string;
  onAction: () => void;
}

export function MyComponent({ title, onAction }: MyComponentProps) {
  return (
    <div className="my-component">
      <h3>{title}</h3>
      <button onClick={onAction}>Click me</button>
    </div>
  );
}
```

2. **Create styles** in `src/components/MyComponent.css`
3. **Export** from `src/components/index.ts` (if you create one)
4. **Use** in pages or other components with proper TypeScript props

#### Adding a New Page

1. **Create page file** in `src/pages/MyPage.tsx`
2. **Add route** in `src/App.tsx`:
```typescript
<Route path="/mypage" element={<MyPage />} />
```
3. **Add navigation link** in header or nav

#### State Management with Zustand

For global state, add to `src/services/store.ts`:

```typescript
interface AppState {
  myValue: string;
  setMyValue: (value: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  myValue: "",
  setMyValue: (value) => set({ myValue: value }),
}));
```

Use in components:
```typescript
const myValue = useAppStore((state) => state.myValue);
const setMyValue = useAppStore((state) => state.setMyValue);
```

### Backend Development

#### File Structure Best Practices
```
src-tauri/src/
├── modules/       # Core business logic
│   ├── config.rs
│   └── torrent.rs
├── commands/      # Tauri IPC endpoints
│   ├── test.rs
│   └── torrent.rs
├── main.rs        # Entry point
└── lib.rs         # Initialization
```

#### Adding a New Rust Module

1. **Create module file** in `src-tauri/src/modules/my_module.rs`:
```rust
pub struct MyStruct {
    pub field: String,
}

impl MyStruct {
    pub fn new() -> Self {
        Self {
            field: "value".to_string(),
        }
    }
}
```

2. **Register in** `src-tauri/src/modules/mod.rs`:
```rust
pub mod my_module;
pub use my_module::*;
```

3. **Use in commands** in `src-tauri/src/commands/`

#### Adding a New Tauri Command

1. **Create in** `src-tauri/src/commands/my_command.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MyResponse {
    pub data: String,
}

#[tauri::command]
pub fn my_command(param: String) -> Result<MyResponse, String> {
    Ok(MyResponse {
        data: format!("Received: {}", param),
    })
}
```

2. **Register in** `src-tauri/src/commands/mod.rs`:
```rust
pub mod my_command;
pub use my_command::*;
```

3. **Add to handler** in `src-tauri/src/lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::my_command,
    // ... other commands
])
```

4. **Call from frontend** in `src/services/api.ts`:
```typescript
export async function myCommand(param: string): Promise<MyResponse> {
  return invoke("my_command", { param });
}
```

## Common Tasks

### Running in Development Mode
```bash
npm run dev
```
This opens the desktop app with hot reload for frontend code.

### Building for Production
```bash
npm run build
```
This builds the frontend and backend into an optimized app.

### Type Checking
```bash
npm run type-check
```
Checks for TypeScript errors without building.

### Debugging Frontend
- Press F12 to open DevTools
- Use console, debugger, and network tabs
- Check React DevTools extension

### Debugging Backend
- Add `println!` macros or use `dbg!` macro
- Check console output in terminal
- Use `rust-analyzer` in VS Code for better debugging

### Testing IPC Communication
1. Navigate to `/test` page
2. Click "Test Connection"
3. Verify response appears

## Coding Standards

### TypeScript/React
- Use functional components with hooks
- Props should be typed with interfaces
- Use descriptive variable/function names
- Keep components under 200 lines
- Extract reusable logic to custom hooks

### Rust
- Use descriptive struct names (PascalCase)
- Functions should be snake_case
- Add documentation comments for public items
- Handle errors with `Result<T, E>`
- Use `serde` for serialization

### Error Handling

Frontend:
```typescript
try {
  const result = await apiCall();
  setError(null);
} catch (err) {
  setError(err instanceof Error ? err.message : String(err));
}
```

Backend:
```rust
#[tauri::command]
pub fn my_command() -> Result<Data, String> {
    operation()
        .map_err(|e| e.to_string())
}
```

## Performance Tips

### Frontend
1. Use React DevTools Profiler to identify slow renders
2. Memoize expensive computations with `useMemo`
3. Use `useCallback` for event handlers passed to children
4. Lazy load pages with React.lazy() and Suspense
5. Keep state as local as possible

### Backend
1. Use `tokio` for async operations
2. Batch database operations
3. Cache frequently accessed data
4. Use efficient data structures
5. Profile with `cargo flamegraph`

## Deployment

### Building Release Version
```bash
npm run build
```

### Output
- macOS: `src-tauri/target/release/bundle/macos/`
- Windows: `src-tauri/target/release/`
- Linux: `src-tauri/target/release/bundle/deb/`

## Troubleshooting

### Build Fails
```bash
# Clean and rebuild
cargo clean
npm run build
```

### Tests Won't Run
```bash
cd src-tauri
cargo test
```

### Hot Reload Not Working
```bash
# Restart dev server
npm run dev
```

### Module Not Found Errors
- Check imports are correct
- Verify files exist
- Clear node_modules: `rm -rf node_modules && npm install`

## Resources

- **Tauri Docs**: https://tauri.app/
- **React Docs**: https://react.dev/
- **Rust Book**: https://doc.rust-lang.org/book/
- **TypeScript**: https://www.typescriptlang.org/docs/
- **Zustand**: https://github.com/pmndrs/zustand
- **React Router**: https://reactrouter.com/

---

For more information, see [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md) and [TEST_GUIDE.md](./TEST_GUIDE.md).
