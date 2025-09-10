<script lang="ts">
	import type { LogMessage } from '$lib';

	let log_messages: LogMessage[] = $state([]);

	export const addLogMessages = (new_logs: LogMessage[]) => {
		log_messages = [...log_messages, ...new_logs];

		if (log_messages.length > 200) {
			log_messages = log_messages.slice(-200);
		}
	};
</script>

<div>
	{#if log_messages.length === 0}
		<p class="text-center text-sm text-gray-500">No log messages yet.</p>
	{:else}
		<div class="flex flex-col gap-2 overflow-y-scroll" style="height: calc(100vh - 90px);">
			{#each log_messages as log}
				<div class="flex flex-row gap-2">
					<span class="w-35 font-mono text-xs text-gray-500">{log.timestamp}</span>
					<span
						class="w-12 text-right font-mono text-xs"
						class:text-red-500={log.level === 'ERROR'}
						class:text-yellow-500={log.level === 'WARN'}
						class:text-green-500={log.level === 'INFO'}
						class:text-blue-500={log.level === 'DEBUG'}>[{log.level}]</span
					>
					<span class="text-sm">{log.message}</span>
				</div>
			{/each}
		</div>
	{/if}
</div>
