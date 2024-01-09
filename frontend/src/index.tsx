/* @refresh reload */
import { render } from 'solid-js/web'
import { Router } from "@solidjs/router";
import { Session, loadSession, userSession } from './scripts/session';
import { createStore } from 'solid-js/store';

import './index.css'
import App from './App'


interface State {
  session: Session | undefined,
}

export const [state, setState] = createStore<State>({
  session: undefined,
});

// Initialize user session.
loadSession()
.then(() => setState("session", userSession()))
.catch(err => console.log(err));

render(
  () => (
    <Router>
      <App />
    </Router>
  ), 
  document.getElementById('root')!
)