// @vitest-environment node
import { expect, it } from "vitest";
import config from "../../electron-builder.config.json";
it("generates stable GitHub update metadata alongside Windows and Linux installers", () => {
 expect(config.publish).toEqual({provider: "github", owner: "alexanderwanyoike", repo: "muzo", releaseType: "draft"});
});
