import { Route, Routes } from '@solidjs/router'
import Placeholder from './pages/Placeholder'
import { Header } from './components/Header'
import { Login } from './pages/Login'
import Sidebar from './components/Sidebar'
import { Application } from './pages/Application'

function App() {
  return (
    <>
      <Header/>
      <Sidebar/>
      <main class="main-cont">
        <Routes>
          <Route path="/" component={Placeholder} />
          <Route path="/login" component={Login} />
          <Route path="/apply" component={Application} />
        </Routes>
      </main>
    </>
  )
}

export default App
