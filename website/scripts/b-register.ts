import { browser } from 'k6/browser';

export const options = {
	scenarios: {
		browser: {
			executor: 'constant-vus',
			exec: 'browserTest',
			vus: 20,
			duration: '10s',
			options: {
				browser: {
					type: 'chromium'
				}
			}
		}
	}
};

export async function browserTest() {
	const page = await browser.newPage();

	await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });

	const randomUsername = Math.random().toString(36).substring(2, 8);
	const randomPassword = Math.random().toString(36).substring(2, 10);
	console.log('Testing with credentials:', { username: randomUsername, password: randomPassword });
	await page.fill('#username', randomUsername);
	await page.fill('#password', randomPassword);
	await page.click('#join-now');

	await Promise.all([page.waitForNavigation(), page.waitForTimeout(2000)]);
}
