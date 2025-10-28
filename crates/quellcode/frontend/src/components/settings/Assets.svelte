<script lang="ts">
	import Button from "@components/input/Button.svelte";
	import TextInput from "@components/input/TextInput.svelte";
	import Join from "@components/Join.svelte";
	import type { SettingsPageState } from "@lib/state";

	let { app }: SettingsPageState = $props();
</script>

<div>
	<div class="sticky top-0 bg-base-100 p-4">
		<label>
			<span class="block">Asset URL</span>
			<Join>
				<TextInput placeholder="https://packagecontrol.io/packages/ayu"
				></TextInput>
				<Button>Add</Button>
			</Join>
		</label>
	</div>

	<div class="px-4">
		<div class="flex flex-col gap-4 pb-80">
			<details
				class="w-full bg-base-200/50 px-2 rounded-theme shadow-md inset-shadow-sm inset-shadow-white/5"
				open
			>
				<summary class="font-bold select-none py-2">Themes</summary>
				<div class="pb-2 flex flex-col h-full overflow-y-auto">
					{#each app.themes as theme}
						<div>
							{theme}
						</div>
					{/each}
				</div>
			</details>
			<details
				class="w-full bg-base-200/50 px-2 rounded-theme shadow-md inset-shadow-sm inset-shadow-white/5"
				open
			>
				<summary class="font-bold select-none py-2">Syntaxes</summary>
				<div class="pb-2 flex flex-col h-full overflow-y-auto">
					{#each app.syntaxes as syntax}
						<div>
							{syntax}
						</div>
					{/each}
				</div>
			</details>
		</div>
	</div>
</div>
