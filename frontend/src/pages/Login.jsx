import { Show, createSignal } from 'solid-js';
import s_img from '../assets/12397866.435000004_space.jpg'
import { validateEmail } from '../scripts/utils';
import { session, updateSession } from '..';
import { startSession, userSession } from '../scripts/session';
import { AccessLevel } from '../scripts/user';
import { Navigate } from '@solidjs/router';

export const Login = () => {
  const [error, setError] = createSignal(false)
  const [msg, setMsg] = createSignal("")

  const loginHandler = (e) => {
    e.preventDefault()

    const data = Object.fromEntries(new FormData(e.target))
    const id = data.id.toString()
    const password = data.password.toString()

    if (!id) {
      showErr("Syötä käyttäjänimi tai sähköpostiosoite")
    } 
    else if (!password) {
      showErr("syötä salasana")
    } 
    else {
      const login = validateEmail(id) ?  {
        email: id,
        password: password,
      } : {
        username: id,
        password: password,
      }

      startSession(login)
      .then(status => {
        switch (status) {
          case 201:
            updateSession(userSession())
            break;
          case 404:
            showErr("Käyttäjää ei löydy")
            break;
          case 401:
            showErr("Salasana on virheellinen")
            break;
          default:
            showErr("Yhteys ongelma")
        }
      })
    }
  }

  const showErr = (error_msg) => {
    setError(true)
    setMsg(error_msg)
  }

  return (
    <div class="login">
      <Show when={session() && session().role > AccessLevel.Anonymous}>
        <Navigate href="/" />
      </Show>
      <div class="login-cont">
        <div class="login-side">
          <h2>Kirjaudu</h2>
          <form class="login-form" onSubmit={loginHandler}>
          <Show when={error()}>
            <p class="error-msg">{msg()}</p>
          </Show>
          <input 
            class="text-field" 
            type="text"
            name="id" 
            placeholder="Käyttäjänimi tai Sähköpostiosoite"
          />

          <input 
            class="text-field" 
            type="password"
            name="password" 
            placeholder="Salasana"
          />
          <div class="login-select">
            <button class="forgot-btn">
                Unohtuiko salasana?
            </button>
            <button class="login-btn" type="submit">
              Kirjaudu sisään
              <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z"/></svg>
            </button> 
          </div>
        </form>
        </div>
        <img class="login-img" src={s_img} />
      </div>
    </div>
  );
}