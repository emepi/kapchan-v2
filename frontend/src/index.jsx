/* @refresh reload */
import { render } from 'solid-js/web'
import { Router, Route } from "@solidjs/router";
import { App } from './App'
import { Board } from './pages/Board';
import { Login } from './pages/Login';
import { Admin } from './pages/Admin';
import { Thread } from './pages/Thread';


render(
  () => (
    <Router root={App}>
      <Route path="/" component={Board} />
      <Route path="/login" component={Login} />
      <Route path="/admin" component={Admin} />
      <Route path="/thread/:id" component={Thread} />
    </Router>
  ), 
  document.body
)
