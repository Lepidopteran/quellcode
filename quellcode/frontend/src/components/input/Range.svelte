<script lang="ts">
	import type { ClassValue, HTMLInputAttributes } from "svelte/elements";

	interface Props
		extends Pick<
			HTMLInputAttributes,
			| "onchange"
			| "oninput"
			| "value"
			| "id"
			| "list"
			| "disabled"
			| "readonly"
			| "aria-label"
		> {
		min?: number;
		max?: number;
		ticks?: number[];
		step?: number;
		value?: number;
		class?: ClassValue;
	}

	let {
		min = 0,
		max = 100,
		step = 1,
		value = $bindable(0),
		ticks = [],
		class: className,
		...rest
	}: Props = $props();

	let progress = $derived(((value - min) / (max - min)) * 100);
</script>

<input
	type="range"
	{min}
	{max}
	{step}
	style:box-shadow={`inset 0 1px 1px 0 rgb(0 0 0 / 0.5)`}
	style:background={`linear-gradient(to right, var(--color-primary-500) ${progress}%, var(--color-base-300) ${progress}%)`}
	bind:value
	class={["w-full", className]}
	{...rest}
/>

<style>
	@layer components {
		input[type="range"] {
			--thumb-size: 16px;
			appearance: none;
			background: var(--base-content);
			outline: none;
			border-radius: 9999px;
			transition: opacity 0.2s;

			&::-webkit-slider-runnable-track {
				cursor: pointer;
				height: calc(var(--thumb-size) / 2);
			}

			&::-webkit-slider-thumb {
				cursor: pointer;
				appearance: none;
				-webkit-appearance: none;
				background: var(--color-primary-500);
				width: var(--thumb-size);
				height: var(--thumb-size);
				border-radius: 9999px;
				top: 50%;
				transform: translateY(-50%);
				position: relative;
			}
		}
	}
</style>
