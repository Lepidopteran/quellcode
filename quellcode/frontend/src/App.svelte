<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import FontSelector from "@components/input/FontSelector.svelte";

	let fontFamily: string | null = $state(null);
</script>

<div class="h-svh grid grid-cols-[1fr_300px]">
	<main class="p-2 h-full bg-base-200">
		<div class="h-full flex flex-col items-stretch justify-items-stretch gap-2">
			<div
				contenteditable="true"
				class="font-mono border border-black/50 shadow-md h-full rounded-theme"
				style:font-family={fontFamily}
			></div>

			<div
				class="font-mono border border-black/50 shadow-md h-full rounded-theme"
				style:font-family={fontFamily}
			></div>
		</div>
	</main>
	<aside class="p-2 overflow-y-auto h-full bg-base-100">
		<details class="w-full bg-base-200/50 px-2 rounded-theme">
			<summary class="font-bold select-none py-2">Code Settings</summary>
			<div>
				<FontSelector
					class="w-full"
					onChange={(f) => (fontFamily = f.name || null)}
					defaultFamily="JetBrains Mono"
				/>
			</div>
		</details>
	</aside>
</div>
