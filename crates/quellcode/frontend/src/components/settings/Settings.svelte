<script lang="ts">
	import Button from "@components/input/Button.svelte";
	import type { Component } from "svelte";
	import General from "./General.svelte";
	import { slide } from "svelte/transition";
	import type { AppState } from "@lib/state";
	import About from "./About.svelte";

	interface Props {
		app: AppState;
	}

	interface Item {
		name: string;
		content: Component<{ app: AppState }>;
	}

	let activeTab = $state(0);

	const tabs: Item[] = [
		{ name: "General", content: General },
		{ name: "About", content: About },
	];

	const uuid = $props.id();

	let { app }: Props = $props();
</script>

<div class="grid grid-cols-[auto_1fr] h-full overflow-hidden">
	<div
		class="bg-base-200 focus-within:inset-shadow-primary inset-shadow-sm"
		role="tablist"
	>
		{#each tabs as tab, index}
			<Button
				id="{tab.name.toLowerCase()}-tab-{uuid}"
				role="tab"
				variant="ghost"
				aria-selected={activeTab === index}
				tabindex={activeTab === index ? undefined : -1}
				class="block rounded-none w-full text-left aria-selected:inset-shadow-primary/25 aria-selected:bg-primary-400/50 outline-none"
				onclick={() => {
					activeTab = index;
				}}
				onkeydown={(e) => {
					if (e.key === "ArrowDown") {
						activeTab = Math.min(activeTab + 1, tabs.length - 1);
					} else if (e.key === "ArrowUp") {
						activeTab = Math.max(activeTab - 1, 0);
					}
				}}
			>
				{tab.name}
			</Button>
		{/each}
	</div>
	<div class="bg-base-100 overflow-y-auto">
		{#each tabs as tab, index}
			<div
				id="{tab.name.toLowerCase()}-panel-{uuid}"
				role="tabpanel"
				transition:slide={{ axis: "y" }}
				aria-labelledby="{tab.name.toLowerCase()}-tab-{uuid}"
				hidden={activeTab !== index}
				tabindex={activeTab === index ? 0 : -1}
			>
				<tab.content {app} />
			</div>
		{/each}
	</div>
</div>
