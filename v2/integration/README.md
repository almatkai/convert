# Unapplied host review proposal

Nothing here was applied to or built in the host checkout. Baseline is exactly
`ea100cee3f69711de12485ba061950c5fc65258f`; original complete files are retained
under `../qa/fixtures/host` with Git-object hashes in `../qa/provenance.json`.
`proposed/` contains complete review versions, not installed files. `host.patch`
is generated against that exact baseline.

The pinned RuntimeView leaves prior Copy actions available after input until the
search promise finishes. Its refresh/action paths do not share the input epoch.
Pinned ListRuntime notifies server search after a 200 ms debounce, and overwrites
local query from arriving root searchText. Its generic color fallback can replace
server-provided results for color-named commands. This converter supplies
`__hasServerSearch: true`; its title is Convert Unit, so the color-name heuristic
is not itself triggered by this command.

The proposal introduces client-owned revisions at the actual input event,
immediate revocation, common session/generation/request fencing, truthful terminal
response handling, a deliberate read-only recovery button, and preserves typed
local drafts. Responses can verify an echoed client revision but cannot create
one. Native model revisions are separate and monotone within a generation.
Repeated value-control callbacks are accepted; each carries the complete current
query and accumulated controlValues with its own clientInputRevision. Supporting
extensions must read that complete draft. Convert Unit has no in-surface value
controls; basePixel is an optional command preference frozen by the SDK per
session, matching own V1 module evaluation. Changing it requires a new session.

Both actual typed wire definitions (`extensionRuntime.ts` and `desktop-api.ts`)
add the optional clientInputRevision. The pinned process router forwards the
complete request body, so it needs no converter-specific branch.

Tests execute complete proposed RuntimeView/ListRuntime, complete pinned
RuntimeView/Surface/PreferencesEditor and preference helpers. They demonstrate
baseline stale actions, exercise A/B/A, malformed input, terminal failures,
late ACKs, session replacement, reusable controls, pending-query/control crosses,
exact selected-row multi-action menu filtering, and raw basePixel strings.
The deterministic React hook scheduler replaces rendering/browser event delivery;
it is not a real React DOM or full OS acceptance test.

Compiler context: actual full modules and both wire definitions, strict TS with
isolated dependency context. Baseline and proposal each have 34 diagnostics;
proposal introduces zero novel diagnostics. Missing host UI/type dependencies
and consequent implicit-any diagnostics are retained in
`../qa/typecheck-diagnostics.json`. This is **not** a passing full host build.
Host integration and regression testing against the complete host application
remain required before use with its GUI. The host working files were never read,
modified, installed or built.
