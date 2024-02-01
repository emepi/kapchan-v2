import { createSignal } from "solid-js"
import { credentials } from "./credentials"
import { parseJWT } from "./utils"
import { AccessLevel } from "./user"


/**
 * Attempts to prematurely end session and to discard user's access token.
 */
export const endSession = () => {
  // End server session.
  if (credentials.access_token) {
    const session = userSession()

    fetch("/sessions/" + session.sub, {
      method: "PUT",
      headers: [
        ["Content-Type", "application/json"],
        ["Authorization", "Bearer " + credentials.access_token]
      ],
      body: JSON.stringify({
        exp: Math.round(Date.now() / 1000)
      }),
    })
  }

  // Discard access token.
  credentials.access_token = undefined
  localStorage.removeItem("access_token")

  // Update UI.
  updateSession({ 
    role: AccessLevel.Anonymous 
  })
}

/**
 * Loads the previous session from the storage and checks expiry.
 * 
 * @returns {boolean} True if a valid session was found.
 */
export const loadSession = () => {
  const token = localStorage.getItem("access_token")

  if (token) {
    const timestamp = Math.round(Date.now() / 1000)
    const session = userSession();

    if (timestamp < parseJWT(token).exp) {
      credentials.access_token = token;
      return true;
    }
  }
  return false;
}

/**
 * Attempts to start a session with the server.
 * On success (201), updates the access_token in credentials & localStorage.
 *  
 * @param {Object} [user] Login information for user session. Malformatted user 
 *                        data is interpreted as undefined by server.
 * @param {string} [user.username] Use username to identify user.
 * @param {string} [user.email] Use email to identify user.
 * @param {string} user.password 
 * @returns {number} HTTP response status code.
 */
export const startSession = async (user) => {
  const res = await fetch("/sessions", {
    method: "POST",
    headers: [[ "Content-Type", "application/json" ]],
    body: user ? JSON.stringify(user) : undefined,
  })

  let access_token

  if (res.ok) {
    const data = await res.json()
    access_token = data.access_token
  }

  if (access_token) {
    credentials.access_token = access_token
    localStorage.setItem("access_token", access_token)
    updateSession(userSession())
  }

  return res.status
}

export const userSession = () => {
  if (credentials.access_token) {
    return parseJWT(credentials.access_token)
  }
    
  return undefined;
}

/* UI session state */
export const [session, updateSession] = createSignal(
  loadSession() ? userSession() : {
    // Defaulted to anon placeholder to save a session request.
    role: AccessLevel.Anonymous,
  }
)