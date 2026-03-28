import { BrowserRouter, Routes, Route, Link } from "react-router-dom";
import { useConnectionTest } from "./hooks/useConnectionTest";
import { Header } from "./components/Header";
import { HomePage } from "./pages/HomePage";
import { TestPage } from "./pages/TestPage";
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
            Downloads
          </Link>
          <Link to="/test" className="nav-link">
            Test Connection
          </Link>
        </nav>

        <main className="app-content">
          <Routes>
            <Route path="/" element={<HomePage />} />
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

