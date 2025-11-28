<script lang="ts">
	import type { Element } from '$lib/bindings/Element';
	import type { ElementPositioning } from '$lib/bindings/ElementPositioning';
	import type { Value } from '$lib/bindings/Value';

	let {
		elementId,
		element,
		updateUi
	}: { elementId: string; element: Element; updateUi: (key: string, value: Value) => void } =
		$props();

	let pos: ElementPositioning = $state({
		row: 0,
		column: 0,
		row_span: 1,
		column_span: 1
	});

	if ('Select' in element) {
		pos = element.Select.positioning;
	} else if ('Image' in element) {
		pos = element.Image.positioning;
	} else if ('Slider' in element) {
		pos = element.Slider.positioning;
	}

	const makeWrapperStype = (pos: ElementPositioning) => {
		return (
			`grid-column: ${pos.column} / span ${pos.column_span};` +
			`grid-row: ${pos.row} / span ${pos.row_span};`
		);
	};
</script>

<div class="mb-2" style={makeWrapperStype(pos)}>
	{#if 'Select' in element}
		{@const obj = element.Select}
		<p class="mb-2">{obj.display_name} ({obj.positioning.column})</p>
		<select
			class="select select-ghost w-full"
			onchange={(e) => updateUi(elementId, (e.target as HTMLSelectElement).value)}
		>
			{#each obj.options as option}
				<option>{option}</option>
			{/each}
		</select>
	{:else if 'Image' in element}
		{@const obj = element.Image}
		<p>{obj.display_name} ({obj.positioning.column})</p>
		<div class="flex w-full items-center justify-center">
			<img src={obj.href} alt={obj.display_name} class="max-h-full max-w-full" />
		</div>
	{:else if 'Slider' in element}
		{@const obj = element.Slider}
		<p class="mb-2">{obj.display_name} ({obj.positioning.column})</p>
		<input
			type="range"
			min={obj.min}
			max={obj.max}
			step={obj.step}
			value={obj.value}
			class="range range-ghost w-full"
			oninput={(e) => updateUi(elementId, parseFloat((e.target as HTMLInputElement).value))}
		/>
	{:else}
		<p>Unsupported element type</p>
	{/if}
</div>
