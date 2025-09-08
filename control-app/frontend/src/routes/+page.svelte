<script lang="ts">
	import { connect } from '$lib/syncSocket';

	let exposure_time = $state(0);

	let ws: WebSocket | null = null;

	$effect(() => {
		let wsUrl = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/api/ws';
		ws = connect(wsUrl);
		return () => ws?.close();
	});

	// Debounced sender while dragging
	let t: ReturnType<typeof setTimeout> | null = null;

	function sendProposed(t: string, v: number) {
		if (!ws || ws.readyState !== WebSocket.OPEN) {
			console.log('WS not open, not sending', v);
			return;
		}

		let data: any = {};
		data[t] = v;
		ws.send(JSON.stringify(data));
	}

	function onInput(e: Event) {
		const v = Number((e.target as HTMLInputElement).value);
		exposure_time = v;
		if (t) clearTimeout(t);
		t = setTimeout(() => sendProposed('exposure_time', v), 60);
	}
</script>

<input
	class="range range-primary"
	type="range"
	min="0"
	max="100"
	step="1"
	bind:value={exposure_time}
	oninput={onInput}
/>

<p>Value: {exposure_time}</p>
<img src="/api/stream" alt="Microscope camera stream" />
