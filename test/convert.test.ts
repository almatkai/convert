import assert from "node:assert/strict";
import test, { describe, it } from "node:test";
import { HEXtoHSL, HEXtoHSLA, HEXtoRGB, HSLtoHEXA, HSLtoRGBA, RGBtoHEX } from "../src/colorsConversion";
import { checkHslMatch, checkRgbMatch, parseAlpha, parseRgbChannel } from "../src/matching";
import { PXtoTailwindSpacing, REMtoTailwindSpacing } from "../src/spacings";
import { findClosestColor } from "../src/colors";
import { REMtoPX, PXtoREM, PXtoPT, PTtoPX, REMtoPT, PTtoREM } from "../src/spacingsConversion";

describe("color conversions", () => {
  it("HEXtoRGB handles 6-digit and 3-digit hex", () => {
    assert.deepEqual(HEXtoRGB("#006699"), [0, 102, 153]);
    assert.deepEqual(HEXtoRGB("#069"), [0, 102, 153]);
    assert.deepEqual(HEXtoRGB("aabbcc"), [170, 187, 204]);
  });

  it("RGBtoHEX round-trips", () => {
    assert.equal(RGBtoHEX([0, 102, 153]), "#006699");
    assert.equal(RGBtoHEX([255, 255, 255]), "#ffffff");
  });

  it("HEXtoHSL and HSL conversions are consistent", () => {
    const hsl = HEXtoHSL("#ff0080");
    assert.deepEqual(hsl.length, 3);
    // hsl for #ff0080 approx 330,100,50
    assert.ok(hsl[0] >= 320 && hsl[0] <= 340);
  });

  it("HSLtoRGBA with alpha decimal and percent match", () => {
    const a = HSLtoRGBA([10, 10, 10, "0.1"]);
    const b = HSLtoRGBA([10, 10, 10, "10%"]);
    assert.deepEqual(a, b);
    assert.deepEqual(a, [28, 24, 23, 0.1]);
  });

  it("HSLtoHEXA embeds alpha", () => {
    assert.equal(HSLtoHEXA([120, 100, 25, "0.5"]), "#00800080");
    assert.equal(HSLtoHEXA([120, 100, 25, "50%"]), "#00800080");
  });

  it("HEXtoHSLA includes alpha channel", () => {
    const hsla = HEXtoHSLA("#00669980");
    const hsl = HEXtoHSL("#006699");
    assert.deepEqual(hsla.slice(0, 3), hsl);
    assert.ok(Math.abs(hsla[3] - 128 / 255) < 0.001);
  });
});

describe("matching helpers", () => {
  it("parseAlpha handles percent and decimal", () => {
    assert.equal(parseAlpha("0.5"), 0.5);
    assert.equal(parseAlpha("50%"), 0.5);
    assert.equal(parseAlpha("100%"), 1);
  });

  it("parseRgbChannel handles percent and int", () => {
    assert.equal(parseRgbChannel("255"), 255);
    assert.equal(parseRgbChannel("100%"), 255);
    assert.equal(parseRgbChannel("50%"), 128);
  });

  it("checkHslMatch parses comma and space syntax", () => {
    const g1 = checkHslMatch("hsl(120, 50%, 75%)");
    assert.ok(g1);
    assert.equal(g1!.h, "120");
    const g2 = checkHslMatch("hsl(120 50% 75% / 0.5)");
    assert.ok(g2);
    assert.equal(g2!.alpha, "0.5");
    assert.equal(checkHslMatch("rgb(1,2,3)"), null);
  });

  it("checkRgbMatch parses rgb and rgba", () => {
    const r1 = checkRgbMatch("rgb(255, 0, 128)");
    assert.ok(r1);
    assert.equal(r1!.r, "255");
    const r2 = checkRgbMatch("rgba(255 0 128 / 0.5)");
    assert.ok(r2);
    // rgb without alpha still matches
    const r3 = checkRgbMatch("rgb(100% 0% 50%)");
    assert.ok(r3);
    assert.equal(checkRgbMatch("hsl(1,2,3)"), null);
  });
});

describe("tailwind spacings", () => {
  it("PXtoTailwindSpacing and REMtoTailwindSpacing resolve known tokens", () => {
    assert.equal(PXtoTailwindSpacing(16), "4");
    assert.equal(PXtoTailwindSpacing(1), "px");
    assert.equal(PXtoTailwindSpacing(999), "[999px]");
    assert.equal(REMtoTailwindSpacing(1), "4");
    assert.equal(REMtoTailwindSpacing(0.5), "2");
    assert.equal(REMtoTailwindSpacing(99), "[99rem]");
  });
});

describe("closest Tailwind color", () => {
  it("findClosestColor returns nearest Tailwind entry", () => {
    const exact = findClosestColor(0xef - 0, 0x44, 0x44); // ~red-500 #ef4444
    assert.equal(exact.name, "red-500");
    assert.equal(exact.hex, "#ef4444");
    const nearBlack = findClosestColor(0, 0, 0);
    // should be some very dark shade, distance minimal
    assert.ok(nearBlack.hex);
    assert.ok(nearBlack.name.includes("-"));
  });
});

test("spacing conversions use default 16px base", () => {
  assert.equal(REMtoPX(1), 16);
  assert.equal(REMtoPX(2), 32);
  assert.equal(PXtoREM(32), 2);
  assert.equal(PXtoPT(16), 12);
  assert.equal(PTtoPX(12), 16);
  assert.equal(REMtoPT(1), 12);
  assert.equal(PTtoREM(12), 1);
});

test("package.json declares Windows+macOS and Tezbar metadata", async () => {
  const pkg = await import("../package.json", { with: { type: "json" } });
  const data = (pkg as unknown as { default: Record<string, unknown> }).default ??
    (pkg as unknown as Record<string, unknown>);
  assert.deepEqual((data as { platforms: string[] }).platforms, ["macOS", "Windows"]);
  const tezbar = (data as { tezbar: { platforms: string[]; repository: string; upstream: { commit: string; path: string } } }).tezbar;
  assert.deepEqual(tezbar.platforms, ["macos", "windows"]);
  assert.equal(tezbar.repository, "https://github.com/almatkai/convert");
  assert.equal(tezbar.upstream.path, "extensions/convert");
  assert.equal(tezbar.upstream.commit, "6734d85992290066fd31a7f5bee3cca3d3545a1a");
  const commands = (data as { commands: { platforms: string[] }[] }).commands;
  for (const cmd of commands) {
    assert.deepEqual(cmd.platforms, ["macOS", "Windows"]);
  }
});
