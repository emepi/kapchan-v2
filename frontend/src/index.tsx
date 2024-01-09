/* @refresh reload */
import { render } from 'solid-js/web'
import { Router } from "@solidjs/router";

import './index.css'
import App from './App'
import { createSignal } from 'solid-js';
import { loadSession, userSession } from './scripts/session';


export const [role, setRole] = createSignal(10);

loadSession()
.then(() => {
  let session = userSession();

  if (session) {
    setRole(session.role);
  }
})
.catch((err) => console.log(err))

render(
  () => (
    <Router>
      <App />
    </Router>
  ), 
  document.getElementById('root')!
)