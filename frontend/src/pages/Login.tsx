import { JSX, Show } from 'solid-js'
import space from "../assets/12397866.435000004_space.jpg";
import './Login.css'
import { A, Navigate } from '@solidjs/router';
import { setState, state } from '..';
import { AccessLevel } from '../scripts/user';
import { startSession, userSession } from '../scripts/session';

export function Login() {
  const loginHandler: JSX.EventHandlerUnion<HTMLFormElement, Event> = (e) => {
    e.preventDefault();

    let data = Object.fromEntries(new FormData(e.target as HTMLFormElement));

    // TODO: handle emails
    startSession({
      username: data.username.toString(),
      password: data.password.toString(),
    })
    .then((res) => {
      if (res === 200) {
        setState("session", userSession());
      }
    });
  }

  return (
    <Show
      when={state.session && state.session.role === AccessLevel.Anonymous}
      fallback={ //TODO: redirect to last page
        <Navigate href={"/"} />
      }
    >
    <div class="login-page">
      <div class="login-wrap">
        <header class="login-page-header">
          <h2>Login</h2>
          <p>Kapchan user login. Available automatically after 
          <A href="/apply">applying</A>.</p>
        </header>
      
        <form class="login-form" onSubmit={loginHandler}>
          <input 
            class="text-field" 
            type="text"
            name="username" 
            placeholder="Username or Email"
          />

          <input 
            class="text-field" 
            type="password"
            name="password" 
            placeholder="Password"
          />
          <button class="login-btn" type="submit">Login</button> 
        </form>
      </div>

      <img class="login-img" src={space} alt="" />
    </div>
    </Show>
  )
}