import { JSX, Show } from 'solid-js'
import space from "../assets/12397866.435000004_space.jpg";
import './Login.css'
import { state } from '..';
import { UserRole } from '../scripts/user';
import { Navigate } from '@solidjs/router';

export function Login() {
  const loginHandler: JSX.EventHandlerUnion<HTMLFormElement, Event> = (e) => {
    e.preventDefault();

    let data = new FormData(e.target as HTMLFormElement);
  }

  return (
    <Show
      when={state.user.role === UserRole.Anonymous}
      fallback={ //TODO: redirect to last page
        <Navigate href={"/"} />
      }
    >
    <div class="login-page">
      <div class="login-wrap">
        <header class="login-page-header">
          <h2>Login</h2>
          <p>todo...</p>
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