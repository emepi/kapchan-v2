import { credentials } from "./credentials";


const api = "http://127.0.0.1";


export const apiFetch = async (resource: string, init: RequestInit): Promise<Response> => {
  // Authenticate the user if a token is available.
  if (credentials.auth_token) {
    const headers = new Headers(init.headers);
    headers.append("Authorization","Bearer " + credentials.auth_token);

    init.headers = headers;
  }

  return fetch(new Request(api + resource, init));
}