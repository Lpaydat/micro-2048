<script lang="ts">
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
</script>

<span>{timeAgo}</span>