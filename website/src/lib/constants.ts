import {
	PUBLIC_CHAIN_ID,
	PUBLIC_APPLICATION_ID,
	PUBLIC_PORT,
	PUBLIC_WEBSITE
} from '$env/static/public';

// All configuration is now sourced from environment variables (.env file)
export const chainId = PUBLIC_CHAIN_ID;
export const applicationId = PUBLIC_APPLICATION_ID;
export const port = PUBLIC_PORT;
export const website = PUBLIC_WEBSITE;

console.log('🔧 Using Configuration:', {
	chainId,
	applicationId,
	port,
	website
});
