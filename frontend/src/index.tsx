/* @refresh reload */
import { render } from 'solid-js/web'
import { Router } from "@solidjs/router";
import { createStore } from 'solid-js/store';

import './index.css'
import App from './App'
import { cookieSession } from './scripts/cookies';
import { anonUser } from './scripts/user';


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