/**
 * Smart Data Fetching Hooks
 * 
 * These hooks provide resilient data fetching with:
 * - Cache-first loading
 * - Background revalidation
 * - Circuit breaker protection
 * - Adaptive polling
 */

export { useBoard, type UseBoardOptions, type UseBoardReturn, type BoardState } from './useBoard';
export { useLeaderboard, type UseLeaderboardOptions, type UseLeaderboardReturn, type LeaderboardState, type Ranker } from './useLeaderboard';
export { useMoveHistory, type UseMoveHistoryOptions, type UseMoveHistoryReturn, type MoveRecord, type LoadedRange } from './useMoveHistory';
export { useResilientQuery, createResilientQueryFactory, type ResilientQueryOptions, type ResilientQueryResult } from './useResilientQuery';
