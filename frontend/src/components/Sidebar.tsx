/**
 * 
 * 
 */

import { A } from "@solidjs/router";
import kapakkaLogo from '/src/assets/misaki2.jpeg'
import './Sidebar.css'


function Sidebar() {

  return (
    <aside class="main-side">
        <img class="logo" src={kapakkaLogo} />

        <h2 class="group-title">Kapchan-v2</h2>
        <nav class="sidebar-nav">
          <hr class="group-divider" />
          <A href="/rules">Rules</A>
        </nav>

        <h3 class="group-title">Account</h3>
        <nav class="sidebar-nav">
         <hr class="group-divider" />
          <A href="/settings">Settings</A>
        </nav>

        <h3 class="group-title">Boards</h3>
        <nav class="sidebar-nav">
          <hr class="group-divider" />
        </nav>
    </aside>
  )
}

export default Sidebar;