import { Sha256 } from '@aws-crypto/sha256-browser';

export const hashSeed = async (
	boardId: string,
	username: string,
	timestamp: string
): Promise<number> => {
	const hasher = new Sha256();
	hasher.update(boardId);
	hasher.update(username);
	hasher.update(timestamp);
	const hash = await hasher.digest();
	const dataView = new DataView(hash.buffer);
	return dataView.getUint32(0, true); // Read the first 4 bytes as a little-endian u32
};

export const rndRange = async (
	boardId: string,
	username: string,
	timestamp: string,
	min: number,
	max: number
): Promise<number> => {
	const seed = await hashSeed(boardId, username, timestamp);
	return (seed % (max - min)) + min;
};
