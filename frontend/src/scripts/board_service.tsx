import { ServiceResponseFrame } from "./service";

export enum BoardServiceType {
    CreateBoard = 1,
}

let caller: Function;

export function boardServiceCallback(callback?: Function) {
    if (callback) {
        caller = callback;
    }
}

export function boardServiceReceive(input: ServiceResponseFrame) {

    console.log("User services received input: ", input);

    let token;

    switch (input.t) {
        case BoardServiceType.CreateBoard:
            token = input.b;

            break;
        
        default:
            break;
    }
}