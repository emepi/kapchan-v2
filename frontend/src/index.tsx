/* @refresh reload */
import { render } from 'solid-js/web'
import { Router } from "@solidjs/router";
import { createStore } from 'solid-js/store';

import './index.css'
import App from './App'
import { anonUser, cookieSession } from './scripts/cookies';


export const [state, setState] = createStore({
  user: cookieSession() ?? anonUser,
});


render(
  () => (
    <Router>
      <App />
    </Router>
  ), 
  document.getElementById('root')!
)