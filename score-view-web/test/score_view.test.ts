import { describe, expect, it } from "vitest";
import {
  COLOR_TRANSITION_MS,
  Markers,
  POSITION_COLOR,
  paint,
} from "../src/markers";
import { SCROLL_MS, Scroller, centerOn } from "../src/scroll";

function setup() {
  const colors = new Map<string, string | undefined>();
  const markers = new Markers((id, fill) => colors.set(id, fill));
  return { markers, colors };
}

describe("markers", () => {
  it("move the position marker with position", () => {
    const { markers, colors } = setup();
    markers.position(["a", "b"]);
    markers.position(["c"]);
    expect(colors).toEqual(
      new Map([
        ["a", undefined],
        ["b", undefined],
        ["c", POSITION_COLOR],
      ]),
    );
  });

  it("mark notes and show the mark once the position moved on", () => {
    const { markers, colors } = setup();
    markers.position(["a"]);
    markers.mark("a", "missing");
    markers.mark("b", "wrong");
    expect(colors.get("a")).toBe(POSITION_COLOR);
    markers.position(["c"]);
    const missing = colors.get("a");
    const wrong = colors.get("b");
    expect(missing).toBeDefined();
    expect(wrong).toBeDefined();
    expect(new Set([missing, wrong, POSITION_COLOR]).size).toBe(3);
  });

  it("keep position and marks for a new layout or display variant", () => {
    const { markers, colors } = setup();
    markers.position(["a"]);
    markers.mark("b", "wrong");
    const before = new Map(colors);
    colors.clear();
    markers.repaint();
    expect(colors).toEqual(before);
  });

  it("clear everything on reset", () => {
    const { markers, colors } = setup();
    markers.position(["a"]);
    markers.mark("b", "wrong");
    markers.reset();
    expect([...colors.values()]).toEqual([undefined, undefined]);
  });

  it("fade colors in 150-400 ms instead of setting them at once", () => {
    expect(COLOR_TRANSITION_MS).toBeGreaterThanOrEqual(150);
    expect(COLOR_TRANSITION_MS).toBeLessThanOrEqual(400);
    const element = { style: { fill: "", transition: "" } };
    paint([element], POSITION_COLOR);
    expect(element.style).toEqual({
      fill: POSITION_COLOR,
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
