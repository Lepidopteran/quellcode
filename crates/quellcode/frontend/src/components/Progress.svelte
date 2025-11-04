<script lang="ts">
	import type { ClassValue } from "svelte/elements";

	interface Props {
		value?: number;
		max?: number;
		class?: ClassValue;
	}

	let {
		value = $bindable(0),
		max = 100,
		class: className,
		...rest
	}: Props = $props();
</script>

<div
	role="progressbar"
	aria-valuemax={Number(max)}
	aria-valuenow={Number(value)}
	class={["progress", className]}
	{...rest}
>
	<div
		class="progress-bar"
		style="width: {((Number(value) * 100) / Number(max)) | 0}%"
	></div>
	<span class="sr-only">{(Number(value) * 100) / Number(max)}%</span>
</div>

<style>
	@layer components {
		.progress {
			display: inline-flex;
			background-color: var(--color-base-300);
			height: 0.5rem;
			border-radius: var(--radius-theme);
			position: relative;
			overflow: hidden;
			min-width: 100px;
		}

		.progress-bar {
			height: 100%;
			background-color: var(--color-primary-500);
		}
	}
</style>
