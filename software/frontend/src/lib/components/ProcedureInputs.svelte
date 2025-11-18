<script lang="ts">
	import type { Input } from '$lib/bindings/Input';
	import type { Procedure } from '$lib/bindings/Procedure';
	import type { State } from '$lib/state';

	let {
		compute_node_id,
		procedure,
		appState = $bindable<State>()
	}: { compute_node_id: string; procedure: Procedure; appState: State } = $props();

	const listInputsForProcedure = () => {
		let input_list: { input_id: string; input_entry: Input }[] = [];

		for (const [input_id, input_entry] of Object.entries(procedure.inputs)) {
			if (input_entry !== undefined) {
				input_list.push({ input_id, input_entry });
			}
		}

		return input_list;
	};

	let input_elements: HTMLInputElement[] = [];

	const updateUiFromInputs = () => {
		// Placeholder for future functionality to update UI based on input changes
	};
</script>

<div class="card-body flex flex-col gap-4">
	{#each listInputsForProcedure() as { input_id, input_entry }}
		{#if 'Selection' in input_entry}
			<p class="mb-[-0.5em]">{input_entry.Selection.display_name}</p>
			<select class="select select-ghost w-full">
				{#each input_entry.Selection.options as option}
					<option>{option}</option>
				{/each}
			</select>
		{:else if 'ImagePreview' in input_entry}
			<p class="mb-2">{input_entry.ImagePreview.display_name}</p>
			<div class="bg-base-200 flex h-48 w-full items-center justify-center">
				<img
					src={input_entry.ImagePreview.href}
					alt={input_entry.ImagePreview.display_name}
					class="max-h-full max-w-full"
				/>
			</div>
		{/if}
	{/each}
</div>
