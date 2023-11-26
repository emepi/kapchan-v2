import { For } from "solid-js";
import { Service, serviceRequest } from "../scripts/connection_manager";
import { UserServiceType } from "../scripts/user_service";
import { createStore } from "solid-js/store";
import './ApplicationBrowser.css'

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

  const fetchApplications = () => {

    serviceRequest(Service.UserService, {
      t: UserServiceType.FetchApplications,
      b: JSON.stringify({
        accepted: false,
        handled: false,
      })
    }, updateApplications);
  };

  const updateApplications = (b: string) => {
    let applications = JSON.parse(b);

    setApplications(applications);
    console.log(applications);
  };

  fetchApplications();

  return (
    <div class="apli-browser">
        <h3>Applications</h3>
        <button onClick={fetchApplications}>Refresh</button>

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
                <button>
                  Accept
                </button>
                <button>
                  Decline
                </button>
              </div>
            </div>
          )}
        }</For>
    </div>
  )
}