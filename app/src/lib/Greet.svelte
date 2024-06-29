<script>
	import { invoke } from '@tauri-apps/api/core';
	import Button from '$lib/components/ui/button/button.svelte';

	let name = '';
	let greetMsg = '';

	async function greet() {
		greetMsg = await invoke('greet', { name });
	}
</script>

<div>
	<input id="greet-input" placeholder="Enter a name..." bind:value={name} />
	<Button on:click={greet}>Button</Button>
	<p>{greetMsg}</p>
</div>
