<script lang="ts">
	import type { Element } from '$lib/bindings/Element';
	import type { ProcedureUi } from '$lib/bindings/ProcedureUi';
	import Live from '$lib/icons/Live.svelte';
	import type { State } from '$lib/state';
	import ProcedureElement from './ProcedureElement.svelte';

	let {
		compute_node_id,
		procedure,
		appState = $bindable<State>()
	}: { compute_node_id: string; procedure: ProcedureUi; appState: State } = $props();

	const listElementsForProcedure = () => {
		let input_list: { input_id: string; element: Element }[] = [];

		for (const [input_id, element_entry] of Object.entries(procedure.elements)) {
			if (element_entry !== undefined) {
				input_list.push({ input_id, element: element_entry });
			}
		}

		return input_list;
	};

	let input_elements: HTMLInputElement[] = [];

	const updateUiFromInputs = () => {
		// Placeholder for future functionality to update UI based on input changes
	};
</script>

<div class="card bg-base-200 w-full shadow-sm">
	<div class="card-body">
		<div class="card-header">
			<div class="flex flex-row items-center gap-2">
				<h2 class="card-title whitespace-nowrap text-lg font-bold">{procedure.display_name}</h2>
				<progress class="progress mx-5" value="40" max="100"></progress>
				<button class="btn btn-ghost text-primary"><Live /></button>
			</div>
		</div>
		<div class="divider"></div>
		<div class="grid grid-cols-{procedure.columns} gap-4">
			{#each listElementsForProcedure() as { input_id, element }}
				<ProcedureElement {element} />
			{/each}
		</div>
	</div>
</div>
