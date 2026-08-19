"use strict";

const { test } = require("node:test");
const assert = require("node:assert");
const { selectPackage, binaryName, resolveBinary, exitCode } = require("../lib/binary");

const NAME = "{{superdev:project-slug}}";

test("selectPackage maps supported platforms", () => {
  assert.strictEqual(selectPackage("linux", "x64"), `${NAME}-linux-x64`);
  assert.strictEqual(selectPackage("darwin", "arm64"), `${NAME}-darwin-arm64`);
  assert.strictEqual(selectPackage("win32", "x64"), `${NAME}-win32-x64`);
});

test("selectPackage returns null for unsupported platforms", () => {
  assert.strictEqual(selectPackage("win32", "arm64"), null);
  assert.strictEqual(selectPackage("linux", "riscv64"), null);
});

test("binaryName adds .exe on Windows only", () => {
  assert.strictEqual(binaryName("win32"), `${NAME}.exe`);
  assert.strictEqual(binaryName("linux"), NAME);
});

test("resolveBinary names the missing package when uninstalled", () => {
  assert.throws(
    () =>
      resolveBinary("linux", "x64", () => {
        throw new Error("not found");
      }),
    (err) => err.message.includes(`${NAME}-linux-x64`),
  );
});

test("resolveBinary rejects unsupported platforms with the list", () => {
  assert.throws(
    () => resolveBinary("linux", "riscv64"),
    (err) => err.message.includes("linux-x64"),
  );
});

test("exitCode forwards status and encodes signals", () => {
  assert.strictEqual(exitCode({ status: 3, signal: null }), 3);
  assert.strictEqual(exitCode({ status: null, signal: "SIGINT" }, { SIGINT: 2 }), 130);
  assert.strictEqual(exitCode({ status: null, signal: null }), 1);
});
