/**
 * Cache Service - IndexedDB-backed persistent cache
 * 
 * Provides fast, persistent caching for:
 * - Board states
 * - Move history (chunked)
 * - Leaderboard data
 * 
 * Features:
 * - TTL-based expiration
 * - LRU eviction when storage is full
 * - Chunked storage for large data (move history)
 * - Graceful degradation when IndexedDB unavailable
 */

import { browser } from '$app/environment';

const DB_NAME = 'u2048-cache';
const DB_VERSION = 1;

// Store names
const STORES = {
	BOARDS: 'boards',
	MOVES: 'moves',
	LEADERBOARDS: 'leaderboards',
	METADATA: 'metadata'
} as const;

// Default TTLs (in milliseconds)
export const TTL = {
	BOARD: 5 * 60 * 1000, // 5 minutes
	MOVES: 30 * 60 * 1000, // 30 minutes (moves don't change once made)
	LEADERBOARD: 30 * 1000, // 30 seconds (frequently updated)
	LEADERBOARD_STALE: 5 * 60 * 1000 // 5 minutes for stale-while-revalidate
} as const;

export interface CacheEntry<T> {
	data: T;
	timestamp: number;
	ttl: number;
	accessCount: number;
	lastAccess: number;
}

export interface BoardCacheData {
	boardId: string;
	board: number[][];
	score: number;
	isEnded: boolean;
	player: string;
	chainId: string;
	leaderboardId: string;
	createdAt: string;
	totalMoves: number;
}

export interface MoveChunkData {
	boardId: string;
	startIndex: number;
	moves: {
		direction: string;
		timestamp: string;
		boardAfter: number[][];
		scoreAfter: number;
	}[];
}

export interface LeaderboardCacheData {
	leaderboardId: string;
	name: string;
	description?: string;
	host: string;
	startTime: string;
	endTime: string;
	totalBoards: number;
	totalPlayers: number;
	rankers: {
		username: string;
		score: number;
		boardId: string;
		isEnded?: boolean;
	}[];
	lastUpdate: string;
}

class CacheService {
	private db: IDBDatabase | null = null;
	private dbReady: Promise<boolean>;
	private memoryCache: Map<string, CacheEntry<unknown>> = new Map();
	private maxMemoryCacheSize = 100;

	constructor() {
		this.dbReady = this.initDB();
	}

	/**
	 * Initialize IndexedDB
	 */
	private async initDB(): Promise<boolean> {
		if (!browser) return false;

		return new Promise((resolve) => {
			try {
				const request = indexedDB.open(DB_NAME, DB_VERSION);

				request.onerror = () => {
					console.warn('IndexedDB not available, using memory cache only');
					resolve(false);
				};

				request.onsuccess = () => {
					this.db = request.result;
					console.log('✅ IndexedDB cache initialized');
					resolve(true);
				};

				request.onupgradeneeded = (event) => {
					const db = (event.target as IDBOpenDBRequest).result;

					// Create object stores
					if (!db.objectStoreNames.contains(STORES.BOARDS)) {
						db.createObjectStore(STORES.BOARDS, { keyPath: 'key' });
					}
					if (!db.objectStoreNames.contains(STORES.MOVES)) {
						const moveStore = db.createObjectStore(STORES.MOVES, { keyPath: 'key' });
						moveStore.createIndex('boardId', 'boardId', { unique: false });
					}
					if (!db.objectStoreNames.contains(STORES.LEADERBOARDS)) {
						db.createObjectStore(STORES.LEADERBOARDS, { keyPath: 'key' });
					}
					if (!db.objectStoreNames.contains(STORES.METADATA)) {
						db.createObjectStore(STORES.METADATA, { keyPath: 'key' });
					}
				};
			} catch (error) {
				console.warn('IndexedDB initialization failed:', error);
				resolve(false);
			}
		});
	}

	// ==================== BOARD CACHE ====================

	/**
	 * Get cached board state
	 */
	async getBoard(boardId: string): Promise<BoardCacheData | null> {
		const key = `board:${boardId}`;
		return this.get<BoardCacheData>(STORES.BOARDS, key, TTL.BOARD);
	}

	/**
	 * Cache board state
	 */
	async setBoard(boardId: string, data: BoardCacheData, ttl: number = TTL.BOARD): Promise<void> {
		const key = `board:${boardId}`;
		await this.set(STORES.BOARDS, key, data, ttl);
	}

	/**
	 * Get board even if stale (for stale-while-revalidate)
	 */
	async getBoardStale(boardId: string): Promise<{ data: BoardCacheData; isStale: boolean } | null> {
		const key = `board:${boardId}`;
		return this.getWithStale<BoardCacheData>(STORES.BOARDS, key, TTL.BOARD);
	}

