export const connect = (onmsg_callback: (msg: any) => void): WebSocket => {
    let url = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/api/ws';
    const reconnectDelay = 1500;

    let ws = new WebSocket(url);

    ws.onopen = () => {
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
