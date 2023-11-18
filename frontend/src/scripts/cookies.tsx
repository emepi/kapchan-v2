export function eraseCookie(name: string) {   
    document.cookie = name + 
    '=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/; SameSite=Strict;';  
}

export function setCookie(name: string, value: string) {
    document.cookie = name + '=' + value + 
    '; max-age=31536000; path=/; SameSite=Strict;';
}