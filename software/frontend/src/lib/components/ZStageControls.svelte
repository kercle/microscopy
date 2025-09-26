<script lang="ts">
	import SliderWithTextbox from './SliderWithTextbox.svelte';
	import { type State } from '$lib/state';

	let { appState = $bindable<State>() } = $props();
</script>

<div class="bg-base-100 border-base-300 collapse-arrow collapse border p-2">
	<input type="checkbox" checked />
	<div class="collapse-title text-lg font-semibold">Z-Stage Controls</div>
	<div class="collapse-content flex flex-col gap-1 text-sm">
		<h2 class="text-md mb-3 font-semibold">Program 1</h2>
		<div class="pl-2">
			<SliderWithTextbox
				label="Speed"
				unit="steps/s"
				value={appState.zStage.program_1.speed}
				min={100}
				max={1400}
				onChange={(v: number) => (appState.zStage.program_1.speed = v)}
			/>
			<SliderWithTextbox
				label="Distance"
				unit="steps"
				value={appState.zStage.program_1.distance}
				min={1}
				max={500}
				onChange={(v: number) => (appState.zStage.program_1.distance = v)}
			/>
		</div>
		<h2 class="text-md my-3 font-semibold">Program 2</h2>
		<div class="pl-2">
			<SliderWithTextbox
				label="Speed"
				unit="steps/s"
				value={appState.zStage.program_2.speed}
				min={100}
				max={1400}
				onChange={(v: number) => (appState.zStage.program_2.speed = v)}
			/>
			<SliderWithTextbox
				label="Distance"
				unit="steps"
				value={appState.zStage.program_2.distance}
				min={1}
				max={500}
				onChange={(v: number) => (appState.zStage.program_2.distance = v)}
			/>
		</div>
	</div>
</div>
