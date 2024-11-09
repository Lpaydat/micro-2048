export const getMessageBlockheight = (data: {
	notifications?: {
		reason?: { NewIncomingBundle?: { height: number }; NewBlock?: { height: number } };
	};
}) => {
	return (
		data?.notifications?.reason?.NewIncomingBundle?.height ||
		data?.notifications?.reason?.NewBlock?.height
	);
};
