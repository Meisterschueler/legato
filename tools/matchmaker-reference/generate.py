"""Writes the expectation files for the Rust port (contracts/matchmaker-referenz.md).

Run from this directory: `uv run generate.py`. Copies the example pieces from Matchmaker to
testdata/matchmaker/pieces/ and writes testdata/matchmaker/expected/{piece}.{method}.json.
"""

import json
import urllib.request
from pathlib import Path

import numpy as np
import partitura as pt
from importlib.metadata import distribution
from matchmaker import Matchmaker
from matchmaker.io.midi import MidiStream
from matchmaker.io.queue import RECVQueue
from matchmaker.io.stream import STREAM_END
from matchmaker.utils.symbolic import midi_messages_from_performance

ROOT = Path(__file__).resolve().parents[2]
TESTDATA = ROOT / "testdata" / "matchmaker"
PIECES_DIR = TESTDATA / "pieces"
PIECES = {
    "simple_mozart": "simple_mozart_k265_var1",
    "mozart_k265_var1": "mozart_k265_var1",
    "bach_fugue": "Bach-fugue_bwv_858",
}
METHODS = ["pthmm", "outerhmm", "arzt"]
# Full intermediates only for the small piece; the others are compared by their steps.
INTERMEDIATES_FOR = {"simple_mozart"}


def matchmaker_commit() -> str:
    url = json.loads(distribution("pymatchmaker").read_text("direct_url.json") or "{}")
    return url.get("vcs_info", {}).get("commit_id", "unknown")


def plain(value):
    """numpy values as JSON."""
    if isinstance(value, np.ndarray):
        return plain(value.tolist())
    if isinstance(value, (list, tuple)):
        return [plain(v) for v in value]
    if isinstance(value, np.generic):
        return value.item()
    return value


def note_array(array) -> list:
    return [
        {"onset_beat": float(n["onset_beat"]), "duration_beat": float(n["duration_beat"]),
         "pitch": int(n["pitch"]), "id": str(n["id"])}
        for n in array
    ]


def pitches(vector: np.ndarray) -> list:
    """Active pitches of a binary 88- or 128-key vector."""
    vector = np.asarray(vector).ravel()
    offset = 21 if vector.size == 88 else 0
    return [int(i) + offset for i in np.flatnonzero(vector)]


def observations(mm: Matchmaker) -> list:
    queue = RECVQueue()
    with MidiStream(processor=mm.processor, file_path=mm.performance_file,
                    polling_period=mm.polling_period, return_midi_messages=False,
                    mediator=None, queue=queue):
        pass
    return [item for item in queue.queue if item is not STREAM_END and item is not None]


def intermediates(follower) -> dict:
    result = {}
    transition_model = getattr(follower, "transition_model", None)
    if transition_model is not None:
        # The public properties switch between log and linear values; record the linear ones.
        result["transition_matrix"] = plain(transition_model._transition_prob)
        result["initial_probabilities"] = plain(transition_model._init_dist)
    observation_model = getattr(follower, "observation_model", None)
    if observation_model is not None and hasattr(observation_model, "pitch_profiles"):
        result["pitch_profiles"] = plain(observation_model.pitch_profiles)
    for name in ["alpha", "D1", "D2", "b_table", "transitions", "pitch_error_probs", "other_prob"]:
        if hasattr(follower, name):
            result[name] = plain(getattr(follower, name))
    return result


def state(follower):
    for name in ["forward_variable", "state_probabilities"]:
        if hasattr(follower, name):
            return plain(getattr(follower, name))
    return None


def generate(piece: str, method: str, commit: str) -> dict:
    stem = PIECES[piece]
    mm = Matchmaker(PIECES_DIR / f"{stem}.musicxml", PIECES_DIR / f"{stem}.mid",
                    input_type="midi", method=method)
    follower = mm.score_follower
    full = piece in INTERMEDIATES_FOR
    result = {
        "matchmaker_commit": commit,
        "piece": piece,
        "method": method,
        "parameters": plain({k: v for k, v in mm.config.items() if isinstance(v, (int, float, str, bool))}),
        "polling_period": mm.polling_period,
        "note_array": note_array(mm.score_part.note_array()),
        "score_positions": plain(np.asarray(follower.score_positions, dtype=float)),
    }
    reference = mm.reference_features
    if isinstance(reference, np.ndarray) and reference.dtype.names is None:
        result["reference_features"] = [pitches(row) for row in reference]
    # Raw input of the stream, in Matchmaker's order, for the preprocessor (T036).
    messages, times = midi_messages_from_performance(mm.performance_file)
    result["midi_note_ons"] = [
        [float(t), int(m.note)] for m, t in zip(messages, times)
        if m.type == "note_on" and m.velocity > 0
    ]
    result["midi_end_time"] = float(times.max())
    if full:
        result["intermediates"] = intermediates(follower)
    steps = []
    for features, perf_time in observations(mm):
        position = follower(features, perf_time)
        step = {
            "perf_time": float(perf_time),
            "input": pitches(features),
            "position": None if position is None else float(position),
            "onset_index": int(follower.current_index),
        }
        if full:
            step["state"] = state(follower)
        steps.append(step)
    result["steps"] = steps
    return result


def main() -> None:
    commit = matchmaker_commit()
    expected_dir = TESTDATA / "expected"
    PIECES_DIR.mkdir(parents=True, exist_ok=True)
    expected_dir.mkdir(parents=True, exist_ok=True)
    # The installed package has no assets; take them from the repository at the same commit.
    base = f"https://raw.githubusercontent.com/pymatchmaker/matchmaker/{commit}/matchmaker/assets"
    for stem in PIECES.values():
        for suffix in [".musicxml", ".mid", ".match"]:
            urllib.request.urlretrieve(f"{base}/{stem}{suffix}", PIECES_DIR / f"{stem}{suffix}")

    for piece in PIECES:
        for method in METHODS:
            data = generate(piece, method, commit)
            path = expected_dir / f"{piece}.{method}.json"
            path.write_text(json.dumps(data, separators=(",", ":")) + "\n", encoding="utf-8")
            print(f"{path.relative_to(ROOT)}: {len(data['steps'])} steps")

    hanon = ROOT / "app" / "assets" / "pieces" / "hanon-01.musicxml"
    data = {"partitura_version": distribution("partitura").version,
            "note_array": note_array(pt.load_musicxml(hanon).note_array())}
    (expected_dir / "hanon-01.note_array.json").write_text(json.dumps(data, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
