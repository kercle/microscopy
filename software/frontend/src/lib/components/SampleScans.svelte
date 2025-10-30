<script lang="ts">
	import Download from "$lib/icons/Download.svelte";
	import Trash from "$lib/icons/Trash.svelte";

	let scan_list = [
		{
			relative_start_pos: 0,
			relative_stop_pos: 500,
			steps_between_layers: 50,
			frame_count: 11,
			uuid: '52017258-2d78-4c13-b341-33ebd49d3a8f',
			timestamp: '2025-10-29T23:05:42.868471485+01:00'
		},
		{
			relative_start_pos: 0,
			relative_stop_pos: 1000,
			steps_between_layers: 100,
			frame_count: 1,
			uuid: '2a9b0e04-9e2b-4a16-a75d-b183d0e807f2',
			timestamp: '2025-10-29T23:04:09.629753052+01:00'
		},
		{
			relative_start_pos: 0,
			relative_stop_pos: 500,
			steps_between_layers: 50,
			frame_count: 11,
			uuid: '60cef4a6-b4bb-4643-8a0d-69a7015cc18c',
			timestamp: '2025-10-29T23:06:08.537632636+01:00'
		},
		{
			relative_start_pos: -100,
			relative_stop_pos: 100,
			steps_between_layers: 10,
			frame_count: 21,
			uuid: 'e4a5f501-0865-4b0b-9840-98744fce5d4e',
			timestamp: '2025-10-30T00:58:56.071188199+01:00'
		},
		{
			relative_start_pos: -10,
			relative_stop_pos: 10,
			steps_between_layers: 2,
			frame_count: 11,
			uuid: '67cae7a7-704c-4beb-a403-4ecb97545956',
			timestamp: '2025-10-30T00:58:12.081582815+01:00'
		},
		{
			relative_start_pos: -500,
			relative_stop_pos: 500,
			steps_between_layers: 100,
			frame_count: 11,
			uuid: '1d1e8a40-85cd-4a3a-be99-73dcd49e76f6',
			timestamp: '2025-10-29T23:16:29.286858242+01:00'
		},
		{
			relative_start_pos: 500,
			relative_stop_pos: 0,
			steps_between_layers: 50,
			frame_count: 11,
			uuid: '42367e0e-fe5a-4363-808b-d4ed1bcbb140',
			timestamp: '2025-10-29T23:06:30.042023606+01:00'
		},
		{
			relative_start_pos: 0,
			relative_stop_pos: 1000,
			steps_between_layers: 100,
			frame_count: 1,
			uuid: '8f1d6485-e520-4494-9130-03ca012b475a',
			timestamp: '2025-10-29T23:03:58.789666987+01:00'
		}
	];
</script>

<div
	class="flex flex-col-reverse gap-2 overflow-y-auto lg:h-[calc(100vh-90px)] lg:flex-row lg:overflow-y-hidden"
>
	<div
		class="divide-base-300 flex flex-col divide-y rounded-md lg:h-[calc(100vh-90px)] lg:overflow-y-auto"
	>
		{#each scan_list as scan}
			<div class="flex flex-row items-start gap-8 p-2">
				<div>
					<img
						src="https://images.unsplash.com/photo-1733665825622-5f84dfd0717b?ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D&auto=format&fit=crop&q=80&w=70"
						alt="Sample Scan 1"
						class="rounded-md"
					/>
				</div>
				<div class="flex-2 w-100">
					<div class="text-sm font-bold">
						{new Date(scan.timestamp).toLocaleString()}
					</div>
					<div class="text-sm opacity-50">
						{scan.frame_count} frames
					</div>
				</div>
			</div>
		{/each}
	</div>
	<div class="divider divider-horizontal lg:flex"></div>
	<div class="flex w-[100%] flex-col items-center gap-2 overflow-y-auto">
		<div class="relative w-full">
			<img
				src="https://images.unsplash.com/photo-1733665825622-5f84dfd0717b?ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D&auto=format&fit=crop&q=80&w=800"
				alt="Sample Scan 1"
				class="w-full rounded-md"
			/>

            <div class="bg-base-100 p-4 rounded-box absolute left-0 top-0 ml-4 mt-4 opacity-70 hover:opacity-100">
                <div class="text-lg font-bold mb-1">Scan Details</div>
                <div class="text-sm">Start offset steps: 0</div>
                <div class="text-sm">Stop offset steps: 500</div>
                <div class="text-sm">Steps between frames: 50</div>
                <div class="text-sm">Frames: 11</div>
            </div>

			<ul
				class="menu bg-base-100 rounded-box absolute right-0 top-0 mr-4 mt-4 w-56 opacity-70 hover:opacity-100"
			>
				<li>
					<button class="btn btn-ghost justify-start">
						<Download />
						Download
					</button>
				</li>
				<li>
					<button class="btn btn-ghost btn-secondary justify-start">
						<Trash />
						Delete
					</button>
				</li>
			</ul>

			<input
				type="range"
				min="0"
				max="100"
				value="40"
				class="range absolute bottom-0 left-0 mb-4 ml-[1rem] w-[calc(100%-2rem)] opacity-70 hover:opacity-100"
			/>
		</div>
	</div>
</div>
