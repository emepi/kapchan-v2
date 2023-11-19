/**
 * Kapchan header bar with top navigation.
 */
import { A } from '@solidjs/router'
import { Show } from 'solid-js'
import './Header.css'
import { state } from '..'
import { UserRole } from '../scripts/user'
import { Service, serviceRequest } from '../scripts/connection_manager'
import { UserServiceType } from '../scripts/user_service'


export function Header() {
  
  const logoutRequest = () => {
    serviceRequest(Service.UserService, {
      t: UserServiceType.Logout,
      b: ""
    });
  };

  return (
    <header class="main-head">
      <A href="/">
        <h1>kapakka</h1>
      </A>
      <nav class="main-header-nav">

      <Show when={state.user.role >= UserRole.Admin}>
        <A class="nav-button" href="/admin">
          <div class="nav-icon">⚖️</div>
          admin
        </A>
      </Show>

      <Show
        when={state.user.role === UserRole.Anonymous}
        fallback={
          <button 
            class="nav-button nav-act" 
            onClick={logoutRequest}
          >
            <div class="nav-icon">🔒</div>
            logout
          </button>
        }
      >
        <A class="nav-button" href="/login">
          <div class="nav-icon">🔒</div>
          login
        </A>
        <A class="nav-button" href="/apply">
          <div class="nav-icon">📩</div>
          join
        </A>
      </Show>

        <form class="nav-search">
          <input class="nav-search-field" type="search" placeholder="threads, posts, images.."></input>
          <button class="nav-search-btn" type="submit">🔍</button>
        </form>
      </nav>
    </header>
  )
}