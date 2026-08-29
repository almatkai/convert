const loadBasePixelsFromPreferences = (): number => {
  try {
    // Lazy require so tests can run without Raycast runtime
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const api = require("@raycast/api") as { getPreferenceValues: <T>() => T };
    const { basePixel } = api.getPreferenceValues<{ basePixel?: string }>();
    const parsed = basePixel === "" || basePixel === undefined ? 16 : Number(basePixel);
    if (isNaN(parsed)) {
      return 16;
    }
    return parsed;
  } catch {
    return 16;
  }
};

const BASE_FONT_PIXELS = loadBasePixelsFromPreferences();

export const REMtoPX = (rem: number) => rem * BASE_FONT_PIXELS;

export const REMtoPT = (rem: number): number => rem * 12;

export const PXtoREM = (px: number): number => px / BASE_FONT_PIXELS;

export const PXtoPT = (px: number): number => px * 0.75;

export const PTtoREM = (pt: number): number => pt / 12;

export const PTtoPX = (pt: number): number => pt / 0.75;
