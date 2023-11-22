import { Service, serviceRequest } from "../scripts/connection_manager";
import { UserServiceType } from "../scripts/user_service";

export function Admin() {
    let applications = () => {

      serviceRequest(Service.UserService, {
        t: UserServiceType.FetchApplications,
        b: JSON.stringify({
          accepted: false,
          handled: false,
        })
      }, printApplication
      );
    };

    let printApplication = (b: string) => {
      console.log(JSON.parse(b));
    };

    return (
        <div class="admin-page">
          <section>
            <h2>Administration</h2>
            <p>todo..</p>
          </section>
          <button onClick={applications}>Get applications</button>
        </div>
    )
}