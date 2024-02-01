import { For, Show, createSignal } from "solid-js"
import { AccessLevel } from "../scripts/user"
import { RadioButton } from "../components/RadioButton"
import { boards } from "./Board";


export const Admin = () => {
  const [view, changeView] = createSignal("boards");

  const selectView = (e) => {
    const input = e.target
    changeView(input.value)
  }

  return (
    <>
      <nav class="board-selector">
        <div class="navbar">
          <RadioButton name="admin" onChange={selectView} accessLevel={AccessLevel.Owner} value="boards" checked>laudat</RadioButton>
        </div>
        <Show when={view() === "boards"}>
          <BoardModalButton />
        </Show>
      </nav>
      <Show when={view() === "boards"}>
        <div class="container">
          <table>
            <tbody>
              <tr>
                <th>koodi</th>
                <th>nimi</th>
                <th>käyttäjätaso</th>
                <th>nostoraja</th>
                <th>nsfw</th>
              </tr>
              <For each={boards()}>{ (board) => 
                <tr>
                  <td>{board.handle}</td>
                  <td>{board.title}</td>
                  <td>{board.access_level}</td>
                  <td>{board.bump_limit}</td>
                  <td>{board.nsfw ? "kyllä" : "ei"}</td>
                </tr>}
                </For>
            </tbody>
          </table>
        </div>
      </Show>
    </>
  )
}

const BoardModalButton = () => {
  const [open, setOpen] = createSignal(false)

  return (
    <div>
      <button onClick={() => setOpen(true)} class="radio-lbl">
        luo lauta
      </button>
    </div>
  )
}