<script lang="ts">
	import { connect } from '$lib/syncSocket';
	import SliderWithTextbox from '$lib/components/sliderWithTextbox.svelte';

	let exposure_time = $state(0);
	let brightness = $state(0);
	let contrast = $state(0);
	let saturation = $state(0);

	let ws: WebSocket | null = null;

	$effect(() => {
		ws = connect(onMessageCallback);
		return () => ws?.close();
	});

	const onMessageCallback = (event: MessageEvent) => {
		let data = JSON.parse(event.data);
		exposure_time = data.camera_properties.exposure_time ?? exposure_time;
		brightness = data.camera_properties.brightness ?? brightness;
		contrast = data.camera_properties.contrast ?? contrast;
		saturation = data.camera_properties.saturation ?? saturation;
	};

	let debounce_timeout: ReturnType<typeof setTimeout> | null = null;

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

	const updateParam = (value: any, param: string) => {
		let v: any = null;

		switch (param) {
			case 'exposure_time':
				v = Number(value);
				exposure_time = v;
				break;
			case 'brightness':
				v = Number(value);
				brightness = v;
				break;
			case 'contrast':
				v = Number(value);
				contrast = v;
				break;
			case 'saturation':
				v = Number(value);
				saturation = v;
				break;
			default:
				return;
		}

		if (debounce_timeout) {
			clearTimeout(debounce_timeout);
		}
		debounce_timeout = setTimeout(() => sendCameraProperty(param, v), 50);
	};
</script>

<div class="drawer drawer-end lg:drawer-open">
	<input id="my-drawer-2" type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col items-center justify-center">
		<img src="/api/stream" alt="Microscope camera stream" />
		<label for="my-drawer-2" class="btn btn-primary drawer-button lg:hidden"> Open drawer </label>
	</div>
	<div class="drawer-side">
		<label for="my-drawer-2" aria-label="close sidebar" class="drawer-overlay"></label>
		<ul class="menu bg-base-200 text-base-content min-h-full w-100 p-4">
			<div class="card bg-base-100 shadow-sm">
				<div class="card-body">
					<h2 class="card-title">Camera Controls</h2>
					<SliderWithTextbox
						label="Exposure Time (µs)"
						value={exposure_time}
						min={0}
						max={30000}
						step={1}
						onChange={(v: number) => updateParam(v, 'exposure_time')}
					/>

					<SliderWithTextbox
						label="Brightness"
						value={brightness}
						min={-1}
						max={1}
						step={0.01}
						onChange={(v: number) => updateParam(v, 'brightness')}
					/>

					<SliderWithTextbox
						label="Contrast"
						value={contrast}
						min={0}
						max={5}
						step={0.01}
						onChange={(v: number) => updateParam(v, 'contrast')}
					/>

					<SliderWithTextbox
						label="Saturation"
						value={saturation}
						min={0}
						max={5}
						step={0.01}
						onChange={(v: number) => updateParam(v, 'saturation')}
					/>
				</div>
			</div>
		</ul>
	</div>
</div>
