<script lang="ts">
	import { connect } from '$lib/syncSocket';

	import CameraControls from '$lib/components/CameraControls.svelte';
	import Drawer from '$lib/icons/Drawer.svelte';
	import LogsJournal from '$lib/components/LogsJournal.svelte';
	import type { LogMessage } from '$lib';
	import Live from '$lib/icons/Live.svelte';
	import Journal from '$lib/icons/Journal.svelte';

	let ws: WebSocket | null = $state(null);
	let log_messages: LogMessage[] = $state([]);

	let camera_controls_ref: CameraControls;
	let logs_journal_ref: LogsJournal;

	$effect(() => {
		ws = connect((event: MessageEvent) => {
			let data = JSON.parse(event.data);

			camera_controls_ref.update(data);
			if (data.logs) {
				console.log('Received logs', data.logs);
				logs_journal_ref.addLogMessages(data.logs);
			}
		});
		return () => ws?.close();
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
			<div class="tab-content bg-base-100 border-t-base-300 p-6">
				<img src="/api/stream" alt="Microscope camera stream" class="w-full rounded-md" />
			</div>

			<!-- Tab 2: Journal -->
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
			<div class="card bg-base-100 shadow-sm">
				<CameraControls {ws} bind:this={camera_controls_ref} />
			</div>
		</ul>
	</div>
</div>
