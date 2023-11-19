export interface ServiceFrame {
    s: Number, // service id
    r: ServiceRequestFrame | ServiceResponseFrame, 
}

export interface ServiceRequestFrame {
    t: Number, // service method id
    b: string, // service request body
}

export interface ServiceResponseFrame {
    t: Number, // service method id
    c: Number, // response code
    b: string, // response body
}