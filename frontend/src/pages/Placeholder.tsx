import { state } from ".."


function Placeholder() {
    return (
        <div class="placeholder-page">
          <section>
            <h2>Welcome to avaruuskapakka</h2>
            <p>We are in development!</p>
            <p>User data:</p>
            <p>role: {state.user.role.toString()}</p>
          </section>
        </div>
    )
}

export default Placeholder