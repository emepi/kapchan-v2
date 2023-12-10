/**
 * 
 * 
 */

import { A } from "@solidjs/router";
import kapakkaLogo from '/src/assets/logo.png'
import './Sidebar.css'
import { Service, serviceRequest } from "../scripts/connection_manager";
import { BoardServiceType } from "../scripts/board_service";
import { createStore } from "solid-js/store";
import { For } from "solid-js";

function Sidebar() {
  const [boards, setBoards] = createStore([]);

  const fetchBoards = () => {
    serviceRequest(Service.BoardService, {
      t: BoardServiceType.FetchBoards,
      b: JSON.stringify("")
    }, updateBoards);
  };

  const updateBoards = (b: string) => {
    let boards = JSON.parse(b);

    setBoards(boards);
    console.log(boards);
  };

  fetchBoards();

  return (
    <aside class="main-side">
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

        <For each={boards} fallback={<p>loading..</p>}>{board => {
          //let data = application as Application;

          console.log(board);
      
          return(
            <A href="/plc">/{board[0].handle}/ {board[0].title}</A>
          )}
        }</For>

        </nav>

        <button onClick={fetchBoards}>fetch boards</button>
    </aside>
  )
}

export default Sidebar;