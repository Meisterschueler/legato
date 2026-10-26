import { describe, expect, it } from "vitest";
import { COLOR_TRANSITION_MS, Markers, paint } from "../src/markers";
import { SCROLL_MS, Scroller, centerOn } from "../src/scroll";

function setup() {
  const colors = new Map<string, string | undefined>();
  const markers = new Markers((id, fill) => colors.set(id, fill));
  return { markers, colors };
}

describe("markers", () => {
  it("color correct, wrong and missing notes differently", () => {
    const { markers, colors } = setup();
    markers.mark("a", "correct");
    markers.mark("b", "wrong");
    markers.mark("c", "missing");
    expect(new Set(colors.values()).size).toBe(3);
    expect([...colors.values()]).not.toContain(undefined);
  });

  it("let a later mark replace an earlier one", () => {
    const { markers, colors } = setup();
    markers.mark("b", "wrong");
    const wrong = colors.get("b");
    markers.mark("a", "correct");
    markers.mark("a", "wrong");
    expect(colors.get("a")).toBe(wrong);
  });

  it("keep marks for a new layout or display variant", () => {
    const { markers, colors } = setup();
    markers.mark("a", "correct");
    markers.mark("b", "wrong");
    const before = new Map(colors);
    colors.clear();
    markers.repaint();
    expect(colors).toEqual(before);
  });

  it("clear everything on reset", () => {
    const { markers, colors } = setup();
    markers.mark("a", "correct");
    markers.mark("b", "wrong");
    markers.reset();
    expect([...colors.values()]).toEqual([undefined, undefined]);
  });

  it("fade colors in 150-400 ms instead of setting them at once", () => {
    expect(COLOR_TRANSITION_MS).toBeGreaterThanOrEqual(150);
    expect(COLOR_TRANSITION_MS).toBeLessThanOrEqual(400);
    const element = { style: { fill: "", transition: "" } };
    paint([element], "#c62828");
    expect(element.style).toEqual({
      fill: "#c62828",
      transition: `fill ${COLOR_TRANSITION_MS}ms ease-in-out`,
    });
  });
});

function viewport(reducedMotion = false) {
  const frames: ((now: number) => void)[] = [];
  const view = {
    y: 0,
    opacity: 1,
    scrolls: [] as number[],
    opacities: [] as number[],
    scrollY: () => view.y,
    height: () => 600,
    maxScrollY: () => 2000,
    scrollTo: (y: number) => {
      view.y = y;
      view.scrolls.push(y);
    },
    setOpacity: (opacity: number) => {
      view.opacity = opacity;
      view.opacities.push(opacity);
    },
    reducedMotion: () => reducedMotion,
    requestFrame: (callback: (now: number) => void) => frames.push(callback),
  };
  // Runs the animation frames at 60 Hz until none is requested.
  const run = () => {
    for (let now = 0; frames.length > 0; now += 1000 / 60) {
      frames.shift()!(now);
    }
  };
  return { view, run };
}

describe("scrolling", () => {
  it("centers the current line so the next one stays visible", () => {
    const { view } = viewport();
    expect(centerOn(900, 1100, view)).toBe(700);
    expect(centerOn(100, 300, view)).toBe(0);
    expect(centerOn(2500, 2700, view)).toBe(2000);
  });

  it("moves smoothly in 150-400 ms", () => {
    expect(SCROLL_MS).toBeGreaterThanOrEqual(150);
    expect(SCROLL_MS).toBeLessThanOrEqual(400);
    const { view, run } = viewport();
    new Scroller(view).scrollTo(700);
    run();
    expect(view.scrolls.at(-1)).toBe(700);
    expect(view.scrolls.length).toBeGreaterThanOrEqual(10);
    expect(view.scrolls[0]).toBeLessThan(100);
    for (let i = 1; i < view.scrolls.length; i++) {
      expect(view.scrolls[i]).toBeGreaterThanOrEqual(view.scrolls[i - 1]);
    }
    expect(view.opacities).toEqual([]);
  });

  it("does not move within the same line", () => {
    const { view, run } = viewport();
    new Scroller(view).scrollTo(0);
    run();
    expect(view.scrolls).toEqual([]);
  });

  it("crossfades instead of moving with reduced motion", () => {
    const { view, run } = viewport(true);
    new Scroller(view).scrollTo(700);
    run();
    expect(new Set(view.scrolls)).toEqual(new Set([700]));
    expect(Math.min(...view.opacities)).toBeLessThan(0.1);
    expect(view.opacities[0]).toBeGreaterThan(0.9);
    expect(view.opacity).toBe(1);
  });
});
