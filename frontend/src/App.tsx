import { Route, Routes } from '@solidjs/router'
import Placeholder from './pages/Placeholder'
import { Header } from './components/Header'
import { Login } from './pages/Login'

function App() {
  return (
    <>
      <Header/>

      <main class="main-cont">
        <Routes>
          <Route path="/" component={Placeholder} />
          <Route path="/login" component={Login} />
        </Routes>
      </main>
    </>
  )
}

export default App
