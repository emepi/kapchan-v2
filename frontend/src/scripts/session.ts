import { apiFetch } from "./connection";
import { credentials } from "./credentials";
import { parseJWT } from "./utils";


export interface LoginInfo {
  email?: string,
  username?: string,
  password: string,
}

interface SessionResponse {
  access_token: string,
}

export const startSession = async (user?: LoginInfo): Promise<number> => {
  return apiFetch("/sessions", {
    method: "POST",
    body: (user?.email || user?.username) ? JSON.stringify(user) : undefined,
  })
  .then(async (res) => {
    if (res.ok) {
      await res.json()
      .then((data: SessionResponse) => replaceSession(data.access_token));
    }

    return res.status;
  })
  .catch((err) => {
    console.log(err);
    return 503; // Service unavailable (503) due to a network error.
  });
};

export const replaceSession = (token: string) => {
  credentials.auth_token = token;
  localStorage.setItem("session", token);
};

export const removeSession = () => {
  credentials.auth_token = undefined;
  localStorage.removeItem("session");
}

export const loadSession = async () => {
  const token = localStorage.getItem("session");

  if (token) {
    credentials.auth_token = token;
  } else {
    await startSession();
  }
}

export const endSession = async () => {
  apiFetch("/sessions/" + userSession()?.sub, {
    method: "PUT",
    body: JSON.stringify({
      continue_session: false,
    }),
  })
}

export interface Session {
  exp: number,          // Expiration time (as UTC timestamp)
  iat: number,          // Issued at (as UTC timestamp)
  sub: string,          // Subject (session id)
  role: number,         // Session access level
}

export const userSession = (): Session | undefined => {
  if (credentials.auth_token) {
    return parseJWT(credentials.auth_token)
  }

  return undefined;
}