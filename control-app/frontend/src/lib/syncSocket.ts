export const connect = (onmsg_callback: (msg: any) => void) : WebSocket => {
    let url = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/api/ws';
    console.log('Connecting WS to', url);
    const reconnectDelay = 1500;

    let ws = new WebSocket(url);

    ws.onopen = () => {
        console.log('WS connected');
    };

    ws.onmessage = (ev) => {
        console.log('WS message', ev.data);
        onmsg_callback(ev);
    };

    ws.onclose = () => {
        console.log('WS closed, reconnecting in', reconnectDelay);
    };

    ws.onerror = () => {
        console.log('WS error, closing');
        ws?.close();
    };

    return ws;
}
