import {
  OpenSheetMusicDisplay,
  type VexFlowGraphicalNote,
} from "opensheetmusicdisplay";
import {
  type HostMessage,
  type ViewMessage,
  noteId,
  parseHostMessage,
} from "./protocol";

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

const POSITION_COLOR = "#1565c0";
const MARK_COLORS: Record<string, string> = {
  wrong: "#c62828",
  missing: "#ef6c00",
};

const container = document.getElementById("score");
if (container === null) {
  throw new Error("element #score is missing");
}

const osmd = new OpenSheetMusicDisplay(container, {
  autoResize: false,
  drawTitle: false,
});
let notes = new Map<string, VexFlowGraphicalNote[]>();
let highlighted: string[] = [];
const marked = new Map<string, string>();
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

function color(id: string, fill: string | undefined): void {
  for (const note of notes.get(id) ?? []) {
    for (const head of note.getNoteheadSVGs()) {
      // The paths carry their own fill attribute, so styling the group is not enough.
      for (const element of [
        head,
        ...head.querySelectorAll<SVGElement>("path"),
      ]) {
        element.style.fill = fill ?? "";
      }
    }
  }
}

async function load(musicxml: string): Promise<void> {
  loaded = false;
  await osmd.load(musicxml);
  highlighted = [];
  marked.clear();
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
  for (const [id, fill] of marked) {
    color(id, fill);
  }
  for (const id of highlighted) {
    color(id, POSITION_COLOR);
  }
  return ids;
}

function position(noteIds: string[], driverTimeUs: number | undefined): void {
  for (const id of highlighted) {
    color(id, marked.get(id));
  }
  highlighted = noteIds;
  for (const id of highlighted) {
    color(id, POSITION_COLOR);
  }
  noteIds
    .flatMap((id) => notes.get(id) ?? [])[0]
    ?.getSVGGElement()
    ?.scrollIntoView({ block: "center", behavior: "auto" });
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
    case "mark": {
      const fill = MARK_COLORS[message.kind] ?? MARK_COLORS.wrong;
      marked.set(message.note_id, fill);
      color(message.note_id, fill);
      break;
    }
    case "zoom":
      osmd.Zoom = message.percent / 100;
      if (loaded) render();
      break;
    case "reset":
      for (const id of [...highlighted, ...marked.keys()]) {
        color(id, undefined);
      }
      highlighted = [];
      marked.clear();
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
