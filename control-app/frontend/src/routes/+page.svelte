<script lang="ts">
	import { browser } from '$app/environment';

	// Svelte 5 runes
	let val = $state(0);
	let rev = $state(0);

	let wsUrl: string | null = null;
	if (browser) {
		wsUrl = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/api/ws';
	}

	let ws: WebSocket | null = null;
	let reconnectDelay = 300;

	function connect() {
        if (!wsUrl) return;

        console.log('Connecting WS to', wsUrl);

		ws = new WebSocket(wsUrl);

		ws.onopen = () => {
			reconnectDelay = 300;
            console.log('WS connected');
		};

		ws.onmessage = (ev) => {
			const msg = JSON.parse(ev.data);
			if ((msg.type === 'slider' || msg.type === 'hello') && msg.rev > rev) {
				rev = msg.rev;
				val = msg.value; // server is authoritative
			}
		};

		ws.onclose = () => {
            console.log('WS closed, reconnecting in', reconnectDelay);
			setTimeout(connect, (reconnectDelay = Math.min(2000, reconnectDelay * 1.5)));
		};
		ws.onerror = () => {
            console.log('WS error, closing');
            ws?.close();
        }
	}

	// Start/cleanup WS
	$effect(() => {
		connect();
		return () => ws?.close();
	});

	// Debounced sender while dragging
	let t: ReturnType<typeof setTimeout> | null = null;
	function sendProposed(v: number) {
		if (!ws || ws.readyState !== WebSocket.OPEN) {
            console.log('WS not open, not sending', v);
            return;
        }

        console.log('send proposed', v, rev);
		ws.send(JSON.stringify({ type: 'set_slider', value: v, rev }));
	}
	function onInput(e: Event) {
		const v = Number((e.target as HTMLInputElement).value);
		// local optimism: the UI moves immediately
		val = v;
		if (t) clearTimeout(t);
		t = setTimeout(() => sendProposed(v), 60);
	}
</script>

<input
	class="range range-primary"
	type="range"
	min="0"
	max="100"
	step="1"
	bind:value={val}
	oninput={onInput}
/>

<p>Value: {val} (rev {rev})</p>
<img src="/api/stream" alt="Microscope camera stream" />
