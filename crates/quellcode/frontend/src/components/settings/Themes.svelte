<script lang="ts">
	import type { AppState } from "@lib/state";

	let { app }: { app: AppState } = $props();
</script>

<div class="space-y-2">
	<div class="bg-base-200 p-2 rounded-theme shadow-md inset-shadow-sm"></div>
	<div class="grid grid-cols-1 md:grid-cols-2 gap-2">
		{#each app.themes as theme}
			<div>
				{theme}
			</div>
		{/each}
	</div>
</div>
