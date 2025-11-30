<script lang="ts">
	import type { ZScanMetadata } from '$lib/bindings/ZScanMetadata';
	import Download from '$lib/icons/Download.svelte';
	import Trash from '$lib/icons/Trash.svelte';
	import { onMount } from 'svelte';

	const Z_SCAN_API_BASE = '/api/z-scan';

	let startOffsetInput: HTMLInputElement;
	let stopOffsetInput: HTMLInputElement;
	let stepsBetweenInput: HTMLInputElement;

	let frameSlider: HTMLInputElement | undefined = $state();
	let previewImage: HTMLImageElement | undefined = $state();

	let data: ZScanMetadata[] = $state([]);
	let selectedIdx: number | null = $state(null);

	const selectIdx = (idx: number) => {
		selectedIdx = idx;

		if (frameSlider) {
			frameSlider.value = '0';
		}
	};

	const deleteSelectedScan = async () => {
		if (selectedIdx === null) return;

		const uuid = data[selectedIdx].uuid;

		const res = await fetch(`${Z_SCAN_API_BASE}/delete/${uuid}`, {
			method: 'DELETE'
		});
		if (!res.ok) {
			console.error(`Failed to delete z-scan ${uuid}: HTTP ${res.status}`);
			return;
		}

		data.splice(selectedIdx, 1);
		selectedIdx = data.length > 0 ? 0 : null;
	};

	const updatePreviewImage = () => {
		if (
			selectedIdx === null ||
			frameSlider === undefined ||
			previewImage === undefined ||
			data[selectedIdx] === undefined
		) {
			return;
		}
		previewImage.src = `${Z_SCAN_API_BASE}/thumbnail/${data[selectedIdx].uuid}/${frameSlider.value}/800`;
	};

	const updateZScanData = async () => {
		const res = await fetch(`${Z_SCAN_API_BASE}/list`);
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
	};

	const startZScan = () => {
		const startOffset = parseInt(startOffsetInput.value, 10);
		const stopOffset = parseInt(stopOffsetInput.value, 10);
		const stepsBetween = parseInt(stepsBetweenInput.value, 10);

		fetch(`${Z_SCAN_API_BASE}/record/${startOffset}/${stopOffset}/${stepsBetween}`)
			.then((res) => {
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
				return res.json();
			})
			.then((json) => {
				data.push(json);
			})
			.catch((err) => {
				console.error('z-scan failed: ', err);
			});
	};

	onMount(async () => {
		try {
			await updateZScanData();
		} catch (err) {
			console.error('Failed to fetch data:', err);
		}
	});
</script>

<div
	class="flex flex-col-reverse gap-2 overflow-y-auto lg:h-[calc(100vh-90px)] lg:flex-row lg:overflow-y-hidden"
>
	<div class="mb-8 flex min-w-60 flex-col rounded-md lg:h-[calc(100vh-90px)] lg:overflow-y-auto">
		<div class="mb-4 flex flex-col gap-2 text-sm">
			<div class="mt-2 flex flex-row items-center">
				<div class="mr-auto">Start offset:</div>
				<input
					bind:this={startOffsetInput}
					type="number"
					class="input input-ghost w-17 h-6 min-h-0 py-0 text-right"
					value="200"
				/>
				<span class="badge badge-sm text-accent text-xs">steps</span>
			</div>
			<div class="flex flex-row">
				<div class="mr-auto">Stop offset:</div>
				<input
					bind:this={stopOffsetInput}
					type="number"
					class="input input-ghost w-17 h-6 min-h-0 py-0 text-right"
					value="-200"
				/>
				<span class="badge badge-sm text-accent text-xs">steps</span>
			</div>
			<div class="flex flex-row">
				<div class="mr-auto">Steps between:</div>
				<input
					bind:this={stepsBetweenInput}
					type="number"
					class="input input-ghost w-17 h-6 min-h-0 py-0 text-right"
					value="10"
					min="1"
				/>
				<span class="badge badge-sm text-accent text-xs">steps</span>
			</div>
			<button class="btn btn-primary mx-2 mt-2" onclick={startZScan}>Start Z-Scan</button>
		</div>
		{#each data as entry, idx}
			<button
				class="hover:bg-base-200 flex cursor-pointer flex-row items-start gap-8 p-2 text-left"
				onclick={() => selectIdx(idx)}
			>
				<div class="pointer-events-none">
					<img
						src="{Z_SCAN_API_BASE}/thumbnail/{entry.uuid}/{Math.floor(entry.frame_count / 2)}/80"
						alt="Thumbnail"
						class="rounded-md"
					/>
				</div>
				<div class="flex-2 w-90 pointer-events">
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
					src="{Z_SCAN_API_BASE}/thumbnail/{data[selectedIdx].uuid}/0/800"
					alt="{Z_SCAN_API_BASE}/thumbnail/{data[selectedIdx].uuid}/0/800"
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
						<button class="btn btn-ghost btn-secondary justify-start" onclick={deleteSelectedScan}>
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

<style>
	/* Chrome, Safari, Edge */
	input[type='number']::-webkit-inner-spin-button,
	input[type='number']::-webkit-outer-spin-button {
		-webkit-appearance: none;
		appearance: none;
		margin: 0;
	}

	/* Firefox */
	input[type='number'] {
		-moz-appearance: textfield;
		appearance: textfield;
		margin: 0;
	}
</style>
