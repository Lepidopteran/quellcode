<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import FontSelector from "@components/input/FontSelector.svelte";
	import { onMount } from "svelte";
	import Combobox from "@components/input/Combobox.svelte";
	import CodeView from "@components/input/CodeView.svelte";
	import { LazyStore } from "@tauri-apps/plugin-store";

	const styleSheet = new CSSStyleSheet();
	const store = new LazyStore("state.json");

	let editorTheme: string | null = $state(null);
	let editorSyntax: string | null = $state(null);
	let editorFontSize: number = $state(14);
	let editorFontFamily: string = $state("Monospace");
	let loaded = $state(false);

	let syntectThemes: string[] = $state([]);
	let syntectLanguages: string[] = $state([]);

	onMount(async () => {
		document.adoptedStyleSheets = [styleSheet];
		syntectThemes = await invoke<string[]>("themes");
		syntectLanguages = await invoke<string[]>("syntaxes");

		syntectThemes.sort();
		syntectLanguages.sort();

		const prevEditorTheme = await store.get<string>("editorTheme");
		const prevEditorSyntax = await store.get<string>("editorLanguage");
		const prevEditorFontSize = await store.get<number>("editorFontSize");
		const prevEditorFontFamily = await store.get<string>("editorFontFamily");

		if (prevEditorTheme) {
			editorTheme = prevEditorTheme;
		}

		if (prevEditorSyntax) {
			editorSyntax = prevEditorSyntax;
		}

		if (prevEditorFontFamily) {
			editorFontFamily = prevEditorFontFamily;
		}

		if (prevEditorFontSize) {
			editorFontSize = prevEditorFontSize;
		}

		console.info(
			"Loaded previous state",
			prevEditorTheme,
			prevEditorSyntax,
			prevEditorFontFamily,
			prevEditorFontSize,
		);

		loaded = true;
	});

	$effect(() => {
		if (!loaded) {
			return;
		}

		store.set("editorTheme", editorTheme);
		store.set("editorLanguage", editorSyntax);
		store.set("editorFontFamily", editorFontFamily);
		store.set("editorFontSize", editorFontSize);
	});

	$effect(() => {
		if (!loaded) {
			return;
		}

		const theme = editorTheme || syntectThemes[0];
		(async () => {
			const css = await invoke<string>("get_css_for_theme", {
				theme,
			});
			styleSheet.replace(css);
		})();
	});
</script>

<div class="overflow-hidden w-screen grid grid-cols-[1fr_300px]">
	<main class="p-2 h-screen bg-base-200">
		<div class="grid gap-2 grid-rows-2 h-full">
			<CodeView
				syntax={editorSyntax}
				fontFamily={editorFontFamily || undefined}
				class=""
				editable
			></CodeView>
			<pre
				class="font-mono syntect-code border border-black/50 shadow-md size-full overflow-auto rounded-theme"
				style:font-family={editorFontFamily}></pre>
		</div>
	</main>
	<aside class="p-2 overflow-y-auto h-full bg-base-100">
		<details class="w-full bg-base-200/50 px-2 rounded-theme" open>
			<summary class="font-bold select-none py-2">Code Settings</summary>
			<div>
				<label>
					Code Font
					<FontSelector
						class="w-full"
						onChange={(f) => (editorFontFamily = f.name)}
						defaultFamily={editorFontFamily}
					/>
				</label>
				<label>
					Theme
					<Combobox
						data={syntectThemes}
						activeIndex={(editorTheme && syntectThemes.indexOf(editorTheme)) ||
							0}
						getDisplayText={(item) => item}
						searchFilter={(query, item) =>
							item.toLowerCase().includes(query.toLowerCase())}
						label="Theme"
						class="w-full"
						onActivate={(item) => (editorTheme = item)}
					>
						{#snippet item(item, _)}
							<div class="flex items-center gap-2">
								<span>{item}</span>
							</div>
						{/snippet}
					</Combobox>
				</label>
				<label>
					Language
					<Combobox
						data={syntectLanguages}
						getDisplayText={(item) => item}
						activeIndex={(editorSyntax &&
							syntectLanguages.indexOf(editorSyntax)) ||
							0}
						onActivate={(item) => (editorSyntax = item)}
						searchFilter={(query, item) =>
							item.toLowerCase().includes(query.toLowerCase())}
						label="Language"
						class="w-full"
					>
						{#snippet item(item, _)}
							<div class="flex items-center gap-2">
								<span>{item}</span>
							</div>
						{/snippet}
					</Combobox>
				</label>
			</div>
		</details>
	</aside>
</div>
