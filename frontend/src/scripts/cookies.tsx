import { User } from "./user";

export function eraseCookie(name: string) {   
    document.cookie = name + 
    '=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/; SameSite=Strict;';  
}

export function setCookie(name: string, value: string) {
    document.cookie = name + '=' + value + 
    '; max-age=31536000; path=/; SameSite=Strict;';
}

export function getCookie(name: string): string {
    let a = `; ${document.cookie}`.match(`;\\s*${name}=([^;]+)`);
    return a ? a[1] : '';
}

export function cookieSession(): User | undefined {
    let jwt = getCookie("access_token");

    console.log(jwt);

    if (jwt) {
        let token_data = JSON.parse(atob(jwt.split('.')[1]));
        console.log(token_data);

        return {
            role: token_data.role,
        }
    }

    return undefined;
}