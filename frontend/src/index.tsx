/* @refresh reload */
import { render } from 'solid-js/web'
import { Router } from "@solidjs/router";
import { createStore } from 'solid-js/store';

import './index.css'
import App from './App'
import { anonUser } from './scripts/user';
import { userSession } from './scripts/credentials';


export const [state, setState] = createStore({
  user: userSession() ?? anonUser,
});


render(
  () => (
    <Router>
      <App />
    </Router>
  ), 
  document.getElementById('root')!
)