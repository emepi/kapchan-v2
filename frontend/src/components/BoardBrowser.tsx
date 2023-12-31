import { JSX, Show } from "solid-js";
import { state } from "..";
import { UserRole } from "../scripts/user";
import './BoardBrowser.css';
import { BoardServiceType } from "../scripts/board_service";
import { Service, serviceRequest } from "../scripts/connection_manager";

export function BoardBrowser() {

  const boardHandler: JSX.EventHandlerUnion<HTMLFormElement, Event> = (e) => {
    e.preventDefault();
      
    let data = Object.fromEntries(new FormData(e.target as HTMLFormElement));

    serviceRequest(Service.BoardService, {
      t: BoardServiceType.CreateBoard,
      b: JSON.stringify({
        handle: data.bshort,
        title: data.bname,
        description: data.bdesc,
        flags: [],
      }),
    });
  }

  return (
    <div class="board-brwsr">
      <h3>Board Browser</h3>
      <Show when={state.user.role >= UserRole.Owner}>
        <form class="board-crt" onSubmit={boardHandler}>

          <input
            class="b-name-fld" 
            type="text" 
            id="bname" 
            name="bname" 
            placeholder="Board name" 
          />

          <input 
            class="b-short-fld"
            type="text" 
            id="bshort" 
            name="bshort" 
            placeholder="code" 
          />
          
          <textarea 
            class="b-desc" 
            id="bdesc" 
            name="bdesc" 
            rows="10" 
            cols="30"
            placeholder="Board description" 
          />

          <button class="b-submit">Create a board</button> 
        </form>     
      </Show>

    </div>
  )
};