/**
 * Structured revision workflow configuration.
 *
 * Defines 4 revision passes, each with specific focus areas,
 * visible ContextPanel sections, and relevant issue types.
 */

export type RevisionPassId = 'structure' | 'arcs' | 'consistency' | 'prose';

export interface RevisionPass {
	id: RevisionPassId;
	label: string;
	description: string;
	/** ContextPanel sections to show */
	sections: string[];
	/** Issue types relevant to this pass */
	issueTypes: string[];
	/** ReviewGrid columns to highlight */
	reviewColumns: string[];
}

export const revisionPasses: RevisionPass[] = [
	{
		id: 'structure',
		label: 'Structure',
		description: 'Order, rhythm, and narrative necessity',
		sections: ['word-count', 'template', 'status'],
		issueTypes: ['isolated_scene', 'narrative_pattern'],
		reviewColumns: ['status', 'words', 'tension'],
	},
	{
		id: 'arcs',
		label: 'Arcs',
		description: 'Arc progression and completion',
		sections: ['arcs', 'template'],
		issueTypes: ['arc_inconsistency'],
		reviewColumns: ['arcs', 'tension'],
	},
	{
		id: 'consistency',
		label: 'Consistency',
		description: 'Timeline, knowledge, and continuity',
		sections: ['associations', 'timeline', 'facts', 'issues'],
		issueTypes: ['timeline_conflict', 'broken_setup_payoff', 'orphan'],
		reviewColumns: ['pov', 'status'],
	},
	{
		id: 'prose',
		label: 'Prose',
		description: 'Annotations, TODOs, and final polish',
		sections: ['annotations', 'notes', 'todos', 'word-count'],
		issueTypes: ['tbd_in_done'],
		reviewColumns: ['status', 'words'],
	},
];

export function getRevisionPass(id: RevisionPassId): RevisionPass {
	return revisionPasses.find((p) => p.id === id) || revisionPasses[0];
}
