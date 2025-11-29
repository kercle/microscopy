<script lang="ts">
	import type { ComputeNode } from '$lib/bindings/ComputeNode';
	import type { TaskUiDescription } from '$lib/bindings/TaskUiDescription';
	import type { State } from '$lib/state';
	import Task from './Task.svelte';

	let { appState = $bindable<State>() } = $props();

	let compute_nodes: ComputeNode[] = $state([]);
	let task_components: Record<string, Task> = $state({});

	export const initAllComputeNodes = (new_compute_nodes: ComputeNode[]) => {
		compute_nodes = new_compute_nodes;
	};

	export const updateComputeNode = (node_id: string, task_ui: TaskUiDescription) => {
		const id = getTaskId(node_id, task_ui.name);
		task_components[id]?.updateUiFromBackend(task_ui);
	};

	const getTaskId = (compute_node_uuid: string, task_name: string) => {
		return `${compute_node_uuid}-${task_name}`;
	};

	const listTasks = () => {
		let task_list: {
			compute_node_uuid: string;
			task_id: string;
			task: TaskUiDescription;
		}[] = [];

		for (const node of compute_nodes) {
			for (const [task_id, task] of Object.entries(node.capabilities.tasks)) {
				if (task !== undefined) {
					task_list.push({
						task: task,
						compute_node_uuid: node.node_id,
						task_id: task_id
					});
				}
			}
		}

		return task_list;
	};
</script>

<div class="flex flex-col gap-2">
	{#each listTasks() as entry}
		{@const id = getTaskId(entry.compute_node_uuid, entry.task.name)}
		<Task
			compute_node_id={entry.compute_node_uuid}
			task={entry.task}
			{appState}
			bind:this={task_components[id]}
		/>
	{/each}
</div>
