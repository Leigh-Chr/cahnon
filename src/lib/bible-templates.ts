/**
 * Field templates for bible entry types.
 *
 * When creating a new bible entry, the custom_fields are pre-filled
 * with the template keys (empty values) for the corresponding entry type.
 *
 * @module
 */

export const BIBLE_FIELD_TEMPLATES: Record<
	string,
	Array<{ key: string; label: string; type: 'text' | 'textarea' }>
> = {
	character: [
		{ key: 'age', label: 'Age', type: 'text' },
		{ key: 'appearance', label: 'Appearance', type: 'textarea' },
		{ key: 'traits', label: 'Traits', type: 'textarea' },
		{ key: 'background', label: 'Background', type: 'textarea' },
		{ key: 'goal', label: 'Goal', type: 'textarea' },
		{ key: 'flaw', label: 'Flaw', type: 'textarea' },
	],
	location: [
		{ key: 'location_type', label: 'Type', type: 'text' },
		{ key: 'atmosphere', label: 'Atmosphere', type: 'textarea' },
		{ key: 'significance', label: 'Significance', type: 'textarea' },
	],
	object: [
		{ key: 'origin', label: 'Origin', type: 'textarea' },
		{ key: 'powers', label: 'Powers', type: 'textarea' },
		{ key: 'significance', label: 'Significance', type: 'textarea' },
	],
	faction: [
		{ key: 'goal', label: 'Goal', type: 'textarea' },
		{ key: 'structure', label: 'Structure', type: 'textarea' },
		{ key: 'size', label: 'Size', type: 'text' },
	],
	concept: [
		{ key: 'rules', label: 'Rules', type: 'textarea' },
		{ key: 'limitations', label: 'Limitations', type: 'textarea' },
	],
	glossary: [
		{ key: 'etymology', label: 'Etymology', type: 'text' },
		{ key: 'pronunciation', label: 'Pronunciation', type: 'text' },
	],
};
