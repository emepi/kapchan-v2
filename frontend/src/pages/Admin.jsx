import { For, Show, createSignal } from "solid-js"
import { AccessLevel } from "../scripts/user"
import { RadioButton } from "../components/RadioButton"
import { boards } from "./Board";
import { Modal } from "../components/Modal";
import { credentials } from "../scripts/credentials";


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

  const createBoard = async (e) => {
    e.preventDefault()
    const data = Object.fromEntries(new FormData(e.target))

    if (credentials.access_token) {
      fetch("/boards", {
        method: "POST",
        headers: [
          ["Content-Type", "application/json"],
          ["Authorization", "Bearer " + credentials.access_token]
        ],
        body: JSON.stringify({
          title: data.title,
          handle: data.handle,
          access_level: Number(data.access_level),
          bump_limit: Number(data.bump_limit),
          nsfw: data.nsfw ? true : false,
        }),
      })
    }

    setOpen(false)
  }

  return (
    <div>
      <button onClick={() => setOpen(true)} class="radio-lbl">
        luo lauta
      </button>
      <Show when={open()}>
        <Modal title="Luo lauta" onClose={() => setOpen(false)}>
          <form class="cb-form" onSubmit={createBoard}>
            <label>
              <input class="input" type="text" name="title" placeholder="nimi" maxLength="100" />
            </label>

            <label>
              <input class="input" type="text" name="handle" placeholder="koodi" maxLength="8" />
            </label>

            <label class="dropdown">
                <select class="post-board" name="access_level">
                  <option value="" disabled selected>käyttäjätaso</option>
                  <For each={Object.entries(AccessLevel)}>
                    { ([key, value]) => <option value={value}>{key}</option>}
                  </For>
                </select>
                <svg class="drop-icon" xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path d="M480-360 280-560h400L480-360Z"/></svg>
            </label>

            <label>
              <input class="input" type="number" name="bump_limit" placeholder="nostoraja" min="0" max="1000" />
            </label>

            <label>
              <input type="checkbox" name="nsfw" value="1" />
              nsfw
            </label>

            <button class="end">
              <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="M120-160v-640l760 320-760 320Zm80-120 474-200-474-200v140l240 60-240 60v140Zm0 0v-400 400Z"/></svg>
            </button>
          </form>
        </Modal>
      </Show>
    </div>
  )
}