export function connectSocket(url: string, onMsg: (data: any) => void) {
    let ws: WebSocket;
    let retry = 500;

    const open = () => {
        ws = new WebSocket(url);
        ws.onopen = () => { retry = 500; };
        ws.onmessage = (ev) => onMsg(JSON.parse(ev.data));
        ws.onclose = () => setTimeout(open, retry = Math.min(retry * 1.5, 5000));
        ws.onerror = () => ws.close();
    };
    open();

    return {
        send: (obj: any) => ws?.readyState === WebSocket.OPEN && ws.send(JSON.stringify(obj)),
    };
}
