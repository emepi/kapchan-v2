/**
 * 
 * 
 */

import { A } from "@solidjs/router";
import kapakkaLogo from '/src/assets/logo.png'
import './Sidebar.css'

function Sidebar() {
  return (
    <aside class="main-side">
        <img src={kapakkaLogo} class="logo" alt="Entity from outer worlds" />
        <h2>Kapchan-v2</h2>
        <nav class="sidebar-nav">
          <A href="/rules">Rules</A>
        </nav>

        <h3>Account</h3>
        <nav class="sidebar-nav">
          <A href="/">Notifications</A>
          <A href="/settings">Settings</A>
        </nav>

        <h3>Boards</h3>
        <nav class="sidebar-nav">
          <A href="/plc">/plc/ Placeholder</A>
        </nav>
    </aside>
  )
}

export default Sidebar;