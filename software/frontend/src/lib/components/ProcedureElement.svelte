<script lang="ts">
	import type { Element } from '$lib/bindings/Element';
	import type { ElementPositioning } from '$lib/bindings/ElementPositioning';

	let { element }: { element: Element } = $props();

	let type: string = $state('');
	let positioning: ElementPositioning = $state({
		row: 0,
		column: 0,
		row_span: 1,
		column_span: 1
	});

	if ('Select' in element) {
		positioning = element.Select.positioning;
	} else if ('Image' in element) {
		positioning = element.Image.positioning;
	} else {
		throw new Error('Unsupported element type');
	}
</script>

<!-- {#if 'Select' in element}
	{@const obj = element.Select}
	<div
		class="col-start-{positioning.column} col-span-{positioning.column_span} row-start-{positioning.row} row-span-{positioning.row_span}"
	>
		<p class="mb-2">{obj.display_name} ({obj.positioning.column})</p>
		<select class="select select-ghost w-full">
			{#each obj.options as option}
				<option>{option}</option>
			{/each}
		</select>
	</div>
{:else if 'Image' in element}
	{@const obj = element.Image}
	<div
		class="col-start-{positioning.column} col-span-{positioning.column_span} row-start-{positioning.row} row-span-{positioning.row_span}"
	>
		<p>{obj.display_name} ({obj.positioning.column})</p>
		<div class="flex w-full items-center justify-center">
			<img src={obj.href} alt={obj.display_name} class="max-h-full max-w-full" />
		</div>
	</div>
{/if} -->

{#if 'Select' in element}
    {@const obj = element.Select}
    <div
        class="mb-2"
        style="
            grid-column: {positioning.column} / span {positioning.column_span};
            grid-row: {positioning.row} / span {positioning.row_span};
        "
    >
        <p class="mb-2">{obj.display_name} ({obj.positioning.column})</p>
        <select class="select select-ghost w-full">
            {#each obj.options as option}
                <option>{option}</option>
            {/each}
        </select>
    </div>

{:else if 'Image' in element}
    {@const obj = element.Image}
    <div
        class="flex flex-col"
        style="
            grid-column: {positioning.column} / span {positioning.column_span};
            grid-row: {positioning.row} / span {positioning.row_span};
        "
    >
        <p>{obj.display_name} ({obj.positioning.column})</p>
        <div class="flex w-full items-center justify-center">
            <img src={obj.href} alt={obj.display_name} class="max-h-full max-w-full" />
        </div>
    </div>
{/if}
