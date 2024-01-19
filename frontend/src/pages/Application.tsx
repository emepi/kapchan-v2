import { JSX } from "solid-js";
import './Application.css'


export function Application() {
    const applyHandler: JSX.EventHandlerUnion<HTMLFormElement, Event> = (e) => {
      e.preventDefault();
    
      //let data = new FormData(e.target as HTMLFormElement);
    }

    return (
      <div class="apply-page">
        <section class="apply-head">
          <h2>Apply for kapchan membership</h2>
          <p>todo</p>
        </section>
        <form class="apply-form" onSubmit={applyHandler}>
          <label for="username">Pick an username:</label>
          <input
            id="username"
            class="text-field" 
            type="text"
            name="username" 
            placeholder="Username"
          />
          
          <label for="email">Email address:</label>
          <input
            id="email"
            class="text-field" 
            type="text"
            name="email" 
            placeholder="Email"
          />

          <label for="password">Password:</label>
          <input
            id="password"
            class="text-field" 
            type="password"
            name="password" 
            placeholder="Password"
          />

          <label for="background">Background:</label>
          <textarea id="background" name="background" rows="10" cols="30">

          </textarea>

          <label for="motivation">Motivation:</label>
          <textarea id="motivation" name="motivation" rows="10" cols="30">

          </textarea>

          <label for="referrer">Referrer:</label>
          <textarea id="referrer" name="referrer" rows="10" cols="30">

          </textarea>

          <button class="apply-btn" type="submit">Submit Application</button> 
        </form>
      </div>
    )
}