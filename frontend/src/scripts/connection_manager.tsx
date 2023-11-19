import { setState } from "..";
import { cookieSession, eraseCookie } from "./cookies";
import { userServiceReceive } from "./user_service";
import { 
  ServiceFrame, 
  ServiceRequestFrame, 
  ServiceResponseFrame 
} from "./service";

enum ConnectionStatus {
  Uninitialized = 0,
  Ready = 1,
  Closed = 2,
}

enum CloseCode {
  InvalidSession = 1,
}

export enum Service {
  UserService = 1,
}

const TIMEOUT_DEFAULT = 1000;

const connection_manager = {
  socket: connect("ws://127.0.0.1:8080/ws"),
  status: ConnectionStatus.Uninitialized,
  channels: new Map([
    [1, userServiceReceive],
  ]),
  timeout: TIMEOUT_DEFAULT,
  timeoutMax: 625000,
  timeoutMult: 5,
}

function connect(addr: string): WebSocket {
    let socket = new WebSocket(addr);
    socket.onopen = onOpen;
    socket.onerror = onError;
    socket.onmessage = onMessage;
    socket.onclose = onClose;

    return socket;
}

function reconnect() {

  console.log(
    "Attempting reconnection in: " + 
    (connection_manager.timeout / 1000) + 
    " seconds. Refresh the page if you wish to skip this timer."
  );

  setTimeout(() => {
    connection_manager.socket = connect("ws://127.0.0.1:8080/ws");

  }, connection_manager.timeout);

  connection_manager.timeout = connection_manager.timeout * 
    connection_manager.timeoutMult;

  if (connection_manager.timeout > connection_manager.timeoutMax) {
    connection_manager.timeout = TIMEOUT_DEFAULT;
  }
}

function onOpen(e: Event) {
    connection_manager.status = ConnectionStatus.Ready;
    setState({user: cookieSession()});
    
    console.log("Connected to server: ", e);
}

function onError(e: Event) {
    console.log("Connection to server was closed due to an error: ", e);
}

function onMessage(e: MessageEvent) {
  connection_manager.timeout = 0; // TODO: set on init message

  let frame: ServiceFrame = JSON.parse(e.data);

  switch (frame.s) {
    case Service.UserService:
      let channel = connection_manager.channels.get(Service.UserService);

      if (channel) {
        channel(frame.r as ServiceResponseFrame);
      }

      break;
        
    default:
      console.error("Unimplemented service response received: ", frame);
      break;
    }
}

function onClose(e: CloseEvent) {
  console.log("Connection to server was closed: ", e);
  connection_manager.status = ConnectionStatus.Closed;

  let close_code = parseInt(e.reason);

  switch (close_code) {
    case CloseCode.InvalidSession:
      console.error(
        "Client is using an invalid access token.\n",
        "", // TODO: instructions for handling invalidation request
        );
      console.log("Cleaning access token and reconnecting..");

      eraseCookie("access_token");
      reconnect();
      break;

    default:
      break;
  }
}

export function serviceRequest(service_id: Number, request: ServiceRequestFrame) {
    if (connection_manager.status === ConnectionStatus.Ready) {
        let frame: ServiceFrame = {
            s: service_id,
            r: request,
        }
        
        connection_manager.socket.send(JSON.stringify(frame));
    }
}