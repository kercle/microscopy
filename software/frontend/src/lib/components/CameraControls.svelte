<script lang="ts">
	import SliderWithTextbox from '$lib/components/SliderWithTextbox.svelte';
	import type { State } from '$lib/state';

	let { appState = $bindable<State>() } = $props();

	type CameraProperties = {
		exposure_time?: number;
		brightness?: number;
		contrast?: number;
		saturation?: number;
		auto_white_balance?: boolean;
		white_balance_mode?: string;
		color_gain_red?: number;
		color_gain_blue?: number;
	};

	let camera_properties: CameraProperties = $state({});

	export const update = (data: any) => {
		if (data.camera_properties) {
			camera_properties = { ...camera_properties, ...data.camera_properties };
		}
	};

	let debounce_timeout: ReturnType<typeof setTimeout> | null = null;

	const sendCameraProperty = (t: string, v: any) => {
		if (!appState.ws || appState.ws.readyState !== WebSocket.OPEN) {
			console.log('WS not open, not sending', v);
			return;
		}

		appState.ws.send(
			JSON.stringify({
				camera_properties: { [t]: v }
			})
		);
	};

	const updateParam = (value: any, param: string) => {
		camera_properties[param as keyof CameraProperties] = value;

		if (debounce_timeout) {
			clearTimeout(debounce_timeout);
		}
		debounce_timeout = setTimeout(() => sendCameraProperty(param, value), 50);
	};
</script>

<div class="bg-base-100 border-base-300 collapse collapse-arrow border p-2">
	<input type="checkbox" />
	<div class="collapse-title font-semibold text-lg">Camera Controls</div>
	<div class="collapse-content text-sm flex flex-col gap-1">
		<SliderWithTextbox
			label="Exposure Time"
			unit="µs"
			value={camera_properties.exposure_time}
			min={0}
			max={30000}
			step={1}
			onChange={(v: number) => updateParam(v, 'exposure_time')}
		/>

		<SliderWithTextbox
			label="Brightness"
			value={camera_properties.brightness}
			min={-1}
			max={1}
			step={0.01}
			onChange={(v: number) => updateParam(v, 'brightness')}
		/>

		<SliderWithTextbox
			label="Contrast"
			value={camera_properties.contrast}
			min={0}
			max={5}
			step={0.01}
			onChange={(v: number) => updateParam(v, 'contrast')}
		/>

		<SliderWithTextbox
			label="Saturation"
			value={camera_properties.saturation}
			min={0}
			max={5}
			step={0.01}
			onChange={(v: number) => updateParam(v, 'saturation')}
		/>

		<div class="mt-3 flex">
			<span class="flex-grow">Auto White Balance</span>
			<input
				type="checkbox"
				checked={camera_properties.auto_white_balance}
				class="toggle toggle-sm flex-none"
				onchange={(e) => updateParam((e.target as HTMLInputElement).checked, 'auto_white_balance')}
			/>
		</div>

		<div class="mt-3 flex flex-col">
			<div class="flex-grow">White Balance Mode:</div>
			<select
				class="select select-ghost mt-2"
				bind:value={camera_properties.white_balance_mode}
				onchange={(e) => updateParam((e.target as HTMLSelectElement).value, 'white_balance_mode')}
			>
				<option value="auto">Auto</option>
				<option value="incandescent">Incandescent</option>
				<option value="tungsten">Tungsten</option>
				<option value="fluorescent">Fluorescent</option>
				<option value="indoor">Indoor</option>
				<option value="daylight">Daylight</option>
				<option value="cloudy">Cloudy</option>
				<option value="custom">Custom</option>
			</select>
		</div>

		<SliderWithTextbox
			label="Red Gain"
			value={camera_properties.color_gain_red}
			min={0}
			max={5}
			step={0.01}
			onChange={(v: number) => updateParam(v, 'color_gain_red')}
		/>

		<SliderWithTextbox
			label="Blue Gain"
			value={camera_properties.color_gain_blue}
			min={0}
			max={5}
			step={0.01}
			onChange={(v: number) => updateParam(v, 'color_gain_blue')}
		/>
	</div>
</div>
