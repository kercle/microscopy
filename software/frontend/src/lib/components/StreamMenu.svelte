<script lang="ts">
	import Download from '$lib/icons/Download.svelte';
	import GotoLowerLimit from '$lib/icons/GotoLowerLimit.svelte';
	import GotoUpperLimit from '$lib/icons/GotoUpperLimit.svelte';
	import Home from '$lib/icons/Home.svelte';
	import MoveDown from '$lib/icons/MoveDown.svelte';
	import MoveDownFast from '$lib/icons/MoveDownFast.svelte';
	import MoveUp from '$lib/icons/MoveUp.svelte';
	import MoveUpFast from '$lib/icons/MoveUpFast.svelte';
	import ReleaseLimits from '$lib/icons/ReleaseLimits.svelte';
	import SetLowerLimit from '$lib/icons/SetLowerLimit.svelte';
	import SetUpperLimit from '$lib/icons/SetUpperLimit.svelte';
	import Stop from '$lib/icons/Stop.svelte';
	import type { State, TravelSettings } from '$lib/state';

	let { appState = $bindable<State>() } = $props();

	const sendCommand = (command: string) => {
		console.log(`Test: ${appState.zStage.program_1.speed}`);
		fetch(command).catch((error) => {
			console.error('Error sending command:', error);
		});
	};

	const up = (program: TravelSettings) => {
		sendCommand(
			`/api/stage_z/steps?steps=${program.distance}&step_delay_us=${Math.floor(1_000_000 / program.speed)}`
		);
	};

	const down = (program: TravelSettings) => {
		sendCommand(
			`/api/stage_z/steps?steps=-${program.distance}&step_delay_us=${Math.floor(1_000_000 / program.speed)}`
		);
	};
</script>

<ul
	class="menu bg-base-200 hover:border-base-300 rounded-box absolute right-4 top-4 ml-auto border border-transparent opacity-50 hover:opacity-100"
>
	<li class="flex justify-center">
		<a href="/api/photo">
			<Download />
		</a>
	</li>

	<li class="mt-6 flex justify-center">
		<button onclick={() => sendCommand('/api/stage_z/goto_upper_limit')}>
			<GotoUpperLimit />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => up(appState.zStage.program_2)}>
			<MoveUpFast />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => up(appState.zStage.program_1)}>
			<MoveUp />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => sendCommand('/api/stage_z/stop')}>
			<Stop />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => down(appState.zStage.program_1)}>
			<MoveDown />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => down(appState.zStage.program_2)}>
			<MoveDownFast />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => sendCommand('/api/stage_z/goto_lower_limit')}>
			<GotoLowerLimit />
		</button>
	</li>

	<li class="mt-6 flex justify-center">
		<button onclick={() => sendCommand('/api/stage_z/set_upper_limit')}>
			<SetUpperLimit />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => sendCommand('/api/stage_z/home')}>
			<Home />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => sendCommand('/api/stage_z/release_limits')}>
			<ReleaseLimits />
		</button>
	</li>
	<li class="flex justify-center">
		<button onclick={() => sendCommand('/api/stage_z/set_lower_limit')}>
			<SetLowerLimit />
		</button>
	</li>
</ul>
