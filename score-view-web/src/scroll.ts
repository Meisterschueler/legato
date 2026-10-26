// Scrolling that keeps the current line in the middle, so the next line stays
// visible (FR-023): eased with requestAnimationFrame, with "reduce motion" a
// crossfade instead of a movement.

/** Duration of a scroll movement or crossfade (FR-023: 150–400 ms). */
export const SCROLL_MS = 300;

export interface Viewport {
  scrollY(): number;
  /** Height of the visible area and the largest scroll position. */
  height(): number;
  maxScrollY(): number;
  scrollTo(y: number): void;
  setOpacity(opacity: number): void;
  reducedMotion(): boolean;
  requestFrame(callback: (now: number) => void): void;
}

/** Scroll position that centers a line from `top` to `bottom` (page coordinates). */
export function centerOn(top: number, bottom: number, view: Viewport): number {
  const y = (top + bottom) / 2 - view.height() / 2;
  return Math.min(Math.max(y, 0), Math.max(view.maxScrollY(), 0));
}

function ease(t: number): number {
  return t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2;
}

export class Scroller {
  private animation:
    { from: number; to: number; fade: boolean; start?: number } | undefined;

  constructor(private readonly view: Viewport) {}

  scrollTo(target: number): void {
    const to = Math.round(target);
    if (to === (this.animation?.to ?? Math.round(this.view.scrollY()))) return;
    const running = this.animation !== undefined;
    this.animation = {
      from: this.view.scrollY(),
      to,
      fade: this.view.reducedMotion(),
    };
    if (!running) this.view.requestFrame(this.step);
  }

  private readonly step = (now: number): void => {
    const animation = this.animation;
    if (!animation) return;
    animation.start ??= now;
    const t = Math.min((now - animation.start) / SCROLL_MS, 1);
    if (animation.fade) {
      // Fade out, jump halfway, fade in.
      this.view.setOpacity(Math.abs(1 - 2 * t));
      if (t >= 0.5) this.view.scrollTo(animation.to);
    } else {
      const { from, to } = animation;
      this.view.scrollTo(from + (to - from) * ease(t));
    }
    if (t < 1) {
      this.view.requestFrame(this.step);
    } else {
      this.animation = undefined;
    }
  };
}