	// ==================== MOVE HISTORY CACHE ====================

	/**
	 * Get cached move chunk
	 */
	async getMoveChunk(boardId: string, startIndex: number): Promise<MoveChunkData | null> {
		const key = `moves:${boardId}:${startIndex}`;
		return this.get<MoveChunkData>(STORES.MOVES, key, TTL.MOVES);
	}

	/**
	 * Cache move chunk
	 */
	async setMoveChunk(
		boardId: string,
		startIndex: number,
		moves: MoveChunkData['moves'],
		ttl: number = TTL.MOVES
	): Promise<void> {
		const key = `moves:${boardId}:${startIndex}`;
		const data: MoveChunkData = { boardId, startIndex, moves };
		await this.set(STORES.MOVES, key, data, ttl, { boardId });
	}

	/**
	 * Get all cached move chunks for a board
	 */
	async getAllMoveChunks(boardId: string): Promise<MoveChunkData[]> {
		if (!this.db) return [];

		return new Promise((resolve) => {
			try {
				const transaction = this.db!.transaction(STORES.MOVES, 'readonly');
				const store = transaction.objectStore(STORES.MOVES);
				const index = store.index('boardId');
				const request = index.getAll(boardId);

				request.onsuccess = () => {
					const results = request.result || [];
					const chunks: MoveChunkData[] = [];
					const now = Date.now();

					for (const entry of results) {
						if (entry.timestamp + entry.ttl > now) {
							chunks.push(entry.data);
						}
					}

					// Sort by startIndex
					chunks.sort((a, b) => a.startIndex - b.startIndex);
					resolve(chunks);
				};

				request.onerror = () => resolve([]);
			} catch {
				resolve([]);
			}
		});
	}

	/**
	 * Clear all move chunks for a board
	 */
	async clearMoveChunks(boardId: string): Promise<void> {
		if (!this.db) return;

		const chunks = await this.getAllMoveChunks(boardId);
		for (const chunk of chunks) {
			const key = `moves:${boardId}:${chunk.startIndex}`;
			await this.delete(STORES.MOVES, key);
		}
	}

	// ==================== LEADERBOARD CACHE ====================

	/**
	 * Get cached leaderboard
	 */
	async getLeaderboard(leaderboardId: string): Promise<LeaderboardCacheData | null> {
		const key = `leaderboard:${leaderboardId || 'main'}`;
		return this.get<LeaderboardCacheData>(STORES.LEADERBOARDS, key, TTL.LEADERBOARD);
	}

	/**
	 * Cache leaderboard
	 */
	async setLeaderboard(
		leaderboardId: string,
		data: LeaderboardCacheData,
		ttl: number = TTL.LEADERBOARD
	): Promise<void> {
		const key = `leaderboard:${leaderboardId || 'main'}`;
		await this.set(STORES.LEADERBOARDS, key, data, ttl);
	}

	/**
	 * Get leaderboard even if stale
	 */
	async getLeaderboardStale(
		leaderboardId: string
	): Promise<{ data: LeaderboardCacheData; isStale: boolean } | null> {
		const key = `leaderboard:${leaderboardId || 'main'}`;
		return this.getWithStale<LeaderboardCacheData>(
			STORES.LEADERBOARDS,
			key,
			TTL.LEADERBOARD_STALE
		);
	}

	// ==================== GENERIC CACHE OPERATIONS ====================

	/**
	 * Get item from cache (memory first, then IndexedDB)
	 */
	private async get<T>(store: string, key: string, maxAge: number): Promise<T | null> {
		// Check memory cache first
		const memEntry = this.memoryCache.get(key);
		if (memEntry && Date.now() - memEntry.timestamp < maxAge) {
			memEntry.accessCount++;
			memEntry.lastAccess = Date.now();
			return memEntry.data as T;
		}

		// Check IndexedDB
		const dbEntry = await this.getFromDB<T>(store, key);
		if (dbEntry && Date.now() - dbEntry.timestamp < maxAge) {
			// Promote to memory cache
			this.setMemoryCache(key, dbEntry);
			return dbEntry.data;
		}

		return null;
	}

	/**
	 * Get item with stale flag (for stale-while-revalidate)
	 */
	private async getWithStale<T>(
		store: string,
		key: string,
		staleTTL: number
	): Promise<{ data: T; isStale: boolean } | null> {
		// Check memory cache
		const memEntry = this.memoryCache.get(key);
		if (memEntry) {
			const age = Date.now() - memEntry.timestamp;
			if (age < staleTTL) {
				memEntry.accessCount++;
				memEntry.lastAccess = Date.now();
				return {
					data: memEntry.data as T,
					isStale: age > memEntry.ttl
				};
			}
		}

		// Check IndexedDB
		const dbEntry = await this.getFromDB<T>(store, key);
		if (dbEntry) {
			const age = Date.now() - dbEntry.timestamp;
			if (age < staleTTL) {
				this.setMemoryCache(key, dbEntry);
				return {
					data: dbEntry.data,
					isStale: age > dbEntry.ttl
				};
			}
		}

		return null;
	}

