# Web Converter — standalone Rust V2

External executable for the own Recommended `almatkai/convert` extension,
`convert` / **Convert Unit**. Schema 3, Rust runtime, process execution, protocol 1.
No JavaScript, Node, beta bridge, host library, network service, storage or shell
process is used by the converter at runtime. Node is used only by QA to execute
immutable original source and renderer functions.

The engine ports the actual own command's rem/px/pt, HEX/HEXA, RGB/RGBA,
HSL/HSLA, OKLCH, nearest Tailwind color and spacing/arbitrary-value behavior.
The source manifest keeps `artifacts: {}`; macOS and Windows verification remain
`not-run`. This is a development review candidate, not an installed or published
release.

## Source and original appearance

Own baseline is `92e37e66e3fbeff22fbf9374c97af36a91832cd6`. Complete original
tracked source, helpers, data, tests, docs, manifest, lock and assets are retained
in `qa/fixtures/v1`; per-file exact Git provenance is in `qa/provenance.json`.
The original Raycast commit is `6734d85992290066fd31a7f5bee3cca3d3545a1a`.
Fresh primary fetches establish **extensions/convert** as the directory; its
package name is **web-converter**, author **nielsvanrijn**. The corresponding
web-converter directory URLs return 404. Evidence is in
`qa/fixtures/raycast/fetch.json`. Own adaptation is by almatkai, with original
contributors yusifaliyevpro, ridemountainpig and cedric25.

The command-owned `assets/icon.png` is byte-identical in own V1, fresh upstream,
V2 and the DEV archive:
`113f0d3d70e149928c4195570c6cfb1b26ca8d5bd378cdf9831b586548a5d15a`.
There are no replacement icons or decorative graphics. Row swatches preserve
source CircleFilled and exact source dynamic color strings with
`adjustContrast: false`, including invalid CSS strings produced by original
malformed inputs. Host-native List/Section/Item nodes carry the same row order,
titles, subtitles, accessories, action labels and clipboard bytes.

## Source behavior intentionally retained

- Search clears all conversion state before parsing. Initial/empty input has no
  result actions; no argument or clipboard read is invented.
- Units use the original ordered, partly unanchored regex alternatives. Only
  `px` is case-sensitive. Prefixes, malformed decimals, negative signs, Unicode
  and multiple units follow the source matches and precedence, rather than a CSS
  parser's interpretation. UTF-16 single-character matching is preserved for
  the unit wildcard. Zero and NaN numeric unit rows are hidden by source truthiness.
- Optional command `basePixel` is evaluated once per session, as V1 evaluates
  its helper at module load: absent/empty/NaN gives 16; whitespace/null/false can
  give zero; negative and infinite values are retained. Tailwind and rem-to-pt
  use the original independent constants. Preferences are changed through host
  settings and take effect in a new session; no extra converter controls exist.
- Source RGB/HSL regex grammar, percentage parsing, unclamped channels/alpha,
  remainder arithmetic, hue behavior and rounding are literal ports. JS decimal
  Number-to-string uses ryu-js; JS Math.round ties and negative zero are explicit.
  The odd five-digit HEX path, malformed OKLCH NaN strings and empty closest-color
  row are preserved. No color-distance library is substituted.
- Tailwind's 220 colors and 35 spacing entries come from own source. JS property
  enumeration order, strict `<` distance updates, MAX_SAFE_INTEGER initial
  threshold, duplicate-color ties, exact spacing equality and arbitrary strings
  are preserved. For example, the `1rem` spacing collision resolves to `4`, not
  `px`. A Tailwind row's displayed token and copied HEX can intentionally differ.

## Ownership, effects and bounds

Copy bytes are frozen in the displayed row token. The token is consumed before
any await, outside the SDK's staged JSON rollback. Duplicate, foreign and stale
IDs have no effect. A successful copy creates fresh tokens for deliberate repeats.
Unknown/malformed ACK, error, cancellation or five-second timeout produces a
truthful uncertainty state with one **Retry Copy Exact Value** action on the
retained selected row. Retry never reads/reconverts the input. Delivery is never
automatically replayed and no exactly-once guarantee is made.

The actual pinned clipboard producer returns the closed `{ok:true,message?:string}`
shape. Its complete route/worker functions are executed in tests with only reachable
OS boundaries replaced. Visibility's null ACK is also tested. Own V1's actual
CopyToClipboard handler stays open, so this extension makes no visibility request
and only grants `clipboard.write`. Copy metadata is `kind: action` to avoid a
renderer fallback announcing success without a validated ACK.

Stale and duplicate client revisions are inert. New input revokes old rows and
tokens before checking searchText type or size; even an invalid newer draft
raises the separate monotonic client revision floor.
Client-owned input revisions are independent of native model revisions. Full
input-time UI ownership requires the separate **unapplied** generic host proposal
in `integration/`; the pinned host alone leaves stale Copy actions visible.
Top-level invalid JSON/non-object wire payloads are rejected earlier by the
immutable SDK; frontend ownership must revoke them locally too. A native reply
cannot advance the client revision. Preference/session replacement is fenced by
both host and native generations.

Explicit bounds: 4096 UTF-8 input bytes, 1024 serialized basePixel bytes, 8192
bytes per converted title/copy, 256 session-ID bytes and 128 native sessions.
Bounds yield explicit errors; values are not truncated. The pinned SDK also caps
frames at 16 MiB, pending reverse calls at 128, active handlers at 128 (120 ordinary),
output frames at 256, and scoped tasks at 64. This extension owns one cleanup task
per session and no persistent files. Scope guards remove the exact session model
on failed/cancelled opens, disposal, EOF or shutdown; old cleanup cannot remove a
replacement generation. No storage migration or invented persistence exists.

## Reproduce

Installed stable and Rust 1.85 are used; no toolchain or platform packages are
installed. Use Python 3.11+ (this machine has `/opt/homebrew/bin/python3`).

```sh
/opt/homebrew/bin/python3 v2/qa/run.py
/opt/homebrew/bin/python3 v2/qa/run.py --fresh-release
/opt/homebrew/bin/python3 v2/qa/seal.py
/opt/homebrew/bin/python3 v2/qa/inventory.py
/opt/homebrew/bin/python3 v2/qa/inventory.py --check
```

QA refreshes measured report files; regenerate the worktree inventory after reruns.
Fresh builds may produce different executable/archive hashes across fresh paths.
The runner keeps HOME/XDG/temp/npm/Cargo homes/targets under ignored `.work`.
It copies exact V1 source plus lock into a disposable oracle, installs locked QA
packages with scripts disabled, and checks the recomputed oracle byte-for-byte.
Dependency fetching is a setup/build activity; process tests replace OS delivery
and use no user profile or public converter service.

`--fresh-release` creates distinct initially absent CargoHome and target paths,
fetches SDK 0.1.1 through immutable HTTPS Git at
`e955911f0bf5b558f9d3eb5183df0adc95c22af2`, binds product sources/lock before and
after, compares every fetched SDK Git file with its commit object, builds real
ARM64 release code, closes the deterministic DEV ZIP inventory and runs the full
process oracle against its extracted executable. It does not install, sign,
publish or release anything. Compiler output is not claimed reproducible across
fresh build paths.

See [VALIDATION.md](VALIDATION.md), `qa/dev-artifact.json`, both archive audits,
and `INVENTORY.json` for measured evidence and remaining gaps. Inventory hashes
are **worktree bytes**; the parent must independently rehash staged blobs.
