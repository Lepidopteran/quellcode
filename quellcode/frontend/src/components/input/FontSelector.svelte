<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import Combobox from "@components/input/Combobox.svelte";
	import { onMount } from "svelte";
	import type { ClassValue } from "svelte/elements";

	interface Props {
		defaultFamily?: string;
		onChange?: (family: string) => void;
		class?: ClassValue;
	}

	let families: string[] = $state([]);
	let loading = $state(true);
	let comboBoxRef: ReturnType<typeof Combobox> | null = $state(null);

	onMount(() => {
		(async () => {
			families = await invoke<string[]>("font_families");
			if (!defaultFamily) {
				activeIndex = 0;
			} else {
				activeIndex = families.findIndex((f) => f === defaultFamily) ?? families[0];
			}

			loading = false;
		})();
	});

	export function setFamily(family: string) {
		if (families.findIndex((f) => f === family) === -1) {
			throw new Error("Font family not found", { cause: family });
		}

		activeIndex = families.findIndex((f) => f === family);
	}

	export function setFamilyByIndex(index: number) {
		if (index < 0 || index >= families.length) {
			throw new Error("Font family index out of range", { cause: index });
		}

		activeIndex = index;
	}

	export function getFamily() {
		return families[activeIndex];
	}

	let activeIndex = $state(0);
	let { defaultFamily, class: classValue, onChange }: Props = $props();
</script>

	<Combobox
		bind:this={comboBoxRef}
		searchFilter={(query, item) => item.includes(query)}
		getDisplayText={(item) => item}
		bind:activeIndex
		data={families}
		style={`font-family: ${families[activeIndex]}`}
		onActivate={(item) => onChange?.(item)}
		class={classValue}
		virtualize
	>
		{#snippet item(item, _)}
			<div class="flex items-center justify-between px-2">
				<div class="truncate" style:font-family={item}>{item}</div>
			</div>
		{/snippet}
	</Combobox>
