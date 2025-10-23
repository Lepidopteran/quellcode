<script lang="ts">
	import type { Snippet } from "svelte";
	import type { ClassValue } from "svelte/elements";

	interface Props {
		children?: Snippet;
		vertical?: boolean;
		class?: ClassValue;
	}

	let {
		children,
		class: className,
		vertical = false,
		...rest
	}: Props = $props();
</script>

<div class={["join", vertical && "join-vertical", className]} {...rest}>
	{@render children?.()}
</div>

<style>
	:global {
		.join {
			display: flex;
			justify-content: center;

			&:not(.join-vertical) > :last-child {
				border-top-left-radius: 0;
				border-bottom-left-radius: 0;
			}

			&:not(.join-vertical) > :first-child {
				border-top-right-radius: 0;
				border-bottom-right-radius: 0;
			}

			&.join-vertical > :last-child {
				border-top-right-radius: 0;
				border-top-left-radius: 0;
			}

			&.join-vertical > :first-child {
				border-bottom-right-radius: 0;
				border-bottom-left-radius: 0;
			}
		}
	}
</style>
