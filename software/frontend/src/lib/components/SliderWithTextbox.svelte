<script lang="ts">
	export let label = 'Value';
	export let value = 0;
	export let min = 0;
	export let max = 100;
	export let step = 1;
	export let onChange: (value: number) => void = () => {};

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		value = Number(target.value);
		onChange(value);
	}
</script>

<div class="my-1 flex flex-col">
	<span class="label-text">{label}:</span>
	<div class="flex items-center">
		<input
			type="range"
			{min}
			{max}
			{step}
			bind:value
			oninput={handleInput}
			class="mr-4 range range-xs [--range-fill:0]"
		/>
		<input
			type="number"
			{min}
			{max}
			{step}
			bind:value
			onfocus={(e) => (e.target as HTMLInputElement).select()}
			onfocusout={handleInput}
			onkeydown={(e) => e.key === 'Enter' && handleInput(e)}
			class="input input-ghost w-17 ml-4 h-6 min-h-0 py-0 text-right"
		/>
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
