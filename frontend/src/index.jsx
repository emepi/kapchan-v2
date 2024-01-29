/* @refresh reload */
import { render } from 'solid-js/web'
import { createSignal } from 'solid-js';
import { Router, Route } from "@solidjs/router";
import { loadSession, userSession } from './scripts/session';
import { AccessLevel } from './scripts/user';
import { App } from './App'
import { Board } from './pages/Board';


/* UI session state: updated on credentials.auth_token change */
export const [session, updateSession] = createSignal(
  loadSession() ? userSession() : {
    // Defaulted to anon placeholder to save a session request.
    role: AccessLevel.Anonymous,
  }
)

render(
  () => (
    <Router root={App}>
      <Route path="/" component={Board} />
    </Router>
  ), 
  document.body
)
