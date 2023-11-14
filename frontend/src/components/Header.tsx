/**
 * Kapchan header bar with top navigation. 
 */
import { A } from '@solidjs/router'
import './Header.css'


export function Header() {
  return (
    <header class="main-header">
      <A href="/">
        <h1>kapakka</h1>
      </A>
      <nav class="main-header-nav">
        <A class="nav-button" href="/login">
          <div class="nav-icon">🔒</div>
          login
        </A>

        <form class="nav-search">
          <input class="nav-search-field" type="search" placeholder="threads, posts, images.."></input>
          <button class="nav-search-btn" type="submit">🔍</button>
        </form>
      </nav>
    </header>
  )
}