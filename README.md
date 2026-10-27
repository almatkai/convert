# Web Converter for Tezbar

Convert web units and colors interactively in Tezbar on Windows or macOS.

## Command

- **Convert Unit** — type any supported web value (`22px`, `1.5rem`, `12pt`, `#006699`, `#00669980`, `rgb(0 102 153)`, `hsl(200 100% 30%)`, `oklch(0.5 0.1 200)`) and see live conversions to rem/px/pt/hex/rgb/hsl/oklch, the closest Tailwind CSS color, and Tailwind spacing tokens.

Supported conversions: rem↔px↔pt, hex↔rgb↔hsl↔oklch (including alpha/Hexa/Hsla), and Tailwind's spacing scale (`0`–`96`, `px`) with arbitrary-value fallback.

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform `@raycast/api` and pure JS color/math helpers and retains the upstream macOS-compatible design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `convert` (upstream `web-converter`) extension at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author `nielsvanrijn` and contributors remain credited in `package.json`.

Standalone Rust V2: [implementation and validation](v2/README.md).
