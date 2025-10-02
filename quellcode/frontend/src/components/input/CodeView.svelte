<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { watch } from "runed";
	import type { Snippet } from "svelte";
	import type { ClassValue, HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<HTMLElement> {
		syntax?: string | null;
		fontFamily?: string;
		fontSize?: number;
		editable?: boolean;
		class?: ClassValue;
		code?: string;
		renderOutputAsHtml?: boolean;
	}

	let htmlCode = $state("");

	watch(
		[() => syntax, () => code],
		([syntax, code], [prevSyntax, prevCode]) => {
			if (
				(prevSyntax !== syntax || prevCode !== code) &&
				code.length > 0 &&
				syntax
			) {
				(async () => {
					htmlCode = await invoke<string>("generate_html", {
						syntax,
						code,
					});
				})();
			} else if (!syntax && code.length > 0) {
				htmlCode = code;
			} else {
				htmlCode = "";
			}
		},
	);

	const uid = $props.id();
	export const id = `code-view-${uid}`;

	let {
		syntax,
		editable,
		fontFamily = "monospace",
		fontSize: textSize = 16,
		code = $bindable(""),
		class: classValue,
		...rest
	}: Props = $props();
</script>

<div
	{id}
	style:font-size={`${textSize}px`}
	style:font-family={fontFamily}
	class={[
		"code-view syntect-code border border-black/50 p-2 focus-within:border-primary-500/50 rounded-theme overflow-auto relative",
		classValue,
	]}
	{...rest}
>
	<pre class="pointer-none"><code class="syntect-code"
			>{#if syntax}{@html htmlCode}{:else}{code}{/if}
	</code>
	</pre>

	<textarea
		disabled={!editable}
		autocomplete="off"
		class="bg-transparent text-transparent resize-none size-full caret-base-text focus:outline-none"
		autocapitalize="off"
		spellcheck="false"
		bind:value={code}
	></textarea>
</div>

<style>
	@layer components {
		.code-view {
			display: grid;
			grid-template: "code-view";
			place-content: stretch;
			place-items: stretch;
			overflow: auto;

			& > * {
				grid-area: code-view;
			}

			& :is(pre, code, textarea) {
				font-size: inherit;
				font-family: inherit;
			}
		}
	}
</style>
