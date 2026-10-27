# Validation and review handoff

Review baseline: `feat/native-v2-rust`, HEAD
`92e37e66e3fbeff22fbf9374c97af36a91832cd6`; neither was changed.
Index remains empty. Changes are confined to `v2/` and one root README link.
No host/sibling/profile/SDK working files, registration or ledger were changed.
No staging, commit, push, PR, install, signing, publication or cleanup was performed.

## Executed evidence

| Check | Measured result |
| --- | --- |
| Exact own V1 disposable typecheck | passed |
| Original own V1 tests | 14 passed; no baseline defect |
| Actual own TSX/helpers/data oracle | 17,041 cases; 78,169 source Copy content handoffs |
| Complete pinned V1 registerAction and copyToSystemClipboard | 78,169 actual callbacks, clipboard OS endpoint captured |
| Rust engine comparison with the executed oracle | zero differences across all titles/subtitles/accessories/swatches/copy bytes/order |
| Stable 1.98 and MSRV 1.85 | locked all-target tests, fmt, strict all-target Clippy passed |
| Rust test cases | 3 unit tests + finite corpus test + complete pinned host producer test |
| Actual SDK JSONL process | all 17,041 views; 2,167 captured normal copy calls, 31,187 UTF-8 bytes, plus retry/cancel/timeout cases |
| Process cleanup | 180 open/dispose churn, then 128 simultaneous live sessions, 129th rejected; 12 cancelled opens; EOF/shutdown; reused ID |
| Renderer | 6 complete pinned/proposed modules; 28 deterministic ownership/menu/preference scenarios |
| Actual typed host compiler context | baseline 34 diagnostics; proposal 34; zero novel diagnostics; full host build not performed |
| Exact pinned schema | 4 positive / 41 negative cases, including closed DEV manifest |
| Windows GNU | installed target check and strict all-target Clippy passed |
| Windows GNU link/build | blocked: missing x86_64-w64-mingw32-dlltool; no toolchain installed |
| Windows MSVC / real macOS+Windows GUI acceptance | not-run |
| Fresh ARM64 release | passed, initially absent distinct CargoHome/target; exact HTTPS Git SDK; all fetched SDK Git files bound |
| Closed DEV ZIP | 106 members; member SHA/mode/path/size, closed manifest, extracted real executable and full process oracle passed |

The source corpus uses deterministic seed 6734: 12,000 randomized inputs, every
4096 short HEX value, all 256 grayscale RGB values, every own Tailwind color,
all spacing entries with ±0.000001 boundaries, preference coercion cases and
explicit syntax/Unicode/nonfinite/malformed cases. The remaining explicit cases
bring the total to 17,041. Whitespace, case, scientific-looking inputs, commas,
slashes, negative/zero values, alpha/hue extremes, exponent formatting and
source action-content differences are included. The UTF-16 emoji and NaN
regressions found during comparison were corrected and remain in the corpus.
This is finite executed evidence, not a universal equivalence proof. Float libm
behavior has not been runtime-verified on Windows or every possible input.

Process regressions additionally cover 40 queued searches in reader order,
late ACK after disposal/reopening the same session ID, malformed/nonstring and
oversized input, A/B/A, invalid-to-valid recovery, duplicate/foreign/stale tokens,
unknown closed-shape ACKs, exact-byte retry, timeout, cancellation outside staged
rollback, and selected-row retry action IDs. All host OS effects were substituted;
no real clipboard, window visibility or external converter service was contacted.
Native producer tests compile the **complete** pinned Host::request,
granted/require/string and clipboard Worker::handle functions. Unrelated branches
trap; reachable OS/context boundaries are replaced. The test harness narrowly
allows Clippy 1.85's nonminimal_bool lint in immutable copied host source; product
Clippy remains strict. The pinned visibility producer is inspected and its null
contract is tested; this converter does not request visibility.

## Development artifact

Path: `.work/convert-aarch64-apple-darwin-DEV.zip` (ignored, not a release).

