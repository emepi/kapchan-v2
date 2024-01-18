import { Route, Routes } from '@solidjs/router'
import { Header } from './components/Header'
import { Login } from './pages/Login'
import Sidebar from './components/Sidebar'
import { Application } from './pages/Application'
import { Admin } from './pages/Admin'
import { Boards } from './pages/Boards'
import { Home } from './pages/Home'

function App() {
  return (
    <>
      <Header/>
      <Sidebar/>
      <main class="main-cont">
        <Routes>
          <Route path="/" component={Home} />
          <Route path="/:board?" component={Boards} />
          <Route path="/login" component={Login} />
          <Route path="/apply" component={Application} />
          <Route path="/admin" component={Admin} />
        </Routes>
      </main>
    </>
  )
}

export default App
