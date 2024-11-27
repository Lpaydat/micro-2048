export const preventDefault = <T extends Event>(fn: (event: T) => void) => {
	return (event: T) => {
		event.preventDefault();
		fn.call(this, event);
	};
};
