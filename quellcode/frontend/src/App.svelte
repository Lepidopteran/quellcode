<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import VirtualList from "svelte-tiny-virtual-list";
</script>

<main>
	{#await invoke<string[]>("font_families")}
		Fetching Fonts
	{:then families}
		<VirtualList
			itemCount={families.length}
			height={400}
			width="100%"
			itemSize={24}
		>
			{#snippet item({ style, index })}
				{@const family = families[index as number]}
				<span {style} style:text-align="left" style:font-family={family}>{family}</span>
			{/snippet}
		</VirtualList>
	{/await}
</main>
