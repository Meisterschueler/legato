import { OpenSheetMusicDisplay } from "opensheetmusicdisplay";

const container = document.getElementById("score");
if (container === null) {
  throw new Error("element #score is missing");
}

export const osmd = new OpenSheetMusicDisplay(container, { autoResize: true });
