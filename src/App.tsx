import { BrowserRouter, Routes, Route, Link } from "react-router-dom";
import { useConnectionTest } from "./hooks/useConnectionTest";
import { Header } from "./components/Header";
import { HomePage } from "./pages/HomePage";
import { TestPage } from "./pages/TestPage";
import { SearchPage } from "./pages/SearchPage";
import { ConfigPage } from "./pages/ConfigPage";
import "./App.css";

function App() {
  const { isChecking } = useConnectionTest();

  if (isChecking) {
    return (
      <div className="loading-screen">
        <div className="spinner"></div>
        <p>Initializing...</p>
      </div>
    );
  }

  return (
    <BrowserRouter>
      <div className="app">
        <Header />
        <nav className="app-nav">
          <Link to="/" className="nav-link">
            📥 Downloads
          </Link>
          <Link to="/search" className="nav-link">
            🔍 Search
          </Link>
          <Link to="/config" className="nav-link">
            ⚙️ Config
          </Link>
          <Link to="/test" className="nav-link">
            🧪 Test
          </Link>
        </nav>

        <main className="app-content">
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/search" element={<SearchPage />} />
            <Route path="/config" element={<ConfigPage />} />
            <Route path="/test" element={<TestPage />} />
          </Routes>
        </main>

        <footer className="app-footer">
          <p>&copy; 2026 Rusty Torrents. Built with Rust & React.</p>
        </footer>
      </div>
    </BrowserRouter>
  );
}

export default App;

