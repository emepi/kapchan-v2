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

export function userServiceReceive(input: ServiceFrame) {
    console.log("User services received input: ", input);
}