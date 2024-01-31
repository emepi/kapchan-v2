import { Show } from 'solid-js'
import { SvgAnchor } from './components/SvgAnchor'
import { AccessLevel } from './scripts/user'
import { session, updateSession } from '.'
import './styles/fuji.css'
import { credentials } from './scripts/credentials'

export const App = (props) => {
  const logout = () => {
    credentials.access_token = undefined
    localStorage.removeItem("access_token")
    updateSession({role: AccessLevel.Anonymous})
  }

  return (
  <>
    <header class="main-header">
      <a href="/"><h1>kapakka</h1></a>
      <nav class="navbar">
        <SvgAnchor
          href="/chat"
          icon="M240-400h320v-80H240v80Zm0-120h480v-80H240v80Zm0-120h480v-80H240v80ZM80-80v-720q0-33 23.5-56.5T160-880h640q33 0 56.5 23.5T880-800v480q0 33-23.5 56.5T800-240H240L80-80Zm126-240h594v-480H160v525l46-45Zm-46 0v-480 480Z"
          label="chat" 
        />

        <div class="vr" />

        <Show when={session() && session().role <= AccessLevel.Registered}>
          <SvgAnchor
            href="/apply"
            icon="M240-160q-33 0-56.5-23.5T160-240q0-33 23.5-56.5T240-320q33 0 56.5 23.5T320-240q0 33-23.5 56.5T240-160Zm0-240q-33 0-56.5-23.5T160-480q0-33 23.5-56.5T240-560q33 0 56.5 23.5T320-480q0 33-23.5 56.5T240-400Zm0-240q-33 0-56.5-23.5T160-720q0-33 23.5-56.5T240-800q33 0 56.5 23.5T320-720q0 33-23.5 56.5T240-640Zm240 0q-33 0-56.5-23.5T400-720q0-33 23.5-56.5T480-800q33 0 56.5 23.5T560-720q0 33-23.5 56.5T480-640Zm240 0q-33 0-56.5-23.5T640-720q0-33 23.5-56.5T720-800q33 0 56.5 23.5T800-720q0 33-23.5 56.5T720-640ZM480-400q-33 0-56.5-23.5T400-480q0-33 23.5-56.5T480-560q33 0 56.5 23.5T560-480q0 33-23.5 56.5T480-400Zm40 240v-123l221-220q9-9 20-13t22-4q12 0 23 4.5t20 13.5l37 37q8 9 12.5 20t4.5 22q0 11-4 22.5T863-380L643-160H520Zm300-263-37-37 37 37ZM580-220h38l121-122-18-19-19-18-122 121v38Zm141-141-19-18 37 37-18-19Z"
            label="liity" 
          />
        </Show>
        <Show when={session() && session().role === AccessLevel.Anonymous}>
          <SvgAnchor
            href="/login"
            icon="M480-120v-80h280v-560H480v-80h280q33 0 56.5 23.5T840-760v560q0 33-23.5 56.5T760-120H480Zm-80-160-55-58 102-102H120v-80h327L345-622l55-58 200 200-200 200Z"
            label="kirjaudu" 
          />
        </Show>
        <Show when={session() && session().role > AccessLevel.Anonymous}>
          <SvgAnchor
            onClick={logout}
            icon="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h280v80H200Zm440-160-55-58 102-102H360v-80h327L585-622l55-58 200 200-200 200Z"
            label="ulos" 
          />
        </Show>
      </nav>
    </header>

    <main class="content">
      {props.children}
    </main>
    <footer class="footer">
      <a>säännöt</a>
      <div class="vr" />
      <a>info</a>
    </footer>
  </>
  )
}
