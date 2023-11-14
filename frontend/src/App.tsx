import { Route, Routes } from '@solidjs/router'
import Placeholder from './pages/Placeholder'
import { Header } from './components/Header'
import { Login } from './pages/Login'

function App() {
  return (
    <>
      <Header/>

      <div class="main-content">
        <Routes>
          <Route path="/" component={Placeholder} />
          <Route path="/login" component={Login} />
        </Routes>
      </div>
    </>
  )
}

export default App
