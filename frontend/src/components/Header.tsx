/**
 * Kapchan header bar with top navigation.
 */
import { A } from '@solidjs/router'
import { Show } from 'solid-js'
import './Header.css'
import { endSession, startSession, userSession } from '../scripts/session';
import { AccessLevel } from '../scripts/user';
import { role, setRole } from '..';


export function Header() {

  const logout = async () => {
    await endSession();

    await startSession()
    .then((_res) => {
      const session = userSession();

      if (session) {
        setRole(session.role);
      }
    })
    .catch((err) => console.log(err));
  };

  return (
    <header class="main-head">
      <A href="/">
        <h1>kapakka</h1>
      </A>
      <nav class="main-header-nav">

      <Show when={role() >= AccessLevel.Admin}>
        <A class="nav-button" href="/admin">
          <div class="nav-icon">⚖️</div>
          admin
        </A>
      </Show>

      <Show when={role() < AccessLevel.PendingMember}>
        <A class="nav-button" href="/login">
          <div class="nav-icon">🔒</div>
          login
        </A>
        <A class="nav-button" href="/apply">
          <div class="nav-icon">📩</div>
          join
        </A>
      </Show>

      <Show when={role() >= AccessLevel.PendingMember}>
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