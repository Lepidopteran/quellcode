<script lang="ts">
	import { icons as mingcute } from "@iconify-json/mingcute";

	import {
		iconToSVG,
		getIconData,
	} from "@iconify/utils";

	import type { Icons } from "../lib/icons";
	import type { ClassValue } from "svelte/elements";


	interface Props {
		name: Icons;
		hFlip?: boolean;
		vFlip?: boolean;
		rotate?: number | string;
		size?: number | string;
		class?: ClassValue;
		[key: string]: unknown;
	}

	let {
		name,
		size = "1em",
		hFlip = false,
		vFlip = false,
		class: className,
		...rest
	}: Props = $props();

	const iconData = getIconData(mingcute, name);

	if (!iconData) {
		throw new Error(`Icon ${name} not found`);
	}

	const renderData = iconToSVG(iconData, {
		hFlip,
		vFlip,
		width: size,
		height: size,
	});
</script>

<svg
	{...renderData.attributes}
	class={[className]}
>
	{@html renderData.body}
</svg>
