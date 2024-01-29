import { credentials } from "./credentials"


let conn_timeout = 0;
let conn_timeout_max = 15000;

export const apiFetch = async (resource, init, auth = false) => {
  if (auth) {
    if (credentials.auth_token) {
      const headers = new Headers(init.headers)
      headers.append("Authorization","Bearer " + credentials.auth_token)
      init.headers = headers;
    } else {
      setTimeout(() => {

        conn_timeout = (conn_timeout >= conn_timeout_max) ? conn_timeout * 2 + 1000 : 0;
        apiFetch(resource, init, auth) 
      }, conn_timeout)
    }
  }

  return fetch(new Request(resource, init))
}