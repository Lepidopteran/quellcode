<script lang="ts" generics="T">
	import { onMount, type Component, type Snippet } from "svelte";
	import Listbox from "./Listbox.svelte";
	import type { ClassValue, HTMLAttributes } from "svelte/elements";
	import Button from "@components/input/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import { match } from "ts-pattern";
	import { autoUpdate, computePosition, shift } from "@floating-ui/dom";
	import { flip } from "@floating-ui/dom";
	import { size } from "@floating-ui/dom";

	interface Props extends HTMLAttributes<HTMLDivElement> {
		data: T[];
		item: Snippet<[item: T, index: number]>;
		getDisplayText: (item: T) => string;
		searchFilter: (query: string, item: T, index: number) => boolean;
		activeIndex?: number;
		label?: string;
		class?: ClassValue;
		/** The max height of the listbox */
		maxHeight?: number | string;
		onActivate?: (item: T, index?: number) => void;
		onSelect?: (item: T, index?: number) => void;
		filters?: Array<(item: T, index: number) => boolean>;
		virtualize?: boolean;
		value?: T;
	}

	let dropdownExpanded = $state(false);

	const uuid = $props.id();
	export const id = `combobox-${uuid}`;

	let {
		data,
		class: className,
		maxHeight = 300,
		activeIndex = $bindable(-1),
		searchFilter,
		filters = [],
		getDisplayText,
		onActivate,
		onSelect,
		item,
		virtualize,
		label = "",
		...rest
	}: Props = $props();

	let listboxRef: ReturnType<typeof Listbox> | null = $state(null);
	let floatingRef: HTMLDivElement | null = $state(null);
	let textInputRef: HTMLInputElement | null = $state(null);
	let buttonRef: ReturnType<typeof Button> | null = $state(null);
	let disableSearchFilter = $state(false);
	let query = $state("");

	function handleSelect(
		item: T,
		index: number,
		source: "keyboard" | "click" | "search",
		activate = false,
	) {
		const input = textInputRef as HTMLInputElement;

		if (source === "click" || activate) {
			input.value = getDisplayText(item);
			dropdownExpanded = false;

			onActivate?.(item, index);

			return;
		}

		onSelect?.(item, index);
	}

	function onkeydown(event: KeyboardEvent) {
		const { key, altKey } = event;

		match(key)
			.returnType<void>()
			.with("Escape", () => {
				if (!dropdownExpanded && textInputRef) {
					textInputRef.value = "";
				}

				dropdownExpanded = false;
			})
			.with("ArrowDown", () => {
				event.preventDefault();
				if (altKey) {
					dropdownExpanded = true;
					return;
				}

				if (dropdownExpanded) {
					listboxRef?.next();
				}
			})
			.with("ArrowUp", () => {
				event.preventDefault();
				if (altKey) {
					dropdownExpanded = false;
					return;
				}

				if (dropdownExpanded) {
					listboxRef?.previous();
				}
			})
			.with("Enter", () => {
				dropdownExpanded = false;

				if (activeIndex !== -1) {
					handleSelect(data[activeIndex], activeIndex, "keyboard", true);
				}
			})
			.with("Tab", () => {
				dropdownExpanded = false;
			})
			.otherwise(() => {
				query = textInputRef?.value ?? "";
				listboxRef?.update();
			});
	}

	export function focus() {
		textInputRef?.focus();
	}

	export function blur() {
		textInputRef?.blur();
	}

	export function open() {
		dropdownExpanded = true;
	}

	export function close() {
		dropdownExpanded = false;
	}

	export function toggle() {
		dropdownExpanded = !dropdownExpanded;
	}

	export function search(query: string) {
		disableSearchFilter = query === "";
		listboxRef?.update();
	}

	export function scrollToIndex(index: number, smooth = true) {
		listboxRef?.scrollToIndex(index, smooth);
	}

	export function update() {
		listboxRef?.update();
	}

	$effect(() => {
		if (!textInputRef || !floatingRef) {
			return;
		}

		const cleanup = autoUpdate(textInputRef, floatingRef, () => {
			computePosition(textInputRef!, floatingRef!, {
				placement: "bottom-start",
				middleware: [
					flip(),
					shift(),
					size({
						apply({ rects }) {
							Object.assign(floatingRef!.style, {
								width: `${rects.reference.width}px`,
							});
						},
					}),
				],
			}).then(({ x, y }) => {
				Object.assign(floatingRef!.style, {
					left: `${x}px`,
					top: `${y}px`,
				});
			});
		});

		return () => {
			cleanup();
		};
	});
</script>

<div
	class={[
		"items-center border inline-flex border-black/50 bg-base-200 rounded-theme focus-within:border-primary-500/50 overflow-hidden",
		className,
	]}
	{...rest}
>
	<input
		type="text"
		role="combobox"
		bind:this={textInputRef}
		value={activeIndex !== -1 && data.length > 0 ? getDisplayText(data[activeIndex]) : ""}
		aria-controls={listboxRef?.id}
		aria-expanded={dropdownExpanded}
		aria-label={label}
		aria-autocomplete="list"
		onblur={() => {
			if (dropdownExpanded && buttonRef?.id !== document.activeElement?.id) {
				dropdownExpanded = false;
			}
		}}
		onfocus={() => {
			if (!dropdownExpanded) {
				dropdownExpanded = true;
			}
		}}
		{onkeydown}
		class="input truncate focus:outline-none px-2 py-1 inset-shadow-sm inset-shadow-black/50 flex-1"
	/>
	<Button
		bind:this={buttonRef}
		aria-controls={listboxRef?.id}
		aria-label={label}
		tabindex={-1}
		variant="ghost"
		aria-expanded={dropdownExpanded}
		onclick={() => {
			if (dropdownExpanded) {
				textInputRef?.focus();
				dropdownExpanded = false;

				return;
			} else {
				dropdownExpanded = true;

				return;
			}
		}}
		class="shadow py-1"
	>
		<Icon
			name="down-small-fill"
			size="1.5em"
			class={[
				"transition-transform duration-100",
				dropdownExpanded ? "rotate-180" : "",
			]}
		></Icon>
	</Button>
</div>
<div
	bind:this={floatingRef}
	style:max-height={typeof maxHeight === "string"
		? maxHeight
		: `${maxHeight}px`}
	class={[
		"absolute z-10 w-full backdrop-blur-3xl shadow overflow-y-auto border border-black/50 mt-1 rounded-theme transition-[height,_opacity] duration-[150ms,_200ms]",
		dropdownExpanded ? "opacity-100 h-full" : "opacity-0 h-0",
	]}
>
	<Listbox
		onSelect={handleSelect}
		bind:this={listboxRef}
		bind:activeIndex
		filters={[
			(item, index) => {
				if (disableSearchFilter || !textInputRef?.value) {
					return true;
				}

				return searchFilter(query, item, index);
			},
			...filters,
		]}
		tabindex={-1}
		{virtualize}
		width="100%"
		{data}
		{item}
	></Listbox>
</div>
