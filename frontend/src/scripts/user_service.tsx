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
    Application = 3,
    FetchApplications = 4,
    CloseApplication = 5,
}

let caller: Function;

export function userServiceCallback(callback?: Function) {
    if (callback) {
        caller = callback;
    }
}

export function userServiceReceive(input: ServiceResponseFrame) {

    console.log("User services received input: ", input);

    let token;

    switch (input.t) {
        case UserServiceType.Login:
            token = input.b;

            if (input.c === ResponseCode.Success && token) {
                setCookie("access_token", token);
                setState({user: cookieSession()});
            }

            break;
        
        case UserServiceType.Logout:
            logout();
            break;

        case UserServiceType.Application:
            token = input.b;

            if (input.c === ResponseCode.Success && token) {
                setCookie("access_token", token);
                setState({user: cookieSession()});
            }
            break;
        
        case UserServiceType.FetchApplications:

            if (caller) {
                caller(input.b);
            }

            break;
        
        case UserServiceType.CloseApplication:

            break;
        
        default:
            break;
    }
}