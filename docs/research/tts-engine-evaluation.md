# TTS engine evaluation: Edge TTS vs VoiceStudio vs Kokoro

Date: 2026-10-24
Question: could [VoiceStudio](https://github.com/debpalash/VoiceStudio) or
[kokoro-onnx](https://github.com/thewh1teagle/kokoro-onnx) replace Edge TTS as
Clip2Pod's narrate engine?

**Verdict: neither replaces Edge TTS. Kokoro is a viable *second* engine (offline
fallback); VoiceStudio is a different product, not a component.**

---

## 1. What Clip2Pod actually needs from a TTS engine

Measured from the current integration, not assumed:

| Requirement | Where it comes from |
|---|---|
| Rust-callable, in-process | `clip2pod-core` is a Rust crate; `msedge-tts` is a normal dependency ([`Cargo.toml`](../../src-tauri/crates/clip2pod-core/Cargo.toml)) |
| MP3 bytes out | `id3` tagging + `mp3-duration` run on the output; feed server serves `/tts/*` MP3 |
| Long-article chunking | `tts.rs:340` test `synthesizes_long_multichunk_article` |
| Cancellable + progress callback | `tts::synthesize_to_file_cb(..., &cancel, |done, total|)` called from `worker.rs:71` |
| Voice list at runtime, filterable, cyclable | `tts::fetch_voices` → `voices::filter_narration_voices` → `voices::pick_voice` (gender-aware cycling) |
| Zero external dependencies | Release notes promise: "Narrate mode has no extra dependencies" ([`release.yml:69`](../../.github/workflows/release.yml)) |
| AGPL-3.0-compatible licence | `clip2pod-core` is AGPL-3.0-only |

The replaceable surface is small — **5 call sites, 2 real functions**
(`fetch_voices`, `synthesize_bytes`/`synthesize_to_file_cb`). Swapping or adding an
engine behind a trait is cheap; everything expensive is downstream of the audio
format and the shipping story.

---

## 2. Candidate 1 — VoiceStudio

**What it is:** a self-hosted "fully-local ElevenLabs alternative" — voice cloning,
voice design, dubbing, dictation, transcription, *audiobook creation* — as an
Electron desktop app plus a Python backend.
([README](https://github.com/debpalash/VoiceStudio))

Facts (GitHub API, 2026-10-24):

- AGPL-3.0, Python-primary polyglot monorepo (Python + TS/Bun + Electron + some Rust)
- 84,698 stars / 9,364 forks, 4,915 commits — but **created 2026-04-09**, i.e. ~6 months old
- Repo size 143 MB; CPU install pulls a slim PyTorch, "roughly 5 GB of disk"
- Tauri shell was **abandoned** at v0.5.3 in favour of Electron
- Engine catalogue of ~18 TTS backends, each with its own `pip install` / clone /
  env-var gate; several require licence acceptance, and audio.cpp's Breeze-TTS-2
  weights are flagged "research/non-commercial"
  ([`docs/engines/README.md`](https://github.com/debpalash/VoiceStudio/blob/main/docs/engines/README.md))
- Local HTTP API exists (`GET /engines`, `/generate`; ASR side documented on port
  3900, OpenAI-compatible for transcription)
  ([`docs/speech-platform.md`](https://github.com/debpalash/VoiceStudio/blob/main/docs/speech-platform.md))

**Why it does not fit:**

1. **Not a library.** There is no crate, no embeddable engine — only an app with an
   HTTP surface. Integrating means Clip2Pod shells out to, or requires the user to
   install and run, a ~5 GB separate application. That is strictly worse than
   today's zero-dependency narrate path, and worse than rip mode's
   "needs yt-dlp + ffmpeg on PATH" compromise.
2. **Scope collision.** VoiceStudio already does audiobook creation and TTS export.
   Depending on it makes Clip2Pod a thin client of a superset product.
3. **Python + PyTorch + GPU** in a Tauri app whose whole value is being a small
   desktop binary.
4. **Immaturity relative to the hype.** 84.7k stars in 6 months with 156 open
   issues and a desktop-shell rewrite mid-flight is not a stable dependency.
5. **Licence hygiene.** AGPL-3.0 app licence is compatible, but the *model weights*
   are "governed separately" and at least one bundled engine's weights are
   non-commercial. Clip2Pod would inherit a licence-audit burden per engine.

**Residual value:** treat its engine catalogue as a shopping list. It is how you
find out that Kokoro, KittenTTS, Sherpa-ONNX and Supertonic-3 are the CPU-viable
local options.

---

## 3. Candidate 2 — Kokoro (kokoro-onnx, and the Rust path)

Kokoro-82M is an 82M-parameter open-weights TTS model, Apache-2.0.

### 3a. `kokoro-onnx` as linked — Python, so: a sidecar

[thewh1teagle/kokoro-onnx](https://github.com/thewh1teagle/kokoro-onnx): MIT
library, Apache-2.0 model, Python 3.10–3.13, onnxruntime ≥1.20.1, 2,808 stars, last
push 2026-09-01. ~300 MB model, ~80 MB quantized. Python API only — no CLI, no Rust
bindings.

Using *this* package means bundling a Python runtime or requiring one on PATH.
That breaks the zero-dependency promise for the narrate path. **Not the way in.**

### 3b. The actual way in — Rust crates

A Rust/`ort` Kokoro ecosystem exists. The most complete is
[`kokoro-en`](https://lib.rs/crates/kokoro-en) (repo `pguso/kokoro`):

- **Apache-2.0**, bin+lib, Rust 2024, v0.1.5 (Aug 4 2026)
- `ort` 2.0.0-rc.11, `ndarray`, `tokio`; **async** API, **streaming** mode
  (`tts.stream("af_alloy")` → `futures::Stream`) and one-shot `tts.synth(text, voice)`
- Builds on macOS/Linux/Windows, cross-compiles to Android/iOS; provider
  auto-select CoreML / CUDA / DirectML / CPU, overridable via `KOKORO_ORT_PROVIDER`
- Output 24 kHz samples; `with_speed(1.2)` for rate
- Pronunciation overrides at runtime via `KOKORO_G2P_LEXICON` (TAB word→IPA)
- **Weights not bundled**: fetch from `onnx-community/Kokoro-82M-v1.0-ONNX` —
  `model.onnx` ~325 MB fp32, `model_q8f16.onnx` ~160 MB, `model_quantized.onnx` ~92 MB int8

Alternatives if `kokoro-en` proves unmaintained:
[Kokoros](https://github.com/lucasjinreal/Kokoros) (CLI `koko`, built-in
phonemizer), [`kokoroxide`](https://crates.io/crates/kokoroxide) (MIT/Apache-2.0),
[`tts-rs`](https://github.com/rishiskhare/tts-rs) (26 voices, 9 languages,
streaming, CPU-only),
[`loqui-kokoro`](https://docs.rs/loqui-kokoro/latest/loqui_kokoro/) (chunks to
≤510 phonemes like Kokoro's own KPipeline, **GPL-free** Misaki G2P port).
[`kokoro-ort`](https://docs.rs/kokoro-ort/latest/kokoro_ort/) is a reserved
scaffold with no synthesis code — ignore it.

### 3c. Real costs, in order of how much they hurt

1. **No MP3.** Kokoro emits PCM/f32 at 24 kHz. Clip2Pod's pipeline is MP3 end to
   end (`id3`, `mp3-duration`, `/tts/*`). ffmpeg is *not* bundled, so this means
   adding an in-process encoder (`mp3lame-encoder`, LGPL-2.1 — compatible) and a
   WAV→MP3 step. Non-trivial new code in the hot path.
2. **Installer weight.** +92 MB (int8) to +325 MB (fp32) of weights, either shipped
   in the installer or downloaded on first run with progress UI, retry, and
   checksum. Either way, "no extra dependencies" becomes "a one-time model download".
3. **Licensing boundary.** `kokoro-en`'s default `g2p-espeak` feature compiles
   bundled **espeak-ng (GPL-3.0)** via CMake. AGPL-3.0 + GPL-3.0 is already
   Clip2Pod's established pattern (per `CLAUDE.md`), but today that GPL linkage is
   confined to `clip2pod-rip`. Pulling GPL into `clip2pod-core` erases the current
   clean separation. Mitigation: `--no-default-features` / `misaki-lean`, or
   `loqui-kokoro`'s GPL-free G2P. Run this past `license-boundary-reviewer`.
4. **CI build complexity.** `g2p-espeak` needs CMake + a C/C++ toolchain on all
   three platforms, and first build "may take several minutes". Clip2Pod already
   fights cross-compilation on arm64 macOS runners (the `openssl-sys` vendored
   workaround in `clip2pod-core/Cargo.toml`); espeak-ng via CMake plus `ort` binary
   provisioning is more of the same, on a bigger surface.
5. **Voice inventory collapses.** Edge TTS exposes hundreds of neural voices across
   ~140 locales, fetched at runtime — which is exactly what
   `filter_narration_voices` / `pick_voice` gender-aware cycling is built around.
   Kokoro v1.0 ships on the order of **50-odd voices, English-dominant**, as a
   static list. `fetch_voices` becomes a constant, and multi-locale narration
   effectively goes away.
6. **Quality.** 82M params on CPU is impressive for its size and genuinely good for
   English narration, but it is below Azure neural voices — audibly so on
   non-English text, numbers, abbreviations, and proper nouns. No SSML.
7. **Maintenance risk.** `kokoro-en` is ~599 downloads/month, one maintainer,
   v0.1.x, weekly downloads tailing to single digits by 2026-10-09. Plan to vendor
   or be ready to swap.

### 3d. What Kokoro buys

- **Offline.** No network in the narrate path.
- **No breakage risk from Microsoft.** This is the real prize — see below.
- **Privacy.** Article text never leaves the machine.
- **No rate limits**, no throttling on long queues.
- **Clean licence** on the model (Apache-2.0) and unambiguous permission to use it,
  including commercially.

---

## 4. Status quo — Edge TTS

What it gets right today, and why it is hard to beat:

- **Zero install, zero disk, zero CPU.** No weights, no ONNX runtime, no toolchain.
- **MP3 natively**, which is what the whole downstream pipeline wants.
- **Hundreds of high-quality neural voices across ~140 locales**, enumerable at
  runtime — the feature `voices.rs` is designed around.
- **One small Rust dependency** (`msedge-tts` 0.2), already integrated, already
  tested against long multi-chunk articles.
- Free.

The liabilities, stated plainly:

- **It is an undocumented, unsanctioned endpoint.** Microsoft's Read Aloud service
  is not a public API. There is no SLA, no terms granting this use, and no
  commercial licence.
- **It breaks.** This is not hypothetical — the last release before this evaluation
  was `ce7938e fix(tts): send Sec-MS-GEC v2 token so Edge TTS stops returning 403`.
  Every such break is a user-visible outage fixed only by shipping a new build, and
  the fix lives in a third-party crate whose maintainer you don't control.
- Requires internet; subject to undisclosed rate limits.
- Article text is sent to Microsoft.

---

## 5. Comparison

| | **Edge TTS** (status quo) | **Kokoro** (via `kokoro-en`) | **VoiceStudio** |
|---|---|---|---|
| Integration shape | Rust crate, in-process | Rust crate, in-process | separate ~5 GB app over HTTP |
| Language/runtime | Rust | Rust + ONNX Runtime | Python + PyTorch + Electron |
| Offline | ✗ | ✓ | ✓ |
| Disk cost | 0 | 92–325 MB weights | ~5 GB |
| Output format | **MP3** (pipeline-native) | 24 kHz PCM → needs encoder | WAV/PCM over HTTP |
| Voices | hundreds, ~140 locales, runtime-fetched | ~50, English-dominant, static | varies by engine; cloning |
| Quality | best of the three | good, below Azure neural | potentially best (cloning), engine-dependent |
| Streaming/progress | ✓ (already wired) | ✓ | HTTP, would need wiring |
| Licence (code) | crate MIT-ish; **service unsanctioned** | Apache-2.0 (+ GPL espeak-ng unless `misaki-lean`) | AGPL-3.0, weights "governed separately" |
| Licence (use) | **no grant — gray area** | Apache-2.0 model, clean | per-engine; some non-commercial |
| Breakage risk | **high — MS can kill it any day** | low (pinned local model) | medium (6-month-old project, rewrites) |
| CI/build cost | none | CMake + C toolchain + `ort` on 3 platforms | n/a |
| Effort to adopt | 0 (done) | ~medium: engine trait + MP3 encode + model provisioning + voice-list rework | high, and architecturally wrong |

---

## 6. Recommendation

**Keep Edge TTS as the default. Add Kokoro as an optional offline engine, not a
replacement. Ignore VoiceStudio.**

Reasoning: Edge TTS wins on every axis that users feel — quality, voice count,
install size, zero setup — and loses on exactly one: it can be switched off by
Microsoft without notice, and once already has been. That is a *continuity* risk,
and the fix for a continuity risk is a fallback, not a migration. Replacing Edge
TTS with Kokoro would trade a 470-voice, 140-locale, zero-install, better-sounding
engine for a ~50-voice English one plus 300 MB and a new MP3 encoding path. Nobody
upgrades into that.

Sequencing, cheapest first:

1. **Do nothing yet.** The Sec-MS-GEC fix is shipped and Edge TTS works. Build the
   fallback when there is a second outage, or when someone actually asks for offline.
2. **When you do build it:** introduce a minimal engine abstraction over the two
   real functions (`fetch_voices`, `synthesize_to_file_cb`) — those 5 call sites are
   the entire blast radius. One `TtsEngine` trait, two impls. No plugin system.
3. Settle the MP3 question before the model question: `mp3lame-encoder` in-process.
   If that doesn't work cleanly, the rest is moot.
4. Prefer the `misaki-lean` / `loqui-kokoro` G2P path to keep GPL out of
   `clip2pod-core`, and have `license-boundary-reviewer` check it.
5. Ship the int8 model (~92 MB) as a first-run download, not in the installer.

An ADR is warranted if step 2 is ever started — the engine abstraction and the
GPL-in-core question are both architectural.

---

## Sources

- [debpalash/VoiceStudio](https://github.com/debpalash/VoiceStudio) — README, [`docs/engines/README.md`](https://github.com/debpalash/VoiceStudio/blob/main/docs/engines/README.md), [`docs/speech-platform.md`](https://github.com/debpalash/VoiceStudio/blob/main/docs/speech-platform.md); repo metadata via `gh api repos/debpalash/VoiceStudio` (2026-10-24)
- [thewh1teagle/kokoro-onnx](https://github.com/thewh1teagle/kokoro-onnx) — README; metadata via `gh api` (2026-10-24)
- [`kokoro-en` on lib.rs](https://lib.rs/crates/kokoro-en) — crate metadata, features, API
- [Kokoros](https://github.com/lucasjinreal/Kokoros), [`kokoroxide`](https://crates.io/crates/kokoroxide), [`tts-rs`](https://github.com/rishiskhare/tts-rs), [`loqui-kokoro`](https://docs.rs/loqui-kokoro/latest/loqui_kokoro/), [`kokoro-ort`](https://docs.rs/kokoro-ort/latest/kokoro_ort/), [`crustytts-phonemize`](https://docs.rs/crustytts-phonemize/latest/crustytts_phonemize/)
- Clip2Pod: `src-tauri/crates/clip2pod-core/src/tts.rs`, `voices.rs`, `Cargo.toml`; `src-tauri/src/commands.rs`, `worker.rs`; `.github/workflows/release.yml`; commit `ce7938e`
