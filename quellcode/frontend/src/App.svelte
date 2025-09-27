<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import FontSelector from "@components/input/FontSelector.svelte";
	import { onMount} from "svelte";
	import Combobox from "@components/input/Combobox.svelte";
	import CodeView from "@components/input/CodeView.svelte";
	import { watch } from "runed";

	let syntectThemes: string[] = $state([]);
	let syntectLanguages: string[] = $state([]);

	onMount(async () => {
		syntectThemes = await invoke("themes");
		syntectLanguages = await invoke("syntaxes");
	});

	let activeLanguage: string | null = $state(null);
	let fontFamily: string | null = $state(null);
	let cssStyleSheet = new CSSStyleSheet();

	async function handleThemeChange(theme: string) {
		const css = await invoke<string>("get_css_for_theme", {
			theme,
		});

		cssStyleSheet.replaceSync(css);
	}

	onMount(() => {
		document.adoptedStyleSheets.push(cssStyleSheet);
	});

	watch(() => syntectThemes, (themes, prev) => {
		if (themes.length > 0 && themes.some((theme, index) => theme !== prev?.[index])) {
			handleThemeChange(syntectThemes[0]);
		}
	});

	watch(
		[() => syntectLanguages],
		(syntaxes, prev) => {
			if (
				syntaxes.length > 0 &&
				syntaxes.some((syntax, index) => syntax !== prev?.[index])
			) {
				activeLanguage = syntaxes[0].at(0) || null;
			}
		},
	);
</script>

<div class="overflow-hidden w-screen grid grid-cols-[1fr_300px]">
	<main class="p-2 h-screen bg-base-200">
		<div class="grid gap-2 grid-rows-2 h-full">
			<CodeView
				syntax={activeLanguage || "Rust"}
				fontFamily={fontFamily || undefined}
				class=""
				editable
			></CodeView>
			<pre
				class="font-mono syntect-code border border-black/50 shadow-md size-full overflow-auto rounded-theme"
				style:font-family={fontFamily}></pre>
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
						onChange={(f) => (fontFamily = f.name || null)}
						defaultFamily="JetBrains Mono"
					/>
				</label>
				<label>
					Theme
					<Combobox
						data={syntectThemes}
						activeIndex={0}
						getDisplayText={(item) => item}
						searchFilter={(query, item) =>
							item.toLowerCase().includes(query.toLowerCase())}
						label="Theme"
						class="w-full"
						onActivate={(item) => handleThemeChange(item)}
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
						activeIndex={0}
						onActivate={(item) => (activeLanguage = item)}
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
