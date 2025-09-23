<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import Listbox from "./components/Listbox.svelte";
</script>

<main class="h-svh">
	{#await invoke<string[]>("font_families")}
		Fetching Fonts
	{:then families}
		<Listbox data={families} getKey={(_, i) => i}>
			{#snippet item(item, _)}
					{item}
			{/snippet}
		</Listbox>
	{/await}
</main>
