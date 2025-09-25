<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import Combobox from "@components/input/Combobox.svelte";
	import FontSelector from "@components/input/FontSelector.svelte";
</script>

<main class="h-svh">
	<FontSelector defaultFamily="JetBrains Mono" />
</main>
