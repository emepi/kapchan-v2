/* @refresh reload */
import { render } from 'solid-js/web'
import { Router, Route } from "@solidjs/router";
import { App } from './App'
import { Board } from './pages/Board';
import { Login } from './pages/Login';
import { Admin } from './pages/Admin';


render(
  () => (
    <Router root={App}>
      <Route path="/" component={Board} />
      <Route path="/login" component={Login} />
      <Route path="/admin" component={Admin} />
    </Router>
  ), 
  document.body
)
