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

	let appState: State = $state({
		ws: null,
		zStage: { program_1: { speed: 500, distance: 50 }, program_2: { speed: 1250, distance: 1000 } }
	});

	let camera_controls_ref: CameraControls;
	let logs_journal_ref: LogsJournal;

	$effect(() => {
		appState.ws = connect((event: MessageEvent) => {
			let msg: WebSocketMessage = JSON.parse(event.data);

			if ("logs" in msg) {
				logs_journal_ref.addLogMessages(msg.logs);
			} else if ("update_parameters" in msg) {
				camera_controls_ref.update(msg.update_parameters);
			}
		});
		return () => appState.ws?.close();
	});
</script>

<div class="drawer drawer-end lg:drawer-open">
	<input id="my-drawer-2" type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col items-center justify-center">
		<div class="tabs tabs-border h-full w-full">
			<!-- Tab 1: Live -->
			<label class="tab">
				<input type="radio" name="main-tabs" checked />
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
				<input type="radio" name="main-tabs" />
				<Layers />
				Scans
			</label>
			<div class="tab-content bg-base-100 p-6">
				<SampleScans />
			</div>

			<!-- Tab 3: Processing -->
			<label class="tab">
				<input type="radio" name="main-tabs" />
				<Processing />
				<span class="ml-2">Processing</span>
			</label>
			<div class="tab-content bg-base-100 p-6">
				<p>TODO</p>
			</div>


			<!-- Tab 4: Journal -->
			<label class="tab">
				<input type="radio" name="main-tabs" />
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
