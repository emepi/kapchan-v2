enum ConnectionStatus {
    Uninitialized = 0,
    Ready = 1,
}

export const connection_manager = {
    socket: connect("ws://127.0.0.1:8080/ws"),
    status: ConnectionStatus.Uninitialized,
}

function connect(addr: string): WebSocket {
    let socket = new WebSocket(addr);
    socket.onopen = onOpen;
    socket.onerror = onError;
    socket.onmessage = onMessage;
    socket.onclose = onClose;

    return socket;
}

function onOpen(e: Event) {
    connection_manager.status = ConnectionStatus.Ready;
    
    console.log("Connected to server: ", e);
}

function onError(e: Event) {
    console.log("Connection to server was closed due to an error: ", e);
}

function onMessage(e: MessageEvent) {
    console.log("Message was received: ", e);
}

function onClose(e: CloseEvent) {
    console.log("Connection to server was closed: ", e);
}