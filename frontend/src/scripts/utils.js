export const parseJWT = (jwt) => {
  return JSON.parse(atob(jwt.split('.')[1]))
}

export const validateEmail = (email) => {
  return String(email)
  .toLowerCase()
  .match(/^\S+@\S+\.\S+$/)
}

export const formatClass = (c, space) => {
  return c ? (space ? " " + c : c) : ""
}