<script lang="ts">
	import { connect } from '$lib/syncSocket';

	import CameraControls from '$lib/components/CameraControls.svelte';
	import Drawer from '$lib/icons/Drawer.svelte';

	let ws: WebSocket | null = $state(null);

	let camera_controls_ref: CameraControls;

	$effect(() => {
		ws = connect((event: MessageEvent) => {
			camera_controls_ref.update(event);
		});
		return () => ws?.close();
	});
</script>

<div class="drawer drawer-end lg:drawer-open">
	<input id="my-drawer-2" type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col items-center justify-center">
		<div class="tabs tabs-border h-full w-full">
			<input type="radio" name="main-tabs" class="tab ml-2" aria-label="Live" checked />
			<div class="tab-content bg-base-100 border-t-base-300 p-6">
				<img src="/api/stream" alt="Microscope camera stream" class="w-full rounded-md" />
			</div>
			<input type="radio" name="main-tabs" class="tab" aria-label="Journal" />
			<div class="tab-content bg-base-100 border-base-300 p-6">Content 2</div>

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
