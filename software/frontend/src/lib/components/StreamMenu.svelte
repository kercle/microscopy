<script lang="ts">
	import Download from '$lib/icons/Download.svelte';
	import GotoLowerLimit from '$lib/icons/GotoLowerLimit.svelte';
	import GotoUpperLimit from '$lib/icons/GotoUpperLimit.svelte';
	import MoveDown from '$lib/icons/MoveDown.svelte';
	import MoveUp from '$lib/icons/MoveUp.svelte';
	import ReleaseLimits from '$lib/icons/ReleaseLimits.svelte';
	import SetLowerLimit from '$lib/icons/SetLowerLimit.svelte';
	import SetUpperLimit from '$lib/icons/SetUpperLimit.svelte';
	import Stop from '$lib/icons/Stop.svelte';

	const sendCommand = (command: string) => {
		fetch(command).catch((error) => {
			console.error('Error sending command:', error);
		});
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
		<button on:click={() => sendCommand('/api/stage_z/goto_upper_limit')}>
			<GotoUpperLimit />
		</button>
	</li>
	<li class="flex justify-center">
		<button on:click={() => sendCommand('/api/stage_z/up')}>
			<MoveUp />
		</button>
	</li>
	<li class="flex justify-center">
		<button on:click={() => sendCommand('/api/stage_z/stop')}>
			<Stop />
		</button>
	</li>
	<li class="flex justify-center">
		<button on:click={() => sendCommand('/api/stage_z/down')}>
			<MoveDown />
		</button>
	</li>
	<li class="flex justify-center">
		<button on:click={() => sendCommand('/api/stage_z/goto_lower_limit')}>
			<GotoLowerLimit />
		</button>
	</li>

	<li class="mt-6 flex justify-center">
		<button on:click={() => sendCommand('/api/stage_z/set_upper_limit')}>
			<SetUpperLimit />
		</button>
	</li>
	<li class="flex justify-center">
		<button on:click={() => sendCommand('/api/stage_z/release_limits')}>
			<ReleaseLimits />
		</button>
	</li>
	<li class="flex justify-center">
		<button on:click={() => sendCommand('/api/stage_z/set_lower_limit')}>
			<SetLowerLimit />
		</button>
	</li>
</ul>
