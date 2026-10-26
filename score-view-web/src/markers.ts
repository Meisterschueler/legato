// Note colors of the marking (FR-025); the position is a bar over the system (main.ts).

/** Semi-transparent blue of the position bar. */
export const POSITION_COLOR = "rgba(21, 101, 192, 0.3)";
const MARK_COLORS: Record<string, string> = {
  correct: "#2e7d32",
  wrong: "#c62828",
  missing: "#ef6c00",
};

/** Duration of every color change (FR-023: 150–400 ms, never set at once). */
export const COLOR_TRANSITION_MS = 250;

/** Fades the elements to `fill`, or back to black without one. */
export function paint(
  elements: Iterable<{ style: { fill: string; transition: string } }>,
  fill: string | undefined,
): void {
  for (const element of elements) {
    element.style.transition = `fill ${COLOR_TRANSITION_MS}ms ease-in-out`;
    element.style.fill = fill ?? "";
  }
}

/** Which notes carry a mark, by note ID. */
export class Markers {
  private readonly marked = new Map<string, string>();

  constructor(
    private readonly color: (id: string, fill: string | undefined) => void,
  ) {}

  mark(id: string, kind: string): void {
    const fill = MARK_COLORS[kind] ?? MARK_COLORS.wrong;
    this.marked.set(id, fill);
    this.color(id, fill);
  }

  reset(): void {
    for (const id of this.marked.keys()) {
      this.color(id, undefined);
    }
    this.marked.clear();
  }

  /** Paints all marks again, e.g. after a new layout or display variant. */
  repaint(): void {
    for (const [id, fill] of this.marked) {
      this.color(id, fill);
    }
  }
}
