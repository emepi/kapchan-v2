import { Route, Routes } from '@solidjs/router'
import Placeholder from './pages/Placeholder'
import Rules from './pages/Rules'
import Settings from './pages/Settings'
import { Header } from './components/Header'

function App() {
  return (
    <>
      <Header/>

      <div class="main-content">
        <Routes>
          <Route path="/" component={Placeholder} />
          <Route path="/plc" component={Placeholder} />
          <Route path="/rules" component={Rules} />
          <Route path="/settings" component={Settings} />
        </Routes>
      </div>
    </>
  )
}

export default App