	/**
	 * Set item in cache (both memory and IndexedDB)
	 */
	private async set<T>(
		store: string,
		key: string,
		data: T,
		ttl: number,
		indexes?: Record<string, string>
	): Promise<void> {
		const entry: CacheEntry<T> = {
			data,
			timestamp: Date.now(),
			ttl,
			accessCount: 1,
			lastAccess: Date.now()
		};

		// Set in memory cache
		this.setMemoryCache(key, entry);

		// Set in IndexedDB
		await this.setToDB(store, key, entry, indexes);
	}

	/**
	 * Delete item from cache
	 */
	private async delete(store: string, key: string): Promise<void> {
		this.memoryCache.delete(key);
		await this.deleteFromDB(store, key);
	}

	// ==================== MEMORY CACHE HELPERS ====================

	private setMemoryCache<T>(key: string, entry: CacheEntry<T>): void {
		// Evict LRU entries if cache is full
		if (this.memoryCache.size >= this.maxMemoryCacheSize) {
			this.evictLRU();
		}
		this.memoryCache.set(key, entry as CacheEntry<unknown>);
	}

	private evictLRU(): void {
		let oldestKey: string | null = null;
		let oldestAccess = Infinity;

		for (const [key, entry] of this.memoryCache) {
			if (entry.lastAccess < oldestAccess) {
				oldestAccess = entry.lastAccess;
				oldestKey = key;
			}
		}

		if (oldestKey) {
			this.memoryCache.delete(oldestKey);
		}
	}

	// ==================== INDEXEDDB HELPERS ====================

	private async getFromDB<T>(store: string, key: string): Promise<CacheEntry<T> | null> {
		if (!this.db) return null;

		return new Promise((resolve) => {
			try {
				const transaction = this.db!.transaction(store, 'readonly');
				const objectStore = transaction.objectStore(store);
				const request = objectStore.get(key);

				request.onsuccess = () => {
					resolve(request.result || null);
				};

				request.onerror = () => resolve(null);
			} catch {
				resolve(null);
			}
		});
	}

	private async setToDB<T>(
		store: string,
		key: string,
		entry: CacheEntry<T>,
		indexes?: Record<string, string>
	): Promise<void> {
		await this.dbReady;
		if (!this.db) return;

		return new Promise((resolve) => {
			try {
				const transaction = this.db!.transaction(store, 'readwrite');
				const objectStore = transaction.objectStore(store);
				const request = objectStore.put({ key, ...entry, ...indexes });

				request.onsuccess = () => resolve();
				request.onerror = () => resolve();
			} catch {
				resolve();
			}
		});
	}

	private async deleteFromDB(store: string, key: string): Promise<void> {
		if (!this.db) return;

		return new Promise((resolve) => {
			try {
				const transaction = this.db!.transaction(store, 'readwrite');
				const objectStore = transaction.objectStore(store);
				const request = objectStore.delete(key);

				request.onsuccess = () => resolve();
				request.onerror = () => resolve();
			} catch {
				resolve();
			}
		});
	}

	// ==================== UTILITY METHODS ====================

	/**
	 * Clear all cache data
	 */
	async clearAll(): Promise<void> {
		this.memoryCache.clear();

		if (!this.db) return;

		const stores = [STORES.BOARDS, STORES.MOVES, STORES.LEADERBOARDS, STORES.METADATA];

		for (const store of stores) {
			await new Promise<void>((resolve) => {
				try {
					const transaction = this.db!.transaction(store, 'readwrite');
					const objectStore = transaction.objectStore(store);
					const request = objectStore.clear();

					request.onsuccess = () => resolve();
					request.onerror = () => resolve();
				} catch {
					resolve();
				}
			});
		}

		console.log('🗑️ Cache cleared');
	}

	/**
	 * Get cache statistics
	 */
	async getStats(): Promise<{
		memoryEntries: number;
		dbAvailable: boolean;
	}> {
		return {
			memoryEntries: this.memoryCache.size,
			dbAvailable: this.db !== null
		};
	}

	/**
	 * Check if cache is ready
	 */
	async isReady(): Promise<boolean> {
		return this.dbReady;
	}
}

// Singleton instance
export const cacheService = new CacheService();

export default CacheService;
