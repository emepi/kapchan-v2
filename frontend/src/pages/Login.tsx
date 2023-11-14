import { JSX } from 'solid-js'
import { serviceRequest } from '../scripts/connection_manager';
import './Login.css'

export function Login() {
  const loginHandler: JSX.EventHandlerUnion<HTMLFormElement, Event> = (e) => {
    e.preventDefault();

    let data = new FormData(e.target as HTMLFormElement);

    serviceRequest(1, JSON.stringify(Object.fromEntries(data)));
  }

  return (
    <div class="login-page">
      <h2>Login</h2>
      <p>todo...</p>

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
  )
}