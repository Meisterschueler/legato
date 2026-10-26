// Position marker and error marking as note colors (FR-023, FR-025).

export const POSITION_COLOR = "#1565c0";
const MARK_COLORS: Record<string, string> = {
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

/** Which notes carry the position and which a mark, by note ID. */
export class Markers {
  private highlighted: string[] = [];
  private readonly marked = new Map<string, string>();

  constructor(
    private readonly color: (id: string, fill: string | undefined) => void,
  ) {}

  position(ids: string[]): void {
    for (const id of this.highlighted) {
      this.color(id, this.marked.get(id));
    }
    this.highlighted = ids;
    for (const id of ids) {
      this.color(id, POSITION_COLOR);
    }
  }

  mark(id: string, kind: string): void {
    const fill = MARK_COLORS[kind] ?? MARK_COLORS.wrong;
    this.marked.set(id, fill);
    if (!this.highlighted.includes(id)) this.color(id, fill);
  }

  reset(): void {
    for (const id of [...this.highlighted, ...this.marked.keys()]) {
      this.color(id, undefined);
    }
    this.highlighted = [];
    this.marked.clear();
  }

  /** Paints all markers again, e.g. after a new layout or display variant. */
  repaint(): void {
    for (const [id, fill] of this.marked) {
      this.color(id, fill);
    }
    for (const id of this.highlighted) {
      this.color(id, POSITION_COLOR);
    }
  }
}
