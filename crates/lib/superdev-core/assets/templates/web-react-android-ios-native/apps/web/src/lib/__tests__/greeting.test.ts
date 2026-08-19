import { describe, expect, it } from "vitest";
import { greeting } from "../greeting";

describe("greeting", () => {
  it("greets the world", () => {
    expect(greeting()).toBe("Hello, world");
  });
});
