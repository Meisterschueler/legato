import { describe, expect, it } from "vitest";
import { noteId, parseHostMessage } from "./protocol";

describe("noteId", () => {
  it("reduces the onset fraction", () => {
    expect(noteId(3, 1, 2, 8, 60)).toBe("3/1/1:4/60");
  });

  it("writes the downbeat as 0:1", () => {
    expect(noteId(0, 0, 0, 4, 48)).toBe("0/0/0:1/48");
  });
});

describe("parseHostMessage", () => {
  it("accepts a position", () => {
    expect(
      parseHostMessage(
        '{"type":"position","note_ids":["0/0/0:1/48"],"driver_time_us":5}',
      ),
    ).toEqual({
      type: "position",
      note_ids: ["0/0/0:1/48"],
      driver_time_us: 5,
    });
  });

  it("rejects a position without note ids", () => {
    expect(() => parseHostMessage('{"type":"position"}')).toThrow(
      "invalid message",
    );
  });

  it("rejects unknown types", () => {
    expect(() => parseHostMessage('{"type":"play"}')).toThrow(
      "invalid message: play",
    );
  });
});
