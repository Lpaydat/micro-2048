export const getMessageBlockheight = (data: {
	notifications?: {
		reason?: {
			NewIncomingBundle?: { height: number };
			NewBlock?: { height: number; hash: string };
		};
	};
}) => {
	return data?.notifications?.reason?.NewBlock?.height;
};
