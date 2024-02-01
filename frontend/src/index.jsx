/* @refresh reload */
import { render } from 'solid-js/web'
import { Router, Route } from "@solidjs/router";
import { App } from './App'
import { Board } from './pages/Board';
import { Login } from './pages/Login';


render(
  () => (
    <Router root={App}>
      <Route path="/" component={Board} />
      <Route path="/login" component={Login} />
    </Router>
  ), 
  document.body
)
