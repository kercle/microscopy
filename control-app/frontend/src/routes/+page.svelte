<script lang="ts">
	import { connect } from '$lib/syncSocket';

	import CameraControls from '$lib/components/CameraControls.svelte';

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
		<div class="navbar bg-base-100 shadow-sm lg:hidden">
			<div class="flex-1"></div>
			<div class="flex-none justify-end">
				<label for="my-drawer-2" class="btn btn-ghost drawer-button">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
						class="inline-block h-5 w-5 stroke-current"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 6h16M4 12h16M4 18h16"
						></path>
					</svg>
				</label>
			</div>
		</div>
		<img src="/api/stream" alt="Microscope camera stream" />
	</div>
	<div class="drawer-side">
		<label for="my-drawer-2" aria-label="close sidebar" class="drawer-overlay"></label>
		<ul class="menu bg-base-200 text-base-content w-100 min-h-full p-4">
			<div class="card bg-base-100 shadow-sm">
				<CameraControls {ws} bind:this={camera_controls_ref} />
			</div>
		</ul>
	</div>
</div>
