<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLInputAttributes } from "svelte/elements";
	interface Props extends HTMLInputAttributes {
		value?: string | null;
		variant?: "base" | "ghost";
		floatingLabel?: boolean;
		prefixChild?: Snippet;
		suffixChild?: Snippet;
		[rest: string]: unknown;
	}

	let {
		value = $bindable(""),
		variant = "base",
		required = true,
		prefixChild,
		suffixChild,
		placeholder,
		pattern,
		class: className,
		...rest
	}: Props = $props();
</script>

<div class={["input", `input-${variant} focus-within:outline outline-primary`, className]}>
	{#if prefixChild}
		<span class="prefix-child">
			{@render prefixChild()}
		</span>
	{/if}
	<input class={["outline-0 "]} type="text" {placeholder} bind:value {...rest} />
	{#if suffixChild}
		<span class="suffix-child">
			{@render suffixChild()}
		</span>
	{/if}
</div>

<style>
	.suffix-child,
	.prefix-child {
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	@layer components {
		div.input {
			display: flex;
			position: relative;
			align-items: center;
			border-radius: var(--radius-theme-sm);
			overflow: hidden;

			& > input {
				padding: calc(var(--spacing) * 2) calc(var(--spacing));
			}

			& > input {
				text-align: inherit;
				display: block;
				width: 100%;
			}

			&.input-base > input,
			&.input-base > span:not(.label) {
				background-color: var(--color-base-300);
			}

			&.input-ghost {
				& > input,
				& > span:not(.label) {
					background-color: transparent;
					box-shadow: none;
				}

				&:has(input:not(:disabled):not([disabled])):hover {
					background-color: rgb(from var(--color-base-300) r g b / 50%);
				}
			}
		}
	}
</style>
