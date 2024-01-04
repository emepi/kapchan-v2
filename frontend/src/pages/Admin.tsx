import { ApplicationBrowser } from "../components/ApplicationBrowser";
import { BoardBrowser } from "../components/BoardBrowser";


export function Admin() {

    return (
        <div class="admin-page">
          <section>
            <h2>Administration</h2>
            <p>todo..</p>
          </section>
          <ApplicationBrowser />
          <BoardBrowser />
        </div>
    )
}