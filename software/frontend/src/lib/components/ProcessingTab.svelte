<script lang="ts">
	import type { ComputeNode } from '$lib/bindings/ComputeNode';
	import type { ProcedureUi } from '$lib/bindings/ProcedureUi';
	import type { State } from '$lib/state';
	import Procedure from './Procedure.svelte';

	let { appState = $bindable<State>() } = $props();
	let compute_nodes: ComputeNode[] = $state([]);

	export const updateComputeNodes = (new_compute_nodes: ComputeNode[]) => {
		compute_nodes = new_compute_nodes;
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
		<Procedure
			compute_node_id={entry.compute_node_uuid}
			procedure={entry.procedure}
			{appState}
		/>
	{/each}
</div>
