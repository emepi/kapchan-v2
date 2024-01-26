export const parseJWT = (jwt) => {
  return JSON.parse(atob(jwt.split('.')[1]))
}

export const formatClass = (c, space) => {
  return c ? (space ? " " + c : c) : ""
}