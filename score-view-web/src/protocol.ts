// Messages between the shell and score-view-web (contracts/engine-schnittstelle.md#nachrichten-an-score-view-web).

export type HostMessage =
  | { type: "load"; musicxml: string; display_variant?: string }
  | { type: "position"; note_ids: string[]; driver_time_us?: number }
  | { type: "mark"; note_id: string; kind: string }
  | { type: "reset" };

export type ViewMessage =
  | { type: "ready"; note_ids: string[] }
  | { type: "frame_shown"; driver_time_us: number }
  | { type: "frame_stats"; fps: number; max_frame_ms: number }
  | { type: "error"; message: string };

export function parseHostMessage(json: string): HostMessage {
  const value: unknown = JSON.parse(json);
  if (typeof value !== "object" || value === null || !("type" in value)) {
    throw new Error("message without type");
  }
  const message = value as { type: unknown } & Record<string, unknown>;
  switch (message.type) {
    case "load":
      if (typeof message.musicxml !== "string") break;
      return message as HostMessage;
    case "position":
      if (!isStringArray(message.note_ids)) break;
      return message as HostMessage;
    case "mark":
      if (
        typeof message.note_id !== "string" ||
        typeof message.kind !== "string"
      )
        break;
      return message as HostMessage;
    case "reset":
      return { type: "reset" };
  }
  throw new Error(`invalid message: ${String(message.type)}`);
}

function isStringArray(value: unknown): value is string[] {
  return (
    Array.isArray(value) && value.every((item) => typeof item === "string")
  );
}

/**
 * Content-based note id: measure index, staff index, onset within the measure
 * (reduced fraction of a whole note) and MIDI pitch. The engine derives the
 * same id from the MusicXML file.
 */
export function noteId(
  measure: number,
  staff: number,
  onsetNumerator: number,
  onsetDenominator: number,
  midiPitch: number,
): string {
  const divisor = gcd(onsetNumerator, onsetDenominator) || 1;
  return `${measure}/${staff}/${onsetNumerator / divisor}:${onsetDenominator / divisor}/${midiPitch}`;
}

function gcd(a: number, b: number): number {
  return b === 0 ? Math.abs(a) : gcd(b, a % b);
}
