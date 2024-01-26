import { Show, createSignal } from "solid-js"
import { RadioButton } from "../components/RadioButton"
import { AccessLevel } from "../scripts/user"
import { Portal } from "solid-js/web"


export const Board = () => {
  return (
    <>
      <nav class="board-selector">
        <div class="navbar">
          <RadioButton name="board" accessLevel={AccessLevel.Anonymous} checked>etusivu</RadioButton>
          <RadioButton name="board" accessLevel={AccessLevel.Anonymous}>satunnainen</RadioButton>
          <RadioButton name="board" accessLevel={AccessLevel.Anonymous}>anime</RadioButton>
          <RadioButton name="board" accessLevel={AccessLevel.Anonymous}>pelit</RadioButton>
          <RadioButton name="board" accessLevel={AccessLevel.Anonymous}>teknologia</RadioButton>
          <RadioButton name="board" accessLevel={AccessLevel.Member}>hikky</RadioButton>
        </div>
        <PostingModalButton />
      </nav>
    </>
  )
}

const PostingModalButton = (props) => {

  const [open, setOpen] = createSignal(false)

  return(
    <div>
      <button onClick={() => setOpen(true)} class="radio-lbl">
        luo lanka
      </button>
      <Show when={open()}>
        <Portal>
          <div class="post-modal">
            posting..
          </div>
        </Portal>
      </Show>
    </div>
  )
}