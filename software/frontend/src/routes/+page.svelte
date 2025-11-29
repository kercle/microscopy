<script lang="ts">
	import { connect } from '$lib/syncSocket';

	import CameraControls from '$lib/components/CameraControls.svelte';
	import Drawer from '$lib/icons/Drawer.svelte';
	import LogsJournal from '$lib/components/LogsJournal.svelte';
	import Live from '$lib/icons/Live.svelte';
	import Layers from '$lib/icons/Layers.svelte';
	import Journal from '$lib/icons/Journal.svelte';
	import StreamMenu from '$lib/components/StreamMenu.svelte';
	import ZStageControls from '$lib/components/ZStageControls.svelte';
	import type { State } from '$lib/state';
	import SampleScans from '$lib/components/SampleScans.svelte';
	import Processing from '$lib/icons/Processing.svelte';
	import type { WebSocketMessage } from '$lib/bindings/WebSocketMessage';
	import TasksTab from '$lib/components/TasksTab.svelte';
	import { onMount } from 'svelte';

	let appState: State = $state({
		ws: null,
		zStage: { program_1: { speed: 500, distance: 50 }, program_2: { speed: 1250, distance: 1000 } }
	});

	let camera_controls_ref: CameraControls;
	let logs_journal_ref: LogsJournal;
	let processing_tab_ref: TasksTab;

	$effect(() => {
		appState.ws = connect((event: MessageEvent) => {
			let msg: WebSocketMessage = JSON.parse(event.data);

			if (msg === 'register_user_client') {
				return;
			}

			if ('logs' in msg) {
				logs_journal_ref.addLogMessages(msg.logs);
			} else if ('update_parameters' in msg) {
				camera_controls_ref.update(msg.update_parameters);
			} else if ('compute_nodes' in msg) {
				console.log('Received compute nodes update:', msg.compute_nodes);
				processing_tab_ref.initAllComputeNodes(msg.compute_nodes);
			} else if ('task_description' in msg) {
				const node_id = msg.task_description.source_uuid;

				if (node_id) {
					processing_tab_ref.updateComputeNode(node_id, msg.task_description.ui_description);
				}
			} else if ('task_progress_update' in msg) {
				const node_id = msg.task_progress_update.compute_node_uuid!;
				const task_name = msg.task_progress_update.task_name;
				const progress = msg.task_progress_update.progress;

				processing_tab_ref.updateTaskProgress(node_id, task_name, progress);
			} else {
				console.warn('Unhandled WebSocket message:', msg);
			}
		});
		return () => appState.ws?.close();
	});

	const updateAnchor = (anchor: string) => {
		history.replaceState(null, '', `#${anchor}`);
	};

	onMount(() => {
		const anchor = window.location.hash.substring(1);
		let input_element = document.getElementById(`tab-select-${anchor}`) as HTMLInputElement;

		if (input_element) {
			input_element.checked = true;
		}
	});
</script>

<div class="drawer drawer-end lg:drawer-open">
	<input id="my-drawer-2" type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col items-center justify-center">
		<div class="tabs tabs-border h-full w-full">
			<!-- Tab 1: Live -->
			<label class="tab">
				<input
					id="tab-select-live"
					type="radio"
					name="main-tabs"
					onchange={() => updateAnchor('live')}
					checked
				/>
				<Live />
				Live
			</label>
			<div class="tab-content bg-base-100 border-t-base-300 relative p-2">
				<img
					src="/api/stream"
					alt="Microscope camera stream"
					class="w-full rounded-md"
					draggable="false"
				/>
				<StreamMenu bind:appState />
			</div>

			<!-- Tab 2: Sample scans -->
			<label class="tab">
				<input
					id="tab-select-scans"
					type="radio"
					name="main-tabs"
					onchange={() => updateAnchor('scans')}
				/>
				<Layers />
				Scans
			</label>
			<div class="tab-content bg-base-100 p-6">
				<SampleScans />
			</div>

			<!-- Tab 3: Processing -->
			<label class="tab">
				<input
					id="tab-select-processing"
					type="radio"
					name="main-tabs"
					onchange={() => updateAnchor('processing')}
				/>
				<Processing />
				<span class="ml-2">Tasks</span>
			</label>
			<div class="tab-content bg-base-100">
				<div class="h-[calc(100%-6.2rem)] overflow-y-auto p-6">
					<TasksTab bind:this={processing_tab_ref} {appState} />
				</div>
			</div>

			<!-- Tab 4: Journal -->
			<label class="tab">
				<input
					id="tab-select-journal"
					type="radio"
					name="main-tabs"
					onchange={() => updateAnchor('journal')}
				/>
				<Journal />
				Journal
			</label>
			<div class="tab-content bg-base-100 p-6">
				<LogsJournal bind:this={logs_journal_ref} />
			</div>

			<!-- Drawer toggle button -->
			<div class="flex-1"></div>
			<label for="my-drawer-2" class="btn btn-ghost drawer-button lg:hidden">
				<Drawer />
			</label>
		</div>
	</div>
	<div class="drawer-side border-l-base-300 border-l">
		<label for="my-drawer-2" aria-label="close sidebar" class="drawer-overlay"></label>
		<ul class="menu bg-base-200 text-base-content w-100 min-h-full p-4">
			<div class="flex flex-col gap-2">
				<CameraControls bind:appState bind:this={camera_controls_ref} />
				<ZStageControls bind:appState />
			</div>
		</ul>
	</div>
</div>
