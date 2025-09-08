<script lang="ts">
    export let label = "Value";
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
		class="range range-xs [--range-fill:0]"
	/>
	<input
		type="number"
		{min}
		{max}
		{step}
		bind:value
		onfocusout={handleInput}
        onkeydown={(e) => e.key === 'Enter' && handleInput(e)}
		class="input input-ghost ml-4 py-0 text-right w-18"
	/>
</div>
</div>

