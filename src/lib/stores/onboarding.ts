/**
 * Onboarding state management (AL1)
 *
 * Tracks which onboarding tips have been shown to avoid repeated display.
 * State is persisted to localStorage.
 */

const ONBOARDING_KEY = 'cahnon-onboarding-v1';

export interface OnboardingState {
	quickOpenTipShown: boolean;
	contextMenuTipShown: boolean;
	shortcutsTipShown: boolean;
	keyboardNavTipShown: boolean;
	/** AV3: Track if welcome dialog was shown for new projects */
	welcomeShown: boolean;
	/** AV4: Track if feature tour has been completed */
	featureTourShown: boolean;
	completed: boolean;
}

const defaultState: OnboardingState = {
	quickOpenTipShown: false,
	contextMenuTipShown: false,
	shortcutsTipShown: false,
	keyboardNavTipShown: false,
	welcomeShown: false,
	featureTourShown: false,
	completed: false,
};

/**
 * Gets the current onboarding state from localStorage.
 */
export function getOnboardingState(): OnboardingState {
	try {
		const stored = localStorage.getItem(ONBOARDING_KEY);
		if (stored) {
			return { ...defaultState, ...JSON.parse(stored) };
		}
	} catch {
		// Ignore parse errors
	}
	return { ...defaultState };
}

/**
 * Marks a specific tip as shown.
 */
export function markTipShown(tip: keyof Omit<OnboardingState, 'completed'>) {
	const state = getOnboardingState();
	state[tip] = true;

	// Check if all tips have been shown
	const allShown =
		state.quickOpenTipShown &&
		state.contextMenuTipShown &&
		state.shortcutsTipShown &&
		state.keyboardNavTipShown &&
		state.welcomeShown &&
		state.featureTourShown;

	if (allShown) {
		state.completed = true;
	}

	try {
		localStorage.setItem(ONBOARDING_KEY, JSON.stringify(state));
	} catch {
		// Ignore localStorage errors
	}
}

/**
 * Resets onboarding state (for testing or user request).
 */
export function resetOnboarding() {
	try {
		localStorage.removeItem(ONBOARDING_KEY);
	} catch {
		// Ignore localStorage errors
	}
}

// =============================================================================
// UA1: First Project Checklist
// =============================================================================

export interface OnboardingStep {
	id: string;
	title: string;
	description: string;
}

export const firstProjectSteps: OnboardingStep[] = [
	{
		id: 'first-chapter',
		title: 'Create your first chapter',
		description: 'Chapters organize your scenes. Click "+ Chapter" in the Outline.',
	},
	{
		id: 'first-scene',
		title: 'Write your first scene',
		description: 'Scenes are the building blocks of your story. Add one to your chapter.',
	},
	{
		id: 'first-character',
		title: 'Add a character to Codex',
		description: 'Create a character in the Bible to track throughout your manuscript.',
	},
	{
		id: 'link-character',
		title: 'Link character to scene',
		description: 'Use the Context Panel to connect your character to a scene.',
	},
];

const CHECKLIST_KEY = 'cahnon-checklist-v1';

export interface ChecklistState {
	[stepId: string]: boolean;
}

export function getChecklistState(): ChecklistState {
	try {
		const stored = localStorage.getItem(CHECKLIST_KEY);
		if (stored) {
			return JSON.parse(stored);
		}
	} catch {
		// Ignore
	}
	return {};
}

export function markChecklistStep(stepId: string) {
	const state = getChecklistState();
	state[stepId] = true;
	try {
		localStorage.setItem(CHECKLIST_KEY, JSON.stringify(state));
	} catch {
		// Ignore
	}
}

export function isChecklistComplete(): boolean {
	const state = getChecklistState();
	return firstProjectSteps.every((step) => state[step.id]);
}
