import { Show, createResource, createSignal } from "solid-js"
import { RadioButton } from "../components/RadioButton"
import { AccessLevel } from "../scripts/user"
import { credentials } from "../scripts/credentials"
import { session, startSession } from "../scripts/user_service"
import logo from"../assets/logo5.png"


const [boards] = createResource(async () => (await fetch("/boards")).json())


export const Board = () => {
  const [view, changeView] = createSignal("");

  const selectView = (e) => {
    const input = e.target
    changeView(input.value)
  }

  return (
    <>
      <nav class="board-selector">
        <div class="navbar">
          <RadioButton name="board" onChange={selectView} accessLevel={AccessLevel.Anonymous} value="" checked>etusivu</RadioButton>
          <For each={boards()}>
            { (board) => <RadioButton name="board" onChange={selectView} accessLevel={board.access_level} value={board.handle}>{board.title}</RadioButton> }
          </For>
        </div>
        <PostingModalButton />
      </nav>
      <Show when={view() === ""}>
        <div class="banner">
          <div class="banner-cont">
            <img class="logo" src={logo} />
            <h1 class="intro-h">Avaruuskapakka</h1>
            <div class="intro">
              <span class="ward">„eirum aum aurum . . . normot poistum"</span>
              <p>Kapakka on kuvalauta/chat syrjäytyneille ja muille normaalista poikkeaville ihmisille.</p>
              <p>Sisäänkirjautuneena pääset useammalle alueelle ja saat muita ominaisuuksisa käyttöösi. <a>Rekisteröidy tästä.</a></p>
            </div>
          </div>
        </div>
      </Show>
    </>
  )
}

const PostingModalButton = (props) => {

  const [open, setOpen] = createSignal(false)

  const post = async (e) => {
    e.preventDefault()
    const data = new FormData(e.target)

    if (!credentials.access_token) {
      await startSession()
    }

    if (credentials.access_token) {
      fetch("/posts", {
        method: "POST",
        headers: [["Authorization", "Bearer " + credentials.access_token]],
        body: data,
      })
    }
  }

  return(
    <div>
      <button onClick={() => setOpen(true)} class="radio-lbl">
        luo lanka
      </button>
      <Show when={open()}>
        <div class="post-modal-bg">
          <div class="post-modal">
            <header class="modal-head">
              <h3>Luo lanka</h3>
              <button class="close-btn" onClick={() => setOpen(false)}>
                <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"/></svg>
              </button>
            </header>
            <form class="post-form" onSubmit={post}>
              <label>
                <input class="post-subject" type="text" name="subject" placeholder="Otsikko" maxLength="100" />
              </label>
              <label class="dropdown">
                <select class="post-board" name="board">
                  <option value="" disabled selected>Valitse lauta</option>
                  <For each={boards()}>
                    { (board) => 
                    <Show 
                      when={session() && session().role >= board.access_level}
                      fallback={<option disabled>{board.title}</option>}
                    >
                      <option value={board.handle}>{board.title}</option>
                    </Show>
                    }
                  </For>
                </select>
                <svg class="drop-icon" xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path d="M480-360 280-560h400L480-360Z"/></svg>
              </label>
              <textarea class="post-body" name="body" placeholder="Sisältö" rows="15" cols="80" maxLength="40000"/>
              <div class="post-selector board-selector">
                <label>
                  <input type="file" name="attachment" accept="image/png, image/jpeg" hidden/>
                  <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="M720-330q0 104-73 177T470-80q-104 0-177-73t-73-177v-370q0-75 52.5-127.5T400-880q75 0 127.5 52.5T580-700v350q0 46-32 78t-78 32q-46 0-78-32t-32-78v-370h80v370q0 13 8.5 21.5T470-320q13 0 21.5-8.5T500-350v-350q-1-42-29.5-71T400-800q-42 0-71 29t-29 71v370q-1 71 49 120.5T470-160q70 0 119-49.5T640-330v-390h80v390Z"/></svg>
                </label>
                <button>
                  <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="M120-160v-640l760 320-760 320Zm80-120 474-200-474-200v140l240 60-240 60v140Zm0 0v-400 400Z"/></svg>
                </button>
              </div>
            </form>
          </div>
        </div>
      </Show>
    </div>
  )
}