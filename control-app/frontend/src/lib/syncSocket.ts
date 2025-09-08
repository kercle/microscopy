export function connect(url: string) : WebSocket {
    console.log('Connecting WS to', url);
    const reconnectDelay = 1500;

    let ws = new WebSocket(url);

    ws.onopen = () => {
        console.log('WS connected');
    };

    ws.onmessage = (ev) => {
        const msg = JSON.parse(ev.data);
        console.log('WS msg', msg);
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
