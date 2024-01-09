export const parseJWT = (jwt: string) => {
    return JSON.parse(atob(jwt.split('.')[1]));
}