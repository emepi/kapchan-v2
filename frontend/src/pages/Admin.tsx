import { JSX, Show, createSignal } from 'solid-js';
import './Admin.css';
import { UserBrowser } from '../components/UserBrowser';


export function Admin() {

  const [view, setView] = createSignal("users");

  const selectView: JSX.ChangeEventHandlerUnion<HTMLInputElement, Event> = (e) => {
    const input: HTMLInputElement = e.target;
    setView(input.value);
  };

  return (
    <div class="admin-page">
      <div class="admin-view-select">
        <div>
          <input type="radio" onChange={selectView} id="users" name="view" value="users" checked />
          <label class="view-select-btn" for="users">users</label>
        </div>
        <div>
          <input type="radio" onChange={selectView} id="applications" name="view" value="applications" />
          <label class="view-select-btn" for="applications">applications</label>
        </div>
        <div>
          <input type="radio" onChange={selectView} id="boards" name="view" value="boards" />
          <label class="view-select-btn" for="boards">boards</label>
        </div>
      </div>

      <div class="admin-view">
        <Show when={view() === 'users'}>
          <UserBrowser />
        </Show>
        <Show when={view() === 'applications'}>
          <p>applications</p>
        </Show>
        <Show when={view() === 'boards'}>
          <p>boards</p>
        </Show>
      </div>
    </div>
  );
}