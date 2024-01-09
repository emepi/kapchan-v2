import { credentials } from "./credentials";


export const apiFetch = async (resource: string, init: RequestInit): Promise<Response> => {
  const headers = new Headers(init.headers);
  headers.append("Content-type","application/json");

  // Authenticate the user if a token is available.
  if (credentials.auth_token) {
    headers.append("Authorization","Bearer " + credentials.auth_token);
  }

  init.headers = headers;

  return fetch(new Request(resource, init));
}