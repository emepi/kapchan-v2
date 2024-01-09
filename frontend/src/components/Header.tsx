/**
 * Kapchan header bar with top navigation.
 */
import { A } from '@solidjs/router'
import { Show, useContext } from 'solid-js'
import './Header.css'
import { endSession, startSession, userSession } from '../scripts/session';
import { AccessLevel } from '../scripts/user';
import { setState, state } from '..';


export function Header() {

  const logout = () => {
    endSession();

    startSession()
    .then((res) => setState("session", userSession()))
    .catch((err) => console.log(err));
  };

  return (
    <header class="main-head">
      <A href="/">
        <h1>kapakka</h1>
      </A>
      <nav class="main-header-nav">

      <Show when={state.session && state.session.role >= AccessLevel.Admin}>
        <A class="nav-button" href="/admin">
          <div class="nav-icon">⚖️</div>
          admin
        </A>
      </Show>

      <Show when={state.session && state.session.role < AccessLevel.PendingMember}>
        <A class="nav-button" href="/login">
          <div class="nav-icon">🔒</div>
          login
        </A>
        <A class="nav-button" href="/apply">
          <div class="nav-icon">📩</div>
          join
        </A>
      </Show>

      <Show when={state.session && state.session.role >= AccessLevel.PendingMember}>
        <button class="nav-button nav-act" onClick={logout}>
          <div class="nav-icon">🔒</div>
          logout
        </button>
      </Show>

        <form class="nav-search">
          <input class="nav-search-field" type="search" placeholder="threads, posts, images.."></input>
          <button class="nav-search-btn" type="submit">🔍</button>
        </form>
      </nav>
    </header>
  )
}