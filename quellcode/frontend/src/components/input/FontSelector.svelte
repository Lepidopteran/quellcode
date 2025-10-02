<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import Combobox from "@components/input/Combobox.svelte";
	import { onMount } from "svelte";
	import type { ClassValue } from "svelte/elements";
	import type { FontFamily } from "@lib/bindings/FontFamily";

	const defaultName = "Monospace";

	interface Props {
		defaultFamily?: string;
		onChange?: (family: FontFamily) => void;
		class?: ClassValue;
	}

	let families: FontFamily[] = $state([]);
	let loading = $state(true);
	let comboBoxRef: ReturnType<typeof Combobox> | null = $state(null);

	onMount(() => {
		(async () => {
			families = await invoke<FontFamily[]>("font_families");
			families.push({
				name: defaultName,
				monospace: true,
			});

			families.sort((a, b) => {
				if (a.name === defaultName) return -1;
				if (b.name === defaultName) return 1;
				return a.name.localeCompare(b.name);
			});

			if (!defaultFamily) {
				activeIndex = 0;
			} else {
				activeIndex =
					defaultFamily === defaultName
						? families.findIndex((family) => family.name === defaultFamily)
						: 0;
			}

			loading = false;
		})();
	});

	export function setFamily(family: string) {
		const index = families.findIndex((f) => f.name === family);
		if (index === -1) {
			throw new Error("Font family not found", { cause: family });
		}

		activeIndex = index;
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
	searchFilter={(query, item) => item.name.toLowerCase().includes(query.toLowerCase())}
	getDisplayText={(item) => item.name}
	bind:activeIndex
	data={families}
	style={`font-family: "${families[activeIndex]?.name === defaultName ? "monospace" : families[activeIndex]?.name}"`}
	onActivate={(item) =>
		onChange?.({
			name: `"${item.name}"`,
			monospace: item.monospace,
		})}
	class={classValue}
	virtualize
>
	{#snippet item(item, _)}
		<div class="flex items-center justify-between px-2">
			<div
				class="truncate h-6 align-middle"
				style:font-family={`"${item.name === defaultName ? "monospace" : item.name}", sans-serif`}
			>
				{item.name}
			</div>
		</div>
	{/snippet}
</Combobox>
