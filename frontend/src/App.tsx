import { BrowserRouter, NavLink, Route, Routes } from 'react-router-dom'
import Ranking from './pages/Ranking'
import Perfil from './pages/Perfil'
import Emitir from './pages/Emitir'

export default function App() {
  return (
    <BrowserRouter>
      <header className="topbar">
        <NavLink to="/" className="brand">
          Selo
        </NavLink>
        <nav>
          <NavLink to="/" end>
            Ranking
          </NavLink>
          <NavLink to="/emitir">Emitir</NavLink>
        </nav>
      </header>
      <main className="container">
        <Routes>
          <Route path="/" element={<Ranking />} />
          <Route path="/m/:address" element={<Perfil />} />
          <Route path="/emitir" element={<Emitir />} />
          <Route path="*" element={<p className="state">Página não encontrada.</p>} />
        </Routes>
      </main>
    </BrowserRouter>
  )
}
