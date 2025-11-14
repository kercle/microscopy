import type { WebSocketMessage } from "./bindings/WebSocketMessage";

export const connect = (onmsg_callback: (msg: any) => void): WebSocket => {
    let url = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/api/ws';
    const reconnectDelay = 1500;

    let ws = new WebSocket(url);

    ws.onopen = () => {
        let msg: WebSocketMessage = "register_user_client";
        ws.send(JSON.stringify(msg))
    };

    ws.onmessage = (ev) => {
        onmsg_callback(ev);
    };

    ws.onclose = () => {
    };

    ws.onerror = () => {
        ws?.close();
    };

    return ws;
}
