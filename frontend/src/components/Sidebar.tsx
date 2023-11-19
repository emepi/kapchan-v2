/**
 * 
 * <img src={kapakkaLogo} class="logo" alt="Entity from outer worlds" />
 */

import { A } from "@solidjs/router";
import kapakkaLogo from '/src/assets/logo.png'

function Sidebar() {
  return (
    <aside class="main-sidebar">
        <h2>Kapchan-v2</h2>
        <nav class="sidebar-nav">
           <div class="icon-a">
              <div class="icon-placeholder">R</div>
              <A href="/rules">Rules</A>
            </div>
        </nav>

        <h3>Account</h3>
        <nav class="sidebar-nav">
          <div class="icon-a">
            <div class="icon-placeholder">N</div>
            <A href="/">Notifications</A>
          </div>

          <div class="icon-a">
            <div class="icon-placeholder">S</div>
            <A href="/settings">Settings</A>
          </div>
        </nav>

        <h3>Boards</h3>
        <nav class="sidebar-nav">
          <A href="/plc">/plc/ Placeholder</A>
        </nav>
    </aside>
  )
}

export default Sidebar;