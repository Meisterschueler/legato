import {
  OpenSheetMusicDisplay,
  type VexFlowGraphicalNote,
} from "opensheetmusicdisplay";
import { Markers, paint } from "./markers";
import {
  type HostMessage,
  type ViewMessage,
  noteId,
  parseHostMessage,
} from "./protocol";
import { Scroller, centerOn, type Viewport } from "./scroll";

declare global {
  interface Window {
    /** Entry point for the shell (runJavaScript). */
    legatoReceive: (json: string) => void;
    /** JavaScript channel of webview_flutter. */
    LegatoHost?: { postMessage: (message: string) => void };
    /** WebView2 (webview_windows). */
    chrome?: { webview?: { postMessage: (message: string) => void } };
  }
}

const container = document.getElementById("score");
if (container === null) {
  throw new Error("element #score is missing");
}

const osmd = new OpenSheetMusicDisplay(container, {
  autoResize: false,
  drawTitle: false,
});
let notes = new Map<string, VexFlowGraphicalNote[]>();
const markers = new Markers((id, fill) =>
  paint(
    (notes.get(id) ?? []).flatMap((note) =>
      note
        .getNoteheadSVGs()
        // The paths carry their own fill attribute, so styling the group is not enough.
        .flatMap((head) => [head, ...head.querySelectorAll("path")]),
    ),
    fill,
  ),
);
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
const viewport: Viewport = {
  scrollY: () => window.scrollY,
  height: () => window.innerHeight,
  maxScrollY: () => document.documentElement.scrollHeight - window.innerHeight,
  scrollTo: (y) => window.scrollTo({ top: y, behavior: "instant" }),
  setOpacity: (opacity) => (container.style.opacity = String(opacity)),
  reducedMotion: () => reducedMotion.matches,
  requestFrame: (callback) => requestAnimationFrame(callback),
};
const scroller = new Scroller(viewport);
let loaded = false;
let renderedWidth = 0;

function send(message: ViewMessage): void {
  const json = JSON.stringify(message);
  if (window.LegatoHost) {
    window.LegatoHost.postMessage(json);
  } else if (window.chrome?.webview) {
    window.chrome.webview.postMessage(json);
  } else {
    window.parent.postMessage(json, "*");
  }
}

// Top and bottom of the system with the note in page coordinates, computed
// like the OSMD cursor (10 px per unit).
function lineOf(note: VexFlowGraphicalNote): [number, number] {
  const system =
    note.parentVoiceEntry.parentStaffEntry.parentMeasure.ParentMusicSystem;
  const first = system.StaffLines[0];
  const last = system.StaffLines[system.StaffLines.length - 1];
  const y = system.PositionAndShape.AbsolutePosition.y;
  const unit = 10 * osmd.Zoom;
  const origin = container!.getBoundingClientRect().top + window.scrollY;
  return [
    origin + (y + first.PositionAndShape.RelativePosition.y) * unit,
    origin +
      (y + last.PositionAndShape.RelativePosition.y + last.StaffHeight) * unit,
  ];
}

// The markers stay, so another display variant keeps the position; the shell
// sends `reset` for another piece.
async function load(musicxml: string): Promise<void> {
  loaded = false;
  await osmd.load(musicxml);
  send({ type: "ready", note_ids: render() });
  loaded = true;
}

// Lays the score out for the current width. Every render replaces the
// graphical notes, so the colors are reapplied by note ID.
function render(): string[] {
  osmd.render();
  renderedWidth = window.innerWidth;
  notes = new Map();
  const ids: string[] = [];
  osmd.Sheet.SourceMeasures.forEach((measure, measureIndex) => {
    for (const vertical of measure.VerticalSourceStaffEntryContainers) {
      for (const staffEntry of vertical.StaffEntries) {
        if (!staffEntry) continue;
        for (const voiceEntry of staffEntry.VoiceEntries) {
          const onset = voiceEntry.Timestamp;
          for (const note of voiceEntry.Notes) {
            if (note.isRest()) continue;
            const graphical = osmd.EngravingRules.GNote(note) as
              VexFlowGraphicalNote | undefined;
            if (!graphical) continue;
            const id = noteId(
              measureIndex,
              staffEntry.ParentStaff.idInMusicSheet,
              onset.WholeValue * onset.Denominator + onset.Numerator,
              onset.Denominator,
              note.halfTone + 12,
            );
            const list = notes.get(id);
            if (list) {
              list.push(graphical);
            } else {
              notes.set(id, [graphical]);
              ids.push(id);
            }
          }
        }
      }
    }
  });
  markers.repaint();
  return ids;
}

function position(noteIds: string[], driverTimeUs: number | undefined): void {
  markers.position(noteIds);
  const note = noteIds.flatMap((id) => notes.get(id) ?? [])[0];
  if (note) scroller.scrollTo(centerOn(...lineOf(note), viewport));
  if (driverTimeUs !== undefined) {
    // The first callback runs before the paint, the second one after it.
    requestAnimationFrame(() =>
      requestAnimationFrame(() =>
        send({ type: "frame_shown", driver_time_us: driverTimeUs }),
      ),
    );
  }
}

function handle(message: HostMessage): void {
  switch (message.type) {
    case "load":
      load(message.musicxml).catch((error: unknown) =>
        send({ type: "error", message: String(error) }),
      );
      break;
    case "position":
      position(message.note_ids, message.driver_time_us);
      break;
    case "mark":
      markers.mark(message.note_id, message.kind);
      break;
    case "zoom":
      osmd.Zoom = message.percent / 100;
      if (loaded) render();
      break;
    case "reset":
      markers.reset();
      break;
  }
}

// Window resizing and Ctrl+wheel zoom both change the width in CSS pixels:
// break the systems anew so the score fills the width.
let resizeTimer: number | undefined;
window.addEventListener("resize", () => {
  window.clearTimeout(resizeTimer);
  resizeTimer = window.setTimeout(() => {
    if (loaded && window.innerWidth !== renderedWidth) render();
  }, 150);
});

window.legatoReceive = (json: string) => {
  try {
    handle(parseHostMessage(json));
  } catch (error) {
    send({ type: "error", message: String(error) });
  }
};

// Frame statistics for the S2 measurement (research.md R7).
let frames = 0;
let maxFrameMs = 0;
let last = performance.now();
let windowStart = last;
function tick(now: number): void {
  frames += 1;
  maxFrameMs = Math.max(maxFrameMs, now - last);
  last = now;
  if (now - windowStart >= 1000) {
    send({
      type: "frame_stats",
      fps: (frames * 1000) / (now - windowStart),
      max_frame_ms: maxFrameMs,
    });
    frames = 0;
    maxFrameMs = 0;
    windowStart = now;
  }
  requestAnimationFrame(tick);
}
requestAnimationFrame(tick);
