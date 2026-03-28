import { Header } from "./components/Header";
import { Dashboard } from "./pages/Dashboard";
import "./App.css";

function App() {
  return (
    <div className="app">
      <Header />
      <main className="app-content">
        <Dashboard />
      </main>
    </div>
  );
}

export default App;
