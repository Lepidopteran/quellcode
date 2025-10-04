let counter = 0;

export function generateId(prefix: string): string {
	return `${prefix}-${counter++}`;
}
