<script lang="ts">
	import type { ComputeNode } from '$lib/bindings/ComputeNode';
	import type { Input } from '$lib/bindings/Input';
	import type { Procedure } from '$lib/bindings/Procedure';
	import Live from '$lib/icons/Live.svelte';

	let compute_nodes: ComputeNode[] = $state([]);

	export const updateComputeNodes = (new_compute_nodes: ComputeNode[]) => {
		compute_nodes = new_compute_nodes;
	};

	const listProcedures = () => {
		let procedure_list: { procedure: Procedure }[] = [];

		for (const node of compute_nodes) {
			for (const [key, procedure] of Object.entries(node.capabilities.procedures)) {
				if (procedure !== undefined) {
					procedure_list.push({ procedure });
				}
			}
		}

		return procedure_list;
	};

	const listInputsForProcedure = (procedure: Procedure) => {
		let input_list: { input_id: string; input_entry: Input }[] = [];

		for (const [input_id, input_entry] of Object.entries(procedure.inputs)) {
			if (input_entry !== undefined) {
				input_list.push({ input_id, input_entry });
			}
		}

		return input_list;
	};
</script>

<div class="flex flex-row gap-2">
	{#each listProcedures() as entry}
		<div class="card card-border border-base-300 bg-base-100 flex-auto">
            <div class="card-body flex flex-col gap-4">
			{#each listInputsForProcedure(entry.procedure) as { input_id, input_entry }}
				{#if 'Selection' in input_entry}
					<p>{input_entry.Selection.display_name}</p>
                    <select class="select select-bordered w-full">
                        {#each input_entry.Selection.options as option}
                            <option>{option}</option>
                        {/each}
                    </select>
				{/if}
			{/each}
            </div>
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
