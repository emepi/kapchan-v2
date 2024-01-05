import { For, JSX } from "solid-js";
import { createStore } from "solid-js/store";
import './ApplicationBrowser.css';

interface Application {
  application_id: number,
  background: string,
  created_at: string,
  email: string,
  motivation: string,
  other: string,
  user_id: number,
  username: string,
}

export function ApplicationBrowser() {
  const [applications, setApplications] = createStore([]);

  return (
    <div class="apli-browser">
        <h3>Applications</h3>

        <For each={applications} fallback={<p>loading..</p>}>{application => {
          let data = application as Application;

          return(
            <div class="apli-cont">
              <h4>
                {data.username}
              </h4>
              <p>
                {data.created_at}
              </p>
              <p>
                {data.email}
              </p>
              <p>
                {data.motivation}
              </p>
              <p>
                {data.background}
              </p>
              <p>
                {data.other}
              </p>
              <div class="apli-ctrl">
                <button 
                  data-id={data.application_id}
                  data-user={data.user_id}
                  data-resolution={true}
                >
                  Accept
                </button>
                <button
                  data-id={data.application_id}
                  data-user={data.user_id}
                  data-resolution={false}
                >
                  Decline
                </button>
              </div>
            </div>
          )}
        }</For>
    </div>
  )
}