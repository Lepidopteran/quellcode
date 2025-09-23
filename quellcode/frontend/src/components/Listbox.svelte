<script lang="ts" generics="T">
	import type { ClassValue, HTMLAttributes } from "svelte/elements";
	import { generateId } from "../lib/util";
	import type { Snippet } from "svelte";
	import { match } from "ts-pattern";
	import { prefersReducedMotion } from "svelte/motion";

	interface Props extends HTMLAttributes<HTMLDivElement> {
		data: T[];
		class?: ClassValue;
		height?: number | string;
		width?: number | string;
		item: Snippet<[item: T, index: number]>;
		onSelect?: (item: T, index?: number) => void;
	}

	let id = generateId("listbox");
	let activeIndex = $state(-1);

	function onkeydown(event: KeyboardEvent) {
		event.preventDefault();
		const { key } = event;

		match(key)
			.returnType<void>()
			.with("ArrowDown", () => {
				setActiveIndex(activeIndex + (1 % data.length));
			})
			.with("ArrowUp", () => {
				setActiveIndex(activeIndex - (1 % data.length));
			})
			.with("Home", () => {
				setActiveIndex(0);
			})
			.with("End", () => {
				setActiveIndex(data.length - 1);
			})
			.otherwise(() => {});
	}

	function setActiveIndex(index: number, scrollIntoView = true) {
		activeIndex = index;

		if (scrollIntoView) {
			ref.querySelector(`[data-index="${index}"]`)?.scrollIntoView({
				block: "nearest",
				behavior: prefersReducedMotion.current ? "auto" : "smooth",
			});
		}

		if (onSelect) {
			onSelect(data[index], index);
		}
	}

	function onclick(event: MouseEvent) {
		event.preventDefault();

		if (event.target === ref || !event.target) {
			return;
		}

		const target = event.target as HTMLElement;
		const index = Number(target.dataset.index);

		setActiveIndex(index, false);
	}

	let {
		data,
		class: className,
		style,
		height = "100%",
		width = "100%",
		onSelect,
		item,
		...rest
	}: Props = $props();

	let ref: HTMLDivElement;
</script>

<div
	{id}
	role="listbox"
	bind:this={ref}
	tabindex="0"
	{style}
	{onkeydown}
	{onclick}
	class={["outline-purple-600", className]}
	style:height={typeof height === "string" ? height : `${height}px`}
	style:width={typeof width === "string" ? width : `${width}px`}
	aria-activedescendant={`${id}-option-${activeIndex}`}
	{...rest}
>
	{#each data as itemData, index}
		<div
			id={`${id}-option-${index}`}
			data-index={index}
			aria-selected={activeIndex === index}
			class="cursor-pointer select-none aria-selected:bg-purple-600 aria-selected:text-white"
			role="option"
		>
			{@render item(itemData, index)}
		</div>
	{/each}
</div>
