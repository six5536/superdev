import { BrowserRouter, Link, Route, Routes } from "react-router-dom";
import { greeting } from "@/lib/greeting";

function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4">
      <h1 className="text-3xl font-semibold">{greeting()}</h1>
      <p className="text-sm opacity-70">Welcome to {{superdev:project-name}}.</p>
      <Link className="underline" to="/about">
        About
      </Link>
    </main>
  );
}

function About() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4">
      <h1 className="text-3xl font-semibold">About</h1>
      <Link className="underline" to="/">
        Home
      </Link>
    </main>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/about" element={<About />} />
      </Routes>
    </BrowserRouter>
  );
}
