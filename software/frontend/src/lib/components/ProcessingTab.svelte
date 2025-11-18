<script lang="ts">
	import type { ComputeNode } from '$lib/bindings/ComputeNode';
	import type { Input } from '$lib/bindings/Input';
	import type { Procedure } from '$lib/bindings/Procedure';
	import Live from '$lib/icons/Live.svelte';
	import ProcedureInputs from './ProcedureInputs.svelte';

	let compute_nodes: ComputeNode[] = $state([]);

	export const updateComputeNodes = (new_compute_nodes: ComputeNode[]) => {
		compute_nodes = new_compute_nodes;
	};

	const listProcedures = () => {
		let procedure_list: {
			compute_node_uuid: string;
			procedure_id: string;
			procedure: Procedure;
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

<div class="flex flex-row gap-2">
	{#each listProcedures() as entry}
		<div class="card card-border border-base-300 bg-base-100 flex-auto">
			<ProcedureInputs procedure={entry.procedure} />
		</div>

		<div class="flex flex-col justify-center">
			<hr class="border-base-300 w-10 border-2" />
		</div>

		<div class="card card-border border-base-300 bg-base-100">
			<div class="card-body flex flex-col gap-4">
				<div class="flex flex-row items-center justify-between gap-4 text-lg font-bold">
					{entry.procedure.display_name}
					<button class="btn btn-ghost text-primary"><Live /></button>
				</div>
				<progress class="progress" value="40" max="100"></progress>
			</div>
		</div>

		<div class="flex flex-col justify-center">
			<hr class="border-base-300 w-10 border-2" />
		</div>

		<div class="card card-border border-base-300 bg-base-100 flex-auto">
			<div class="card-body">test</div>
		</div>
	{/each}
</div>
