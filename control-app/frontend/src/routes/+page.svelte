<script lang="ts">
	import { connect } from '$lib/syncSocket';

	let exposure_time = $state(0);
	let test_pattern = $state('smpte');

	let ws: WebSocket | null = null;

	$effect(() => {
		ws = connect(onMessageCallback);
		return () => ws?.close();
	});

	const onMessageCallback = (event: MessageEvent) => {
		let data = JSON.parse(event.data);
		exposure_time = data.camera_properties.exposure_time ?? exposure_time;
		test_pattern = data.camera_properties.test_pattern ?? test_pattern;
	};

	// Debounced sender while dragging
	let t: ReturnType<typeof setTimeout> | null = null;

	const sendCameraProperty = (t: string, v: any) => {
		if (!ws || ws.readyState !== WebSocket.OPEN) {
			console.log('WS not open, not sending', v);
			return;
		}

		ws.send(
			JSON.stringify({
				camera_properties: { [t]: v }
			})
		);
	};

	const updateParam = (e: Event, param: string) => {
		let v;

		switch (param) {
			case 'exposure_time':
				v = Number((e.target as HTMLInputElement).value);
				if (v < 0 || v > 30000 || isNaN(v)) return;
				exposure_time = v;
				break;
			case 'test_pattern':
				v = (e.target as HTMLInputElement).value;
				if (!['smpte', 'snow', 'ball'].includes(v.toString())) return;
				test_pattern = v.toString();
				break;
			default:
				return;
		}

		if (t) clearTimeout(t);
		t = setTimeout(() => sendCameraProperty(param, v), 50);
	};
</script>

<input
	class="range range-primary"
	type="range"
	min="0"
	max="30000"
	step="1"
	bind:value={exposure_time}
	oninput={(e) => updateParam(e, 'exposure_time')}
	onmousedown={() => (t = null)}
/>

<select bind:value={test_pattern} onchange={(e) => updateParam(e, 'test_pattern')}>
	<option value="smpte">SMPTE</option>
	<option value="snow">Snow</option>
	<option value="ball">Ball</option>
</select>

<p>Value: {exposure_time}</p>
<img src="/api/stream" alt="Microscope camera stream" />
