<script>
	import { generateTabletFromMatrix } from "$lib/game/utils";
	import Tablet from "./Tablet.svelte";

    const board = [[3, 6, 2, 8], [1, 0, 4, 5], [2, 5, 2, 3], [1, 4, 2, 1]];
    const tablet = generateTabletFromMatrix(board);
</script>

<div class="flex justify-center items-center">
    <Tablet tablet={tablet} />
</div>
