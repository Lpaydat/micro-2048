<script lang="ts">
	import { onMount } from "svelte";

    export let time: string;

    const MINUTE = 60 * 1000;
    const HOUR = 60 * MINUTE;
    const DAY = 24 * HOUR;

    function getTimeAgo(timestamp: Date): string {
        const diff = Date.now() - timestamp.getTime();

        if (diff < MINUTE) {
            return 'just now';
        } else if (diff < HOUR) {
            const minutes = Math.floor(diff / MINUTE);
            return `${minutes} ${minutes === 1 ? 'minute' : 'minutes'} ago`;
        } else if (diff < DAY) {
            const hours = Math.floor(diff / HOUR);
            return `${hours} ${hours === 1 ? 'hour' : 'hours'} ago`;
        } else {
            const days = Math.floor(diff / DAY);
            return `${days} ${days === 1 ? 'day' : 'days'} ago`;
        }
    }

    $: formattedTime = new Date(parseInt(time));
    $: timeAgo = getTimeAgo(formattedTime);

    // Add an interval to update the timeAgo every minute
    let interval: NodeJS.Timeout;
    onMount(() => {
        interval = setInterval(() => {
            timeAgo = getTimeAgo(formattedTime);
        }, 5000);

        return () => clearInterval(interval);
    });
</script>

<span>{timeAgo}</span>