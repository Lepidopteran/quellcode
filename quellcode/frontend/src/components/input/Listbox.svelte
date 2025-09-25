<script lang="ts" generics="T">
	import type { ClassValue, HTMLAttributes } from "svelte/elements";
	import { type Snippet } from "svelte";
	import { match } from "ts-pattern";
	import { prefersReducedMotion } from "svelte/motion";
	import { VList } from "virtua/svelte";

	type FilteredItem = {
		data: T;
		originalIndex: number;
	};

	interface Props extends HTMLAttributes<HTMLDivElement> {
		data: T[];
		activeIndex?: number;
		class?: ClassValue;
		height?: number | string;
		width?: number | string;
		item: Snippet<[item: T, index: number]>;
		onSelect?: (
			item: T,
			index: number,
			source: "keyboard" | "click" | "search",
		) => void;
		virtualize?: boolean;
		itemSize?: number;
		overscan?: number;
		filters?: Array<(item: T, index: number) => boolean>;
		getKey?: (item: T, index: number) => string | number;
	}

	function onkeydown(event: KeyboardEvent) {
		event.preventDefault();
		const { key } = event;

		match(key)
			.returnType<void>()
			.with("ArrowDown", () => {
				next();
			})
			.with("ArrowUp", () => {
				previous();
			})
			.with("Home", () => {
				setActiveIndex(0);
			})
			.with("End", () => {
				setActiveIndex(data.length - 1);
			})
			.otherwise(() => {});
	}

	let {
		data,
		class: className,
		activeIndex = $bindable(-1),
		height = "100%",
		width = "100%",
		overscan = 5,
		virtualize,
		onSelect,
		filters,
		style,
		item,
		...rest
	}: Props = $props();
	const uuid = $props.id();

	export const id = `listbox-${uuid}`;

	export function setActiveIndex(
		index: number,
		scrollIntoView = true,
		source: "keyboard" | "click" | "search" = "keyboard",
	) {
		activeIndex = index;

		if (scrollIntoView) {
			scrollToIndex(index);
		}

		if (onSelect) {
			onSelect(data[index], index, source);
		}
	}

	export function scrollToIndex(index: number, smooth = true) {
		if (vListRef) {
			const outOfBounds =
				vListRef.findEndIndex() + overscan < index || vListRef.findStartIndex() - overscan > index;

			vListRef.scrollToIndex(index, {
				align: "nearest",
				smooth: !prefersReducedMotion.current && smooth && !outOfBounds,
			});
		} else {
			ref.querySelector(`[data-index="${index}"]`)?.scrollIntoView({
				block: "nearest",
				behavior: prefersReducedMotion.current || !smooth ? "auto" : "smooth",
			});
		}
	}

	export function next() {
		const currentOriginalIndex = filteredData.findIndex(
			(item) => item.originalIndex === activeIndex,
		);

		const next = filteredData.at(
			Math.min(currentOriginalIndex + 1, filteredData.length - 1),
		);

		if (!next || next.originalIndex === currentOriginalIndex) {
			return;
		}

		setActiveIndex(next.originalIndex);
	}

	export function previous() {
		const currentOriginalIndex = filteredData.findIndex(
			(item) => item.originalIndex === activeIndex,
		);

		const prev = filteredData.at(Math.max(currentOriginalIndex - 1, 0));
		if (!prev || prev.originalIndex === currentOriginalIndex) {
			return;
		}

		setActiveIndex(prev.originalIndex);
	}

	function updateFilteredData() {
		filteredData = data
			.map(
				(item, index) => ({ data: item, originalIndex: index }) as FilteredItem,
			)
			.filter(
				(item) =>
					filters?.every((filter) => filter(item.data, item.originalIndex)) ??
					true,
			);
	}

	export function update() {
		updateFilteredData();

		if (filteredData.length > 0) {
			setActiveIndex(
				filteredData[0].originalIndex,
				false,
				"search",
			);
		}
	}

	$effect(() => {
		if (data.length > 0) {
			updateFilteredData();
		}
	});

	let ref: HTMLDivElement;
	let vListRef: ReturnType<typeof VList> | undefined = $state();
	let filteredData: FilteredItem[] = $state([]);
</script>

{#snippet row(itemData: T, index: number)}
	<div
		id={`${id}-option-${index}`}
		data-index={index}
		tabindex="-1"
		aria-selected={activeIndex === index}
		aria-setsize={data.length}
		aria-posinset={index + 1}
		onpointerdown={() => setActiveIndex(index, false, "click")}
		class="cursor-pointer select-none hover:bg-purple-300/10 aria-selected:bg-primary-500/50 aria-selected:text-(--selected-color)"
		role="option"
	>
		{@render item(itemData, index)}
	</div>
{/snippet}

<div
	{id}
	role="listbox"
	bind:this={ref}
	tabindex="0"
	{style}
	{onkeydown}
	class={["outline-primary-600", className]}
	style:height={typeof height === "string" ? height : `${height}px`}
	style:width={typeof width === "string" ? width : `${width}px`}
	aria-activedescendant={activeIndex === -1
		? undefined
		: `${id}-option-${activeIndex}`}
	{...rest}
>
	{#if virtualize}
		<VList
			bind:this={vListRef}
			data={filteredData}
			{overscan}
			getKey={(_, index) => index}
		>
			{#snippet children(item, _)}
				{@render row(item.data, item.originalIndex)}
			{/snippet}
		</VList>
	{:else}
		{#each filteredData as item}
			{@render row(item.data, item.originalIndex)}
		{/each}
	{/if}
</div>
