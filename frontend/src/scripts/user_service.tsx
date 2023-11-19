import { setState } from "..";
import { cookieSession, setCookie } from "./cookies";
import { ServiceResponseFrame } from "./service";
import { logout } from "./user";

enum ResponseCode {
    Success = 1,
    Failure = 2,
    NotFound = 3,
    NotAvailable = 4,
    NotAllowed = 5,
    Malformatted = 6,
    InvalidServiceType = 7,
}

export enum UserServiceType {
    Login = 1,
    Logout = 2,
}

export function userServiceReceive(input: ServiceResponseFrame) {

    console.log("User services received input: ", input);

    switch (input.t) {
        case UserServiceType.Login:
            let token = input.b;

            if (token) {
                setCookie("access_token", token);
                setState({user: cookieSession()});
            }

            break;
        
        case UserServiceType.Logout:
            logout();
            break;
        
        default:
            break;
    }
}