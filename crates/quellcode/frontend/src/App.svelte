<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import FontSelector from "@components/input/FontSelector.svelte";
	import { onMount } from "svelte";
	import Combobox from "@components/input/Combobox.svelte";
	import CodeView from "@components/input/CodeView.svelte";
	import { LazyStore } from "@tauri-apps/plugin-store";
	import Button from "@components/input/Button.svelte";
	import type { GeneratorInfo } from "@lib/bindings/GeneratorInfo";
	import { fade } from "svelte/transition";
	import { Debounced } from "runed";
	import type { GeneratorOptions } from "@lib/bindings/GeneratorOptions";
	import type { PropertyValue } from "@lib/bindings/PropertyValue";
	import Range from "@components/input/Range.svelte";
	import { open, save } from "@tauri-apps/plugin-dialog";
	import { writeText } from "@tauri-apps/plugin-clipboard-manager";
	import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
	import Icon from "@components/Icon.svelte";
	import Modal from "@components/Modal.svelte";
	import Settings from "@components/settings/Settings.svelte";

	const styleSheet = new CSSStyleSheet();
	const store = new LazyStore("state.json");

	let syntectThemes: string[] = $state([]);
	let syntectLanguages: string[] = $state([]);
	let generators: GeneratorInfo[] = $state([]);

	let editorTheme: string | null = $state(null);
	let editorSyntax: string | null = $state(null);
	let editorFontSize: number = $state(12);
	let editorFontFamily: string = $state("Monospace");
	let editorCode: string = $state("");
	let outputCode: string = $state("");

	let debouncedEditorCode = new Debounced(() => editorCode, 1000);
	let debouncededitorFontSize = new Debounced(() => editorFontSize, 1000);

	let fontSelectorRef: ReturnType<typeof FontSelector> | null = $state(null);
	let settingsDialogOpen = $state(false);

	let loaded = $state(false);

	let activeGenerator = $state<string | null>(null);
	let activeGeneratorInfo = $derived<GeneratorInfo | null>(
		activeGenerator
			? (generators.find((g) => g.name === activeGenerator) as GeneratorInfo)
			: null,
	);
	let activeGeneratorOptions = $state<Record<string, PropertyValue>>({});

	onMount(async () => {
		document.adoptedStyleSheets = [styleSheet];
		syntectThemes = await invoke<string[]>("themes");
		syntectLanguages = await invoke<string[]>("syntaxes");
		generators = await invoke<GeneratorInfo[]>("generators");

		syntectThemes.sort();
		syntectLanguages.sort();

		const prevActiveGenerator = await store.get<string>("activeGenerator");
		const prevEditorTheme = await store.get<string>("editorTheme");
		const prevEditorSyntax = await store.get<string>("editorLanguage");
		const prevEditorFontSize = await store.get<number>("editorFontSize");
		const prevEditorFontFamily = await store.get<string>("editorFontFamily");

		if (prevActiveGenerator) {
			activeGenerator = prevActiveGenerator;
		} else {
			activeGenerator = generators[0].name;
		}

		if (prevEditorTheme) {
			editorTheme = prevEditorTheme;
		}

		if (prevEditorSyntax) {
			editorSyntax = prevEditorSyntax;
		}

		if (prevEditorFontFamily) {
			editorFontFamily = prevEditorFontFamily;

			console.log(editorFontFamily.replace(/^["']|["']$/g, ""));

			if (fontSelectorRef) {
				fontSelectorRef.setFamily(editorFontFamily.replace(/^["']|["']$/g, ""));
			}
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
		store.set("activeGenerator", activeGenerator);
	});

	$effect(() => {
		if (!loaded || debouncedEditorCode.current.length <= 0) {
			return;
		}

		const options: GeneratorOptions = {
			fontFamily: editorFontFamily.replace(/^["']|["']$/g, ""),
			fontSize: debouncededitorFontSize.current,
			extra: activeGeneratorOptions,
		};

		(async () => {
			outputCode = await invoke<string>("generate_code", {
				generatorName: activeGenerator,
				syntax: editorSyntax,
				theme: editorTheme,
				code: debouncedEditorCode.current,
				options,
			});
		})();
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

<Modal title="Settings" class="p-0" bind:open={settingsDialogOpen}>
	<Settings app={{ themes: syntectThemes, syntaxes: syntectLanguages }} />
</Modal>

<div
	class="overflow-hidden h-full w-full grid grid-cols-[1fr_300px] grid-rows-[auto_1fr]"
>
	<header
		class="p-2 bg-base-100 col-span-2 flex items-center justify-between shadow-md"
	>
		<Button
			variant="primary"
			onclick={async () => {
				const file = await open({ title: "Open File" });

				if (file) {
					editorCode = await readTextFile(file);
				}
			}}>Open</Button
		>
		<Button
			variant="ghost"
			aria-label="Settings"
			title="Settings"
			class="p-2 inset-shadow-none!"
			onclick={() => {
				settingsDialogOpen = true;
			}}><Icon name="settings-3-line" size="1.25em"></Icon></Button
		>
	</header>
	<main class="p-2 bg-base-200 h-full overflow-hidden shadow-md col-span-1">
		<div class="grid-rows-2 grid grid-cols-1 gap-2 overflow-hidden h-full">
			<CodeView
				syntax={editorSyntax}
				fontFamily={editorFontFamily || undefined}
				fontSize={editorFontSize}
				bind:code={editorCode}
				editable
			></CodeView>
			<CodeView syntax={activeGeneratorInfo?.syntax || null} code={outputCode}
			></CodeView>
		</div>
	</main>
	<aside class="grid grid-cols-1 grid-rows-[1fr_auto] p-2 h-full bg-base-100">
		<div class="space-y-2 h-full overflow-y-auto">
			<label class="block">
				Generator
				<select class="w-full block" bind:value={activeGenerator}>
					{#each generators as generator}
						<option>{generator.name}</option>
					{/each}
				</select>
			</label>
			<details
				class="w-full bg-base-200/50 px-2 rounded-theme shadow-md inset-shadow-sm inset-shadow-white/5"
				open
			>
				<summary class="font-bold select-none py-2">Editor Settings</summary>
				<div class="pb-2 space-y-2">
					<label>
						Theme
						<Combobox
							data={syntectThemes}
							activeIndex={(editorTheme &&
								syntectThemes.indexOf(editorTheme)) ||
								0}
							getDisplayText={(item) => item}
							searchFilter={(query, item) =>
								item.toLowerCase().includes(query.toLowerCase())}
							label="Theme"
							class="w-full"
							onActivate={(item) => (editorTheme = item)}
						>
							{#snippet item(item, _)}
								<div class="flex items-center gap-2 px-2">
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
								<div class="flex items-center gap-2 px-2">
									<span>{item}</span>
								</div>
							{/snippet}
						</Combobox>
					</label>
				</div>
			</details>
			<details
				class="w-full bg-base-200/50 px-2 rounded-theme shadow-md inset-shadow-sm inset-shadow-white/5"
				open
			>
				<summary class="font-bold select-none py-2">Font Settings</summary>
				<div class="pb-2 space-y-2">
					<label>
						Code Font
						<FontSelector
							bind:this={fontSelectorRef}
							class="w-full"
							onChange={(f) => (editorFontFamily = f.name)}
							defaultFamily={editorFontFamily}
						/>
					</label>
					<label>
						<div class="flex justify-between items-center">
							<span class="block mb-1">Font Size</span>
							<span class="text-sm text-base-content/50">
								{editorFontSize}px
							</span>
						</div>
						<div class="flex items-center">
							<span class="mr-2 text-base-text/50">a</span>
							<Range bind:value={editorFontSize} min={8} max={96} />
							<span class="ml-2 text-base-text/50" style="">A</span>
						</div>
					</label>
				</div>
			</details>
			{#if activeGeneratorInfo?.properties?.length}
				<details
					transition:fade
					class="w-full bg-base-200/50 px-2 rounded-theme overflow-hidden shadow-md inset-shadow-sm inset-shadow-white/5"
					open
				>
					<summary class="font-bold select-none py-2"
						>Generator Settings</summary
					>
					<div class="pb-2">
						{#each activeGeneratorInfo.properties as property}
							{@const name =
								property.displayName || property.name.replace("_", " ")}
							<label class="block">
								{#if property.kind === "string"}
									<span class="capitalize">{name}</span>
									<input
										type="text"
										class="w-full"
										bind:value={
											() =>
												activeGeneratorOptions[property.name] ||
												property.default ||
												"",
											(value) => (activeGeneratorOptions[property.name] = value)
										}
									/>
								{/if}
								{#if property.kind === "integer"}
									<span class="capitalize">{name}</span>
									<input
										type="number"
										class="w-full"
										max={property.max}
										min={property.min}
										bind:value={
											() =>
												activeGeneratorOptions[property.name] ||
												property.default ||
												0,
											(value) => (activeGeneratorOptions[property.name] = value)
										}
									/>
								{/if}
								{#if property.kind === "boolean"}
									<input
										type="checkbox"
										bind:checked={
											() =>
												(activeGeneratorOptions[property.name] as boolean) ||
												property.default ||
												false,
											(value) => (activeGeneratorOptions[property.name] = value)
										}
									/>
									<span class="capitalize">{name}</span>
								{/if}
							</label>
						{/each}
					</div>
				</details>
			{/if}
		</div>
		<Button
			disabled={outputCode.length === 0}
			onclick={async () => {
				if (activeGeneratorInfo?.saveable) {
					const filters = [];

					if (
						activeGeneratorInfo?.extensions &&
						activeGeneratorInfo?.extensions?.length > 0
					) {
						filters.push({
							name:
								activeGeneratorInfo.extensions[0]
									.replace(".", "")
									.toUpperCase() || "Code",
							extensions: activeGeneratorInfo.extensions,
						});
					}

					const path = await save({
						filters: [...filters, { name: "All Files", extensions: ["*"] }],
					});

					if (path) {
						await writeTextFile(path, outputCode);
					}
				} else {
					await writeText(outputCode);
				}
			}}
			variant="primary"
			class="mt-auto p-2 rounded-theme"
		>
			{#if activeGeneratorInfo?.saveable}
				<span>Save</span>
			{:else}
				<span>Copy</span>
			{/if}
		</Button>
	</aside>
</div>
