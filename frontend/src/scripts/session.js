import { apiFetch } from "./connection"
import { credentials } from "./credentials"
import { parseJWT } from "./utils"


export const loadSession = async () => {
  const token = localStorage.getItem("session")

  if (token) {
    credentials.auth_token = token;
  } else {
    await startSession()
  }
}

export const startSession = async (user) => {
  return apiFetch("/sessions", {
    method: "POST",
    body: user ? JSON.stringify(user) : undefined,
  })
  .then(async (res) => {
    if (res.ok) {
      const sess = await res.json()
      replaceSession(sess.access_token)
    }

    return res.status
  })
  .catch((err) => {
    console.log(err);
    return 503; // Service unavailable (503) due to a network error.
  })
}

const replaceSession = (token) => {
  credentials.auth_token = token
  localStorage.setItem("session", token)
}

export const userSession = () => {
    if (credentials.auth_token) {
      return parseJWT(credentials.auth_token)
    }
    
    return undefined;
  }