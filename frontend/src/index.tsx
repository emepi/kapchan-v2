/* @refresh reload */
import { render } from 'solid-js/web'
import { Router } from "@solidjs/router";

import { connection } from './scripts/connection_manager'
import './styles/kapchan.css'
import App from './App'

const root = document.getElementById('root')

let test = connection;

render(
  () => (
    <Router>
      <App />
    </Router>
  ), 
  root!
)
