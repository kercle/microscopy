<script lang="ts">
	import type { Widget } from '$lib/bindings/Widget';
	import type { WidgetPosition } from '$lib/bindings/WidgetPosition';
	import type { Value } from '$lib/bindings/Value';

	let {
		elementId,
		element,
		fetchUiFromParamChange
	}: {
		elementId: string;
		element: Widget;
		fetchUiFromParamChange: (key: string, value: Value) => void;
	} = $props();

	let html_element: HTMLElement | null = $state(null);

	export const update = (new_element: Widget) => {
		if (element.type !== new_element.type) {
			console.warn(
				`Element type changed from ${element.type} to ${new_element.type}, which is not supported.`
			);
			return;
		}

		if (new_element.type === 'select') {
			const select_element = html_element as HTMLSelectElement;

			if (select_element.value !== new_element.value) {
				select_element.value = new_element.value;
			}
		} else if (new_element.type === 'slider') {
			const slider_element = html_element as HTMLInputElement;

			if (parseFloat(slider_element.value) !== new_element.value) {
				slider_element.value = new_element.value.toString();
			}
		} else if (new_element.type === 'image') {
			const img_element = html_element as HTMLImageElement;

			if (img_element.src !== new_element.href) {
				img_element.src = new_element.href;
			}
		}
	};

	const makeWrapperStype = (pos: WidgetPosition) => {
		return (
			`grid-column: ${pos.column} / span ${pos.column_span};` +
			`grid-row: ${pos.row} / span ${pos.row_span};`
		);
	};
</script>

<div class="mb-2" style={makeWrapperStype(element.positioning)}>
	{#if element.type === 'select'}
		<p class="mb-2">{element.display_name}</p>
		<select
			class="select select-ghost w-full"
			onchange={(e) => fetchUiFromParamChange(elementId, (e.target as HTMLSelectElement).value)}
			bind:this={html_element}
		>
			{#each element.options as option}
				<option>{option}</option>
			{/each}
		</select>
	{:else if element.type === 'image'}
		<p>{element.display_name}</p>
		<div class="flex w-full items-center justify-center">
			<img
				src={element.href}
				alt={element.display_name}
				class="max-h-full max-w-full"
				bind:this={html_element}
			/>
		</div>
	{:else if element.type === 'slider'}
		<p class="mb-2">{element.display_name}</p>
		<input
			type="range"
			min={element.min}
			max={element.max}
			step={element.step}
			value={element.value}
			class="range range-ghost w-full"
			onmouseup={(e) =>
				fetchUiFromParamChange(elementId, parseFloat((e.target as HTMLInputElement).value))}
			ontouchend={(e) =>
				fetchUiFromParamChange(elementId, parseFloat((e.target as HTMLInputElement).value))}
			bind:this={html_element}
		/>
	{:else}
		<p>Unsupported element type</p>
	{/if}
</div>
