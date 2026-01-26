import type { Options } from '@wdio/types';
import { ChildProcess, spawn } from 'child_process';
import * as path from 'path';

let tauriDriver: ChildProcess | null = null;

export const config: Options.Testrunner = {
	runner: 'local',
	autoCompileOpts: {
		autoCompile: true,
		tsNodeOpts: {
			transpileOnly: true,
			project: './tsconfig.json',
		},
	},

	specs: ['./tests/e2e/**/*.spec.ts'],
	exclude: [],

	maxInstances: 1,

	// Connect to tauri-driver on port 4444
	hostname: 'localhost',
	port: 4444,
	path: '/',

	capabilities: [
		{
			// W3C WebDriver capabilities format
			alwaysMatch: {
				'tauri:options': {
					application: path.resolve('./src-tauri/target/release/cahnon'),
				},
			},
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any,
	],

	logLevel: 'info',
	bail: 0,
	waitforTimeout: 10000,
	connectionRetryTimeout: 120000,
	connectionRetryCount: 3,

	framework: 'mocha',
	reporters: ['spec'],

	mochaOpts: {
		ui: 'bdd',
		timeout: 60000,
	},

	// Start tauri-driver before running tests
	onPrepare: function () {
		return new Promise<void>((resolve, reject) => {
			console.log('Starting tauri-driver...');
			tauriDriver = spawn('tauri-driver', [], {
				stdio: ['ignore', 'pipe', 'pipe'],
			});

			tauriDriver.stdout?.on('data', (data) => {
				console.log('[tauri-driver]', data.toString().trim());
			});

			tauriDriver.stderr?.on('data', (data) => {
				console.error('[tauri-driver error]', data.toString().trim());
			});

			tauriDriver.on('error', (err) => {
				console.error('Failed to start tauri-driver:', err);
				reject(err);
			});

			// Give tauri-driver time to start
			setTimeout(() => {
				console.log('tauri-driver should be ready');
				resolve();
			}, 2000);
		});
	},

	// Clean up tauri-driver after tests
	onComplete: function () {
		console.log('Stopping tauri-driver...');
		if (tauriDriver) {
			tauriDriver.kill();
			tauriDriver = null;
		}
	},
};
