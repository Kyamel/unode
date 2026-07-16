import favicon from '../assets/favicon.svg?raw';

export function GET() {
	return new Response(favicon, {
		headers: {
			'Cache-Control': 'public, max-age=31536000, immutable',
			'Content-Type': 'image/svg+xml',
		},
	});
}
