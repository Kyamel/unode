/** An action dispatch callback: receives the lowered action ref `{ t, p? }`. */
export type OnAction = (action: { t: string; p?: Record<string, unknown> }) => void;
