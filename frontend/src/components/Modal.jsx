export const Modal = (props) => {
  return (
    <div class="modal-bg">
      <div class="modal">
        <header class="modal-head">
          <h3>{props.title}</h3>
          <button class="close-btn" onClick={props.onClose}>
            <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"/></svg>
          </button>
        </header>
        {props.children}
      </div>
    </div>
  )
}