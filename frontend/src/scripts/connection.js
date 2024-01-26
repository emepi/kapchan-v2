import { credentials } from "./credentials"


export const apiFetch = async (resource, init) => {
  const headers = new Headers(init.headers)
  headers.append("Content-type","application/json")

  if (credentials.auth_token) {
    headers.append("Authorization","Bearer " + credentials.auth_token)
  }

  init.headers = headers;

  return fetch(new Request(resource, init))
}