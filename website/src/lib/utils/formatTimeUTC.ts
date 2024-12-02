export const formatTimeUTC = (time?: string) => {
	if (!time) return '';
	return new Date(Number(time)).toLocaleString('en-US', {
		timeZone: 'UTC',
		year: 'numeric',
		month: 'long',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		hour12: true
	});
};
