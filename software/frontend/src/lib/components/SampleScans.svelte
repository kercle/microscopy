<script lang="ts">
	import Download from '$lib/icons/Download.svelte';
	import Trash from '$lib/icons/Trash.svelte';
	import { onMount } from 'svelte';

	type ScanEntry = {
		timestamp: string;
		frame_count: number;
		relative_start_pos: number;
		relative_stop_pos: number;
		steps_between_layers: number;
		uuid: string;
	};

	let frameSlider: HTMLInputElement;
    let previewImage: HTMLImageElement;

	let data: ScanEntry[] = [];
	let selectedIdx: number | null = null;

	const selectIdx = (idx: number) => {
		selectedIdx = idx;
		frameSlider.value = '0';
	};

    const updatePreviewImage = () => {
        if (selectedIdx === null) return;
        previewImage.src = `/api/z_scan_thumbnail/${data[selectedIdx].uuid}/${frameSlider.value}/800`;
    };

	onMount(async () => {
		try {
			const res = await fetch('/api/list_z_scans');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			data = await res.json();

			// data = [
			// 	{
			// 		timestamp: '2024-06-01T12:00:00Z',
			// 		frame_count: 150,
			// 		relative_start_pos: 0,
			// 		relative_stop_pos: 3000,
			// 		steps_between_layers: 20,
			// 		uuid: '123e4567-e89b-12d3-a456-426614174000'
			// 	},
			// 	{
			// 		timestamp: '2024-06-02T15:30:00Z',
			// 		frame_count: 200,
			// 		relative_start_pos: 100,
			// 		relative_stop_pos: 4100,
			// 		steps_between_layers: 20,
			// 		uuid: '123e4567-e89b-12d3-a456-426614174001'
			// 	}
			// ];

			selectedIdx = data.length > 0 ? 0 : null;
		} catch (err) {
			console.error('Failed to fetch data:', err);
		}
	});
</script>

<div
	class="flex flex-col-reverse gap-2 overflow-y-auto lg:h-[calc(100vh-90px)] lg:flex-row lg:overflow-y-hidden"
>
	<div
		class="divide-base-300 flex flex-col divide-y rounded-md lg:h-[calc(100vh-90px)] lg:overflow-y-auto"
	>
		{#each data as entry, idx}
			<button
				class="hover:bg-base-200 flex cursor-pointer flex-row items-start gap-8 p-2 text-left"
				onclick={() => selectIdx(idx)}
			>
				<div class="pointer-events-none">
					<img src="/api/z_scan_thumbnail/{entry.uuid}/0/100" alt="Thumbnail" class="rounded-md" />
				</div>
				<div class="flex-2 w-100 pointer-events">
					<div class="text-sm font-bold">
						{new Date(entry.timestamp).toLocaleString()}
					</div>
					<div class="text-sm opacity-50">
						{entry.frame_count} frames
					</div>
				</div>
			</button>
		{/each}
	</div>
	<div class="divider divider-horizontal lg:flex"></div>
	<div class="flex w-[100%] flex-col items-center gap-2 overflow-y-auto">
		{#if selectedIdx !== null}
			<div class="relative w-full">
				<img
                    bind:this={previewImage}
					src="/api/z_scan_thumbnail/{data[selectedIdx].uuid}/0/800"
					alt="Preview"
					class="w-full rounded-md"
					draggable="false"
				/>

				<div
					class="bg-base-100 rounded-box absolute left-0 top-0 ml-4 mt-4 p-4 opacity-70 hover:opacity-100"
				>
					<div class="mb-1 text-lg font-bold">Scan Details</div>
					<div class="text-sm">Start offset steps: {data[selectedIdx].relative_start_pos}</div>
					<div class="text-sm">Stop offset steps: {data[selectedIdx].relative_stop_pos}</div>
					<div class="text-sm">Steps between frames: {data[selectedIdx].steps_between_layers}</div>
					<div class="text-sm">Frames: {data[selectedIdx].frame_count}</div>
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
					bind:this={frameSlider}
					type="range"
					min="0"
					max={data[selectedIdx].frame_count - 1}
					value="0"
					class="range absolute bottom-0 left-0 mb-4 ml-[1rem] w-[calc(100%-2rem)] opacity-70 hover:opacity-100"
                    onchange={updatePreviewImage}
				/>
			</div>
		{/if}
	</div>
</div>