- ZIP SHA256: `2ac1abb780c77245f6483c06573a223cf43a3d04871badb5a78ed72bf9c0acc1`
- Executable SHA256: `cd7ec843f8e9a8a711effde840be69518ac5081be4e29c14752425ce59d04e12`
- Original icon SHA256: `113f0d3d70e149928c4195570c6cfb1b26ca8d5bd378cdf9831b586548a5d15a`

`qa/dev-artifact.json` retains the complete member inventory, compiler version,
source/lock before-and-after binding, protocol-info and hashes of every fetched
SDK Git file. Same input bytes produced the same ZIP twice. No reproducible
compiler-output claim is made across fresh paths. This DEV archive intentionally
has no published artifact URL and is not an installable release reference.

## Audit and remaining prerequisites

All 48 Cargo.lock registry archives, including non-host target packages, were
checksum-verified and their expanded source files compared with the archives.
97 installed, locked npm tool tarballs (91 own V1 tool dependencies and 6 schema
tool dependencies) were SRI/SHA256 checked. JS/TS installed code was compared
with tarball bytes; per-archive member inventories are bound. Authentic license,
NOTICE and AUTHORS files are retained, including r-efi's actual AUTHORS grants.
The product ZIP contains Cargo/SDK/Raycast grants, not npm tools.

Precise npm notice gaps in the published tool tarballs: @raycast/api 1.104.25
contains an MIT declaration but no complete MIT grant/copyright text; filelist
1.0.6 and jake 10.9.4 declare Apache-2.0 without a complete Apache license file
(jake's README contains an abbreviated notice/link). Their grants were not
synthesized from SPDX names. @esbuild/darwin-arm64 0.27.7 and 0.28.2 contain no
standalone license file; exact matching parent esbuild packages supply authentic
MIT LICENSE files, linked by version/hash in `qa/npm-audit.json`. These are QA
package evidence gaps; no additional approval gate or runtime npm dependency
was introduced.

The unapplied generic renderer proposal is required for input-time UI revocation
and coherent late-response handling with the pinned host. See
`integration/README.md` and `integration/host.patch`. Its strict isolated compiler
report includes all 34 baseline diagnostics from missing full host dependency
context; it is not a clean full-host compiler result. Full host integration,
React DOM behavior, OS acceptance, Windows MSVC linking and actual Windows math
remain unverified. No other host extension's compatibility is claimed.

Input >4096 UTF-8 bytes, basePixel serialization >1024 bytes, output title/copy
>8192 bytes, session IDs >256 bytes and native sessions above 128 are explicitly
outside supported bounds. They return errors without truncation. Malformed
outer JSON frames/payloads are handled by the immutable SDK; the client must
revoke before such requests fail. Unknown effect delivery always remains uncertain
until explicit retry; it is not an exactly-once protocol.

XERJ initial indexing covered all 9 source code files, exit 3
completed-with-junk. The final update under ax-d6c17d284711 stopped on the frozen
schema for v2/Cargo.toml; coverage of V2 is incomplete. No new prefix, settings
change, schema bypass or restart was used. Final source checks use complete Git,
rg and hash evidence instead.

`qa/fixture-seal.json` checks all 23 original tracked files (22 byte-exact plus
README's single permitted discovery link), 65 exact Git fixtures, the fresh icon,
source manifest verification states, unchanged HEAD/branch and empty index.
`INVENTORY.json` covers all Git-visible root README/V2 worktree files except
itself. Parent review must independently hash staged blobs before publication.

## Parent review fixes

The parent repeated the full workflow and independently matched 65 immutable Git fixtures and a freshly downloaded original icon. It reproduced stale native revision 1 replacing already accepted revision 2, then fixed a separate monotonic client floor. Seven actual-process regressions cover old/duplicate inputs, unchanged tokens, a newer invalid draft, no resurrection and valid recovery. It also reproduced an old callback mutating a replacement session and fixed pre-dispatch owner/activity guards on search, refresh and action callbacks. Both replacement and unmount are exercised through the actual proposed component. The host patch was regenerated and checked against an owned baseline fixture only. Full stable/MSRV, Windows GNU check/Clippy, original oracle and fresh extracted-production workflow passed again; native OS acceptance remains not-run.
