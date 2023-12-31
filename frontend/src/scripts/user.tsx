import { eraseCookie } from "./cookies"

export interface User {
    role: number,
}

export enum UserRole {
    Anonymous = 10,
    Member = 20,
    Admin = 100,
    Owner = 200,
}

export const anonUser: User = {
    role: UserRole.Anonymous,
}

export function logout() {
    //TODO: tell server to drop this session
    console.log("Erasing the current session.");
    eraseCookie("access_token");
    location.reload();
}