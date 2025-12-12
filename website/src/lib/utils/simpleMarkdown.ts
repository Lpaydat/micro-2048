/**
 * Simple markdown-like formatter for tournament descriptions.
 * Supports: **bold**, *italic*, [links](url), and bullet points (- or •)
 * 
 * Does NOT use innerHTML - returns structured data for safe rendering.
 */

export interface FormattedSegment {
	type: 'text' | 'bold' | 'italic' | 'link' | 'linebreak';
	content: string;
	href?: string; // For links
}

export interface FormattedLine {
	isBullet: boolean;
	segments: FormattedSegment[];
}

/**
 * Parse a single line into formatted segments.
 * Handles: **bold**, *italic*, [text](url)
 */
function parseLineSegments(text: string): FormattedSegment[] {
	const segments: FormattedSegment[] = [];
	
	// Regex to match **bold**, *italic*, or [text](url)
	// Order matters: check ** before * to avoid conflicts
	const pattern = /(\*\*(.+?)\*\*)|(\*(.+?)\*)|(\[([^\]]+)\]\(([^)]+)\))/g;
	
	let lastIndex = 0;
	let match;
	
	while ((match = pattern.exec(text)) !== null) {
		// Add plain text before this match
		if (match.index > lastIndex) {
			segments.push({
				type: 'text',
				content: text.slice(lastIndex, match.index)
			});
		}
		
		if (match[1]) {
			// **bold**
			segments.push({
				type: 'bold',
				content: match[2]
			});
		} else if (match[3]) {
			// *italic*
			segments.push({
				type: 'italic',
				content: match[4]
			});
		} else if (match[5]) {
			// [text](url)
			segments.push({
				type: 'link',
				content: match[6],
				href: match[7]
			});
		}
		
		lastIndex = pattern.lastIndex;
	}
	
	// Add remaining plain text
	if (lastIndex < text.length) {
		segments.push({
			type: 'text',
			content: text.slice(lastIndex)
		});
	}
	
	// If no segments were added, return the whole text as plain
	if (segments.length === 0) {
		segments.push({
			type: 'text',
			content: text
		});
	}
	
	return segments;
}

/**
 * Parse text into formatted lines with bullet point detection.
 */
export function parseSimpleMarkdown(text: string): FormattedLine[] {
	if (!text) return [];
	
	const lines = text.split('\n');
	const result: FormattedLine[] = [];
	
	for (const line of lines) {
		const trimmed = line.trim();
		
		// Check for bullet points (-, •, or *)
		const bulletMatch = trimmed.match(/^[-•*]\s+(.*)$/);
		
		if (bulletMatch) {
			result.push({
				isBullet: true,
				segments: parseLineSegments(bulletMatch[1])
			});
		} else {
			result.push({
				isBullet: false,
				segments: parseLineSegments(trimmed)
			});
		}
	}
	
	return result;
}

/**
 * Check if text contains any markdown formatting.
 * Useful to decide whether to use simple or formatted rendering.
 */
export function hasMarkdownFormatting(text: string): boolean {
	if (!text) return false;
	// Check for **bold**, *italic*, [links](url), or bullet points
	return /\*\*.+?\*\*|\*.+?\*|\[.+?\]\(.+?\)|^[-•*]\s+/m.test(text);
}
