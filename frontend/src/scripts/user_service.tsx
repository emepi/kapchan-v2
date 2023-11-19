import { setCookie } from "./cookies";
import { ServiceFrame } from "./service";

enum ResponseCode {
    Success = 1,
    Failure = 2,
    NotFound = 3,
    NotAvailable = 4,
    NotAllowed = 5,
    Malformatted = 6,
    InvalidServiceType = 7,
}

enum ServiceType {
    Login = 1,
}

export function userServiceReceive(input: ServiceFrame) {

    let body = JSON.parse(input.b);

    console.log("User services received input: ", body);

    switch (input.t) {
        case ServiceType.Login:
            let token = body.m;

            let token_data = JSON.parse(atob(token.split('.')[1]));

            console.log(token_data);

            if (token) {
                setCookie("access_token", token);
            }

            break;
        
        default:
            break;
    }
}