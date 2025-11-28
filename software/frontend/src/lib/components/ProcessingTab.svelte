<script lang="ts">
	import type { ComputeNode } from '$lib/bindings/ComputeNode';
	import type { ProcedureUi } from '$lib/bindings/ProcedureUi';
	import type { State } from '$lib/state';
	import Procedure from './Procedure.svelte';

	let { appState = $bindable<State>() } = $props();
	let compute_nodes: ComputeNode[] = $state([]);

	export const initAllComputeNodes = (new_compute_nodes: ComputeNode[]) => {
		compute_nodes = new_compute_nodes;
	};

	export const updateComputeNode = (node_id: string, procedure_ui: ProcedureUi) => {
		const index = compute_nodes.findIndex((node) => node.node_id === node_id);

		if (index == -1) {
			return;
		}

		console.log('Updating procedure', procedure_ui.name, 'to', procedure_ui);

		if (procedure_ui.name in compute_nodes[index].capabilities.procedures) {
			compute_nodes[index].capabilities.procedures[procedure_ui.name] = procedure_ui;
		}
	};

	const listProcedures = () => {
		let procedure_list: {
			compute_node_uuid: string;
			procedure_id: string;
			procedure: ProcedureUi;
		}[] = [];

		for (const node of compute_nodes) {
			for (const [procedure_id, procedure] of Object.entries(node.capabilities.procedures)) {
				if (procedure !== undefined) {
					procedure_list.push({
						procedure,
						compute_node_uuid: node.node_id,
						procedure_id: procedure_id
					});
				}
			}
		}

		return procedure_list;
	};
</script>

<div class="flex flex-col gap-2">
	{#each listProcedures() as entry}
		<Procedure compute_node_id={entry.compute_node_uuid} procedure={entry.procedure} {appState} />
	{/each}
</div>
