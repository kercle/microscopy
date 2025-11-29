<script lang="ts">
	import type { Widget } from '$lib/bindings/Widget';
	import type { TaskUiDescription } from '$lib/bindings/TaskUiDescription';
	import type { Value } from '$lib/bindings/Value';
	import type { WebSocketMessage } from '$lib/bindings/WebSocketMessage';
	import Live from '$lib/icons/Live.svelte';
	import type { State } from '$lib/state';
	import TaskWidget from './TaskWidget.svelte';

	let {
		compute_node_id,
		task,
		appState = $bindable<State>()
	}: { compute_node_id: string; task: TaskUiDescription; appState: State } = $props();

	let task_title: HTMLHeadingElement | null = $state(null);
	let task_progress: HTMLProgressElement | null = $state(null);
	let task_elements: Record<string, TaskWidget> = $state({});

	let ui_values: Record<string, Value> = {};

	export const updateUiFromBackend = (task_ui: TaskUiDescription) => {
		task_title!.textContent = task_ui.display_name;

		for (const [element_id, element_entry] of Object.entries(task_ui.elements)) {
			if (element_entry === undefined || task_elements[element_id] === undefined) {
				// TODO: If the element is not added yet, we should add it dynamically.
				// not relevant for now, since all task UIs are static.
				continue;
			}

			task_elements[element_id].update(element_entry);
		}
	};

	const listWidgetsForTask = () => {
		let input_list: { element_id: string; element: Widget }[] = [];

		for (const [element_id, element_entry] of Object.entries(task.elements)) {
			if (element_entry !== undefined) {
				input_list.push({ element_id, element: element_entry });
			}
		}

		return input_list;
	};

	const fetchUiFromParamChange = (key: string, value: Value) => {
		ui_values[key] = value;

		let msg: WebSocketMessage = {
			with_task_params: {
				task_name: task.name,
				source_uuid: null,
				destination_uuid: compute_node_id,
				params: ui_values
			}
		};

		appState.ws?.send(JSON.stringify(msg));
	};

	const startTask = () => {
		let msg: WebSocketMessage = {
			start_task: {
				task_name: task.name,
				compute_node_uuid: compute_node_id,
				params: ui_values
			}
		};

		appState.ws?.send(JSON.stringify(msg));
	};
</script>

<div class="card bg-base-200 w-full shadow-sm">
	<div class="card-body">
		<div class="card-header">
			<div class="flex flex-row items-center gap-2">
				<h2 class="card-title whitespace-nowrap text-lg font-bold" bind:this={task_title}>
					{task.display_name}
				</h2>
				<progress class="progress mx-5" value="40" max="100" bind:this={task_progress}></progress>
				<button class="btn btn-ghost text-primary" onclick={startTask}><Live /></button>
			</div>
		</div>
		<div class="divider"></div>
		<div class="grid grid-cols-{task.columns} gap-4">
			{#each listWidgetsForTask() as { element_id: elementId, element }}
				<TaskWidget
					{elementId}
					{element}
					{fetchUiFromParamChange}
					bind:this={task_elements[elementId]}
				/>
			{/each}
		</div>
	</div>
</div>
