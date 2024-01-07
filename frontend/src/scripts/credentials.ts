import { apiFetch } from "./connection";

interface Credentials {
  auth_token: string | undefined,
}

interface SessionResponse {
  access_token: string,
}


export const credentials: Credentials = {
  auth_token: undefined,
};

export const sessionRequest = () => {
  apiFetch("/sessions", {
    method: "POST",
  })
  .then((res) => res.json())
  .then((data: SessionResponse) => {
    credentials.auth_token = data.access_token;
    localStorage.setItem("auth_token", data.access_token);
  });
};

export const parseJWT = (jwt: string) => {
    return JSON.parse(atob(jwt.split('.')[1]));
}

export const userSession = () => {
    if (credentials.auth_token) {
        let token_data = parseJWT(credentials.auth_token)

        return {
            role: token_data.role,
        }
    }

    return undefined;
}

document.addEventListener("DOMContentLoaded", (event) => {
  // Check for existing auth token.
  const token = localStorage.getItem("auth_token");

  if (token) {
    credentials.auth_token = token;
  } else {
    sessionRequest();
  }
});