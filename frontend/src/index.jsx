/* @refresh reload */
import { render } from 'solid-js/web'
import { createSignal } from 'solid-js';
import { Router, Route } from "@solidjs/router";
import { loadSession, userSession } from './scripts/session';
import { App } from './App'
import { Board } from './pages/Board';


export const [session, updateSession] = createSignal()

loadSession()
.then(() => updateSession(userSession()))
.catch((err) => console.log(err))

render(
  () => (
    <Router root={App}>
      <Route path="/" component={Board} />
    </Router>
  ), 
  document.body
)
