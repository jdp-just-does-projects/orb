# orb — Plan

## Context

orb is a new, local-first photo and video manager for Windows, Linux, and macOS. The directory `/root/code/personal/orb` is empty; this is a greenfield design. The goal is a fast, simple app that can preview, sort, and search media in virtually any format, driven by two critical capabilities:

1. **Metadata-driven organisation**: read EXIF/IPTC/XMP/QuickTime/Matroska tags and use them to tag, sort, and group.
2. **Content-driven search**: local embeddings (visual, face, speech, sound) stored locally, computed locally on CPU with optional GPU.

Decisions already made with the user:

| Decision | Choice |
|---|---|
| Stack | Rust core + Tauri 2, SolidJS frontend |
| Scale target | 100k–1M files (needs HNSW ANN, not brute force) |
| Writes to disk | Read-only toward originals; tags/ratings/people written to `.xmp` sidecars |
| GPU backends | CUDA, CoreML/Metal, DirectML, Vulkan/ROCm (CPU always works) |
| Licensing | Possibly commercial later, so permissive-licence models by default |
| First usable release must include | text-to-image search, face clustering/naming, video speech transcripts, non-speech audio tags |
| RAW | Embedded JPEG preview only; full demosaic later |

Dev box today: Linux x86_64, 3 cores, 3.7 GB RAM, no GPU, Node 18, Python 3.12, no rustc/ffmpeg. Phase 0 bootstraps the toolchain and the plan sizes memory use accordingly.

## Building blocks (verified Oct 2026)

| Concern | Choice | Why |
|---|---|---|
| ONNX inference | `ort` 2.0 (rc), feature `load-dynamic` | Exposes CUDA/TensorRT/DirectML/CoreML/ROCm EPs with per-op CPU fallback. Loading `libonnxruntime` dynamically lets us ship a small CPU "runtime pack" and download CUDA (~700 MB) / DirectML packs on demand instead of bloating the installer. |
| Speech | `whisper-rs` 0.16 (whisper.cpp) | GPU is a compile-time feature (`cuda`, `metal`, `vulkan`, `hipblas`, `coreml`), so GPU whisper ships as a per-backend `orb-whisper` sidecar binary. Default model `base` (~400 MB RAM), `small` when RAM > 8 GB. |
| ANN index | `usearch` 2.x | HNSW with `save/load/view` (mmap), `filtered_search`, f16/i8 storage. `sqlite-vec` is brute-force only; LanceDB duplicates SQLite's role. |
| Metadata store | `rusqlite` (bundled, WAL, FTS5) | Source of truth; usearch files are rebuildable caches. |
| Metadata read | `nom-exif` 3.x + `ffprobe` JSON | Pure-Rust EXIF for JPEG/TIFF/HEIC/RAW plus QuickTime/MKV tags; ffprobe fills gaps (IPTC, codec detail, odd containers). No bundled ExifTool (Perl packaging pain). |
| XMP write | `xmp_toolkit` 1.7+ | Only mature Rust path that writes namespaced sidecars (`dc`, `xmp`, `mwg-rs`, `Iptc4xmpExt`). Needs Rust 1.88+. |
| Image decode | `image` 0.25 hub, `zune-jpeg`, `libheif-rs` (`embedded-libheif`), `jxl-oxide`, `rawler` (pure-Rust RAW, embedded previews) | ffmpeg sidecar is the universal fallback for AVIF and anything that fails. |
| Video | `ffmpeg-sidecar` 2.x spawning bundled LGPL ffmpeg/ffprobe (Tauri `externalBin`) | Avoids FFI build pain on Windows and keeps licensing clean (decode only, no GPL codecs). |
| Visual embeddings | Default `Xenova/clip-vit-base-patch32` (MIT, int8/q4f16, 512-d). Quality tiers: `immich-app/ViT-B-16-SigLIP2__webli`, `ViT-SO400M-16-SigLIP2-384__webli` (Apache-2.0) | Split vision/text ONNX towers available; CPU-friendly quantised variants. |
| Faces | YuNet detector (MIT) + SFace embeddings (Apache-2.0) default; insightface `buffalo_l` as opt-in behind a non-commercial licence dialog | Keeps the default distribution commercially clean. |
| Audio tags | `audiomagic/yamnet-onnx` (Apache-2.0, 521 AudioSet classes) | Covers laughter/music/dog/waves etc. CLAP (`Xenova/clap-htsat-unfused`) later for open-vocabulary audio search. |
| Tokenizer | `tokenizers` 0.22, `default-features = false` | CLIP BPE and SigLIP SentencePiece from `tokenizer.json` shipped with each text tower. |
| Frontend | SolidJS + TS + Vite, `@tanstack/solid-virtual`, custom justified-rows layout | Fine-grained reactivity with no VDOM suits a 100k-cell grid; Svelte 5 is the fallback if preferred. |
| Tauri IPC | `tauri::ipc::Channel<T>` for progress; `register_asynchronous_uri_scheme_protocol("orb-thumb")` for thumbnails; `tauri-plugin-updater` | Ordered progress streams; thumbnails served as bytes with ETag/immutable caching, not base64 over IPC. |

## Architecture

### Workspace layout

```
Cargo.toml (workspace)          package.json (pnpm workspace)
crates/
  orb-core    ids, MediaKind, errors, config, paths, tracing
  orb-db      rusqlite + migrations (refinery), repositories
  orb-meta    nom-exif + ffprobe -> normalised Metadata; xmp_toolkit sidecar read/write
  orb-media   decode (image/zune/libheif/jxl/rawler), orientation, resize (fast_image_resize), WebP encode, ffmpeg wrappers
  orb-ml      ort sessions, device probe/benchmark, model registry + downloader, preprocessing, CLIP/SigLIP, YuNet/SFace, YAMNet, whisper
  orb-index   usearch wrappers (one index per embedding space), roaring filters, hybrid search + RRF fusion
  orb-jobs    SQLite-backed job queue, stage worker pools, notify watcher, memory governor
  orb-cli     headless `orb scan|index|search|bench` for CI and benchmarks
apps/desktop/
  src-tauri   Tauri 2 app: commands, channels, orb-thumb protocol, updater, sidecar config
  src         SolidJS UI
fixtures/     ~200-file CC0 corpus, one per format, plus synthetic generators
```

Dependency direction: `core <- db <- meta/media <- ml <- index <- jobs <- tauri`. Heavy native deps (`ort`, `whisper-rs`, `xmp_toolkit`, `libheif`) sit behind cargo features so `orb-cli` and unit tests build fast.

### SQLite schema (source of truth)

- `roots(id, path, enabled, last_scan_at)`
- `files(id, root_id, rel_path, size, mtime_ns, quick_hash, content_hash?, asset_id, status[present|missing|error], seen_gen)` UNIQUE(root_id, rel_path)
- `assets(id, kind[image|video|raw|live], quick_hash, content_hash?, width, height, duration_ms, taken_at, taken_at_local, tz_offset, gps_lat, gps_lon, camera_make, camera_model, lens, iso, f_number, exposure, focal_mm, codec, fps, rating, orientation, meta_json, meta_version)` with indexes on `taken_at`, `(camera_make, camera_model)`, `(gps_lat, gps_lon)`, `duration_ms`. Typed columns are what we sort/filter on; `meta_json` is the full tag dump.
- `tags(id, name, parent_id)`, `asset_tags(asset_id, tag_id, source[user|xmp|auto], confidence)`
- `albums(id, name, kind[manual|smart], query_json)`, `album_assets(album_id, asset_id, pos)`
- `segments(id, asset_id, t_start_ms, t_end_ms, kind[keyframe|speech|sound], thumb_key)` for video time ranges
- `faces(id, asset_id, segment_id?, bbox, landmarks_json, quality, embedding_id, person_id?, cluster_id?, confirmed)`, `persons(id, name, cover_face_id, hidden)`
- `transcripts(id, asset_id, segment_id, lang, text)` + `transcripts_fts` (FTS5 trigram) and `assets_fts` (filename, tags, person names, camera)
- `audio_tags(asset_id, segment_id, label, score)`
- `embeddings(id, space[clip_b32|siglip2_b16|face_sface|clap], asset_id, segment_id?, dim, vec BLOB f16, model_rev)` — durable copy; usearch key = `embeddings.id`
- `jobs(id, kind, asset_id?, priority, state[queued|running|done|failed|skipped], attempts, lease_until, payload, error, created_at, updated_at)` index `(state, priority DESC, id)`
- `models(name, rev, path, sha256, device_bench_ms)`, `thumb_cache(key, size, atime)`, `schema_meta`

**File identity**: `quick_hash = blake3(size ‖ first 1 MiB ‖ last 1 MiB ‖ middle 64 KiB)` at scan; full `content_hash` as an idle-priority job for exact dedup. A `missing` file matching `(size, quick_hash)` of a new `present` file is relinked to the same asset so tags, faces, and embeddings survive moves/renames. Identical `content_hash` files share one asset.

### Indexing pipeline

Stages are job kinds with their own worker pools:

1. `scan` (1 per root, `walkdir`) upserts `files`, bumps `seen_gen`, marks absent as missing.
2. `hash_quick` (IO, 2) -> `meta` (4) -> `thumb` (num_cores) -> `embed_visual` (1 batch worker, batch 16–32) -> `face` (1 batch) -> `video_keyframes` (1–2 ffmpeg procs) -> `audio_tag` -> `transcribe` (1, memory-gated) -> `hash_full` (idle only).
3. Priorities: viewport-visible assets 100 (UI calls `boost(asset_ids)`), new files 50, backfill 10, full hash 0.
4. Crash safety: lease-based claiming in one transaction, expired leases requeued at startup, `attempts > 3` -> `failed` with error shown in UI. Each stage idempotent, output + `done` committed together.
5. Batching: ML worker drains up to 32 same-kind jobs, preprocesses on `rayon`, one `session.run` per batch. Batch size from the startup benchmark.
6. Incremental: `notify` + `notify-debouncer-full` (2 s) feeds `scan`; nightly or on-demand full rescan; `.xmp` changes trigger `xmp_import`.
7. Memory governor: global budget on resident model bytes; whisper and SigLIP SO400M never co-resident below 6 GB RAM; transcription pauses when free RAM < 1 GB.

### Search

Query DSL: free text + filters (`date:2023-07..2023-08`, `camera:`, `lens:`, `gps:bbox`, `dur:>60s`, `person:Anna`, `tag:`, `kind:video`, `rating:>=3`).

1. Filters -> SQL -> `roaring` bitmap of candidate asset/segment ids.
2. Free text runs in parallel: text tower -> ANN over visual space (k=500, `filtered_search` when bitmap < ~20% of corpus, else over-fetch k×4 and post-filter), FTS5 over transcripts (BM25), FTS5 over `assets_fts`, exact match on `persons` if it looks like a name.
3. Reciprocal Rank Fusion (k=60), weights visual 1.0 / transcript 0.8 / filename+tags 1.2. Video score = max over segments; best segment timestamp returned for "jump to moment".
4. Similar-image = ANN on the clicked asset's vector; face search = ANN in face space grouped by `person_id`.
5. Grouping is a presentation step: day/month (`taken_at_local`), event (gap > 4 h), person, location (geohash-6 buckets + bundled offline `cities1000` reverse geocode).

### Model manager and device selection

- `models.toml` registry baked in: name, pinned HF revision URL, sha256, size, licence, task, dim, preprocessing params, variants {fp32, fp16, int8}.
- Download-on-demand to `<app_data>/models/<name>/<rev>/` with resumable `reqwest` ranges, sha256 verify, atomic rename. Licence-gate dialog for non-commercial packs.
- ONNX Runtime itself is a runtime pack in the same registry (`ort-cpu`, `ort-cuda`, `ort-directml`; macOS build includes CoreML).
- Device probe at first launch and on demand: enumerate CUDA / DirectML / CoreML / Vulkan(whisper) / CPU, build a session with `.error_on_failure()`, run 8 warm + 16 timed inferences on CLIP B/32, persist `device_bench_ms`, pick fastest, allow user override. CPU path uses int8/q4f16 variants with intra-op threads = physical cores − 1.

### Thumbnail / preview cache

- Sizes: `t256` (grid, WebP q80), `p1024` (preview and model input source, WebP q85), `p2560` only for RAW/HEIC.
- Layout: `<cache>/thumbs/<2 hex>/<quick_hash>-<size>.webp`, so thumbs survive renames and duplicates share.
- Video: `t256` at 10% duration; 20-frame scrub sprite generated on first hover as a priority job.
- LRU eviction by `atime` in `thumb_cache`, default cap 10% of disk (min 2 GB), `p2560` evicted first.
- Served by `orb-thumb://localhost/<quick_hash>/<size>` (Windows: `http://orb-thumb.localhost/...`); handler generates `t256` on miss (≤ 4 concurrent). CSP `img-src orb-thumb: http://orb-thumb.localhost`.

### XMP sidecar sync

- Written only on user change, never on import: `dc:subject` + `lr:hierarchicalSubject`, `xmp:Rating`, `xmp:Label`, `dc:title`, `dc:description`, `Iptc4xmpExt:PersonInImage`, `mwg-rs:Regions` (normalised face rects with names), `photoshop:DateCreated` if user corrected the date, plus private `orb:assetId` / `orb:writtenAt`.
- Sidecar path `IMG_0001.xmp` next to `IMG_0001.CR2` (Lightroom convention), same for video. Temp file + atomic rename; originals never touched.
- Import on scan when sidecar mtime > `orb:writtenAt`, `source='xmp'`.
- Conflicts: tags = set union minus orb's explicit removals; rating = last-writer-wins by mtime; regions = external names win, orb keeps geometry. Unresolvable differences go to a "Sidecar conflicts" view.

### Privacy / offline

Network only from the model downloader and the updater, both behind explicit user action and a persisted "offline mode" switch. Tauri capabilities grant `http` only to Hugging Face and the GitHub release host. No telemetry. "Forget this library" deletes DB and caches.

## Phased delivery

Each phase is shippable on its own; the "first usable release" the user asked for is the end of Phase 4.

### Phase 0 — Toolchain and skeleton (~1 week)
- Install `rustup` (stable ≥ 1.88), Tauri Linux prerequisites (`libwebkit2gtk-4.1-dev build-essential libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf`), `cmake`, `clang`, Node 20 via `fnm`, `pnpm`, `cargo install tauri-cli@^2`.
- `just fetch-sidecars` downloads LGPL static ffmpeg/ffprobe into `apps/desktop/src-tauri/binaries/ffmpeg-<triple>`.
- Workspace with empty crates, `orb-cli` printing versions, Tauri hello window.
- CI: `ci.yml` (fmt/clippy/test on ubuntu-22.04, windows-latest, macos-14), `release.yml` with `tauri-action`.
- Dev-box mitigations: `CARGO_BUILD_JOBS=2`, `mold`/`lld`, `opt-level=1` for deps in dev profile, 4 GB swapfile.
- Verify: CI green on three OSes producing unsigned bundles.

### Phase 1 — Library browser (~3–4 weeks)
- Roots management, scanner, `files/assets`, metadata via nom-exif + ffprobe, thumbnails via `orb-thumb`.
- SolidJS justified grid virtualised to 100k+ items; sort by date/name/size/duration; filter panel (date range, camera, kind, has-GPS); group by day/month; detail pane with full metadata; lightbox (RAW via embedded preview).
- Key crates: `walkdir`, `rusqlite`, `nom-exif`, `image`, `rawler`, `libheif-rs`, `fast_image_resize`, `webp`, `ffmpeg-sidecar`, `tokio`, `rayon`.
- Verify: fixture corpus asserts `taken_at`/GPS/camera per file; `orb-cli bench` target 10k JPEGs scanned + metadata + t256 in < 4 min on 4 cores; grid scroll at 60 fps with 100k rows.

### Phase 2 — Semantic search (~3 weeks)
- Model manager + downloader, `ort` `load-dynamic` CPU pack, CLIP B/32 int8 embeddings from `p1024`, usearch index with mmap `view`, text search box, "find similar", RRF fusion with filename FTS.
- Key crates: `ort`, `tokenizers`, `usearch`, `ndarray`, `roaring`, `reqwest`, `sha2`.
- Verify: embed 10 fixture images and assert cosine ≥ 0.99 against reference vectors from the Python model (preprocessing parity); ≥ 8 img/s on 4 cores; 1M synthetic vectors recall@10 ≥ 0.95 vs brute force; search latency < 100 ms.

### Phase 3 — People (~3 weeks)
- YuNet detection on `p1024`, SFace embeddings, incremental clustering (DBSCAN via `linfa-clustering` on face space; new faces attach to nearest centroid if cos > 0.6 else pending), People view, naming, merge/split, `person:` filter. Optional `buffalo_l` pack behind licence acceptance.
- Store `cluster_id` separately from `person_id`; re-cluster only unconfirmed faces to avoid drift.
- Verify: fixture of 5 identities × 10 photos clusters with purity ≥ 0.9 and no cross-identity merges.

### Phase 4 — Video intelligence (~4 weeks)
- Scene-change keyframes (`select='gt(scene,0.3)'` plus 1 frame / 10 s fallback, cap 400 per video) -> `segments` + per-segment CLIP vectors; scrub strips; whisper `base` transcription with timestamps into FTS5; YAMNet tags per 1 s window aggregated per segment; transcript panel with click-to-seek.
- Audio extracted via `ffmpeg -vn -ac 1 -ar 16000 -f s16le -` streamed to stdin of whisper.
- Verify: 1 h of 1080p -> keyframes + CLIP + base transcript in < 15 min on 4 cores; transcript fixture WER sanity check; search "dog barking" returns tagged clip.

### Phase 5 — Acceleration (~3 weeks)
- Runtime packs for CUDA (Linux/Windows), DirectML (Windows), CoreML (macOS); device benchmark UI; fp16 models on GPU; `orb-whisper` sidecar builds with `cuda`/`vulkan`/`metal`; batch-size autotune; `orb-cli bench` JSON reports in `docs/benchmarks/`.
- Verify: cross-device embedding parity cos ≥ 0.995 fp16 vs fp32; DirectML path compared against CPU bench to catch silent per-op fallback slowdowns.

### Phase 6 — Sidecars, albums, polish, distribution (~3–4 weeks)
- XMP write/import/conflict view, manual + smart albums, keyboard culling (rating, flags), map view, settings, signed updater, Windows NSIS + MSI, macOS signed/notarised DMG, Linux AppImage + .deb.
- Verify: sidecar round-trip read back with ExifTool in CI; Lightroom-style sidecar fixture import; `codesign --verify --deep` on every macOS tag build.

## Top risks and mitigations

1. **HEIC/AVIF/RAW portability on Windows**: ffmpeg sidecar fallback for every decoder; rawler embedded previews avoid demosaic; per-format fixtures on all three OSes in CI.
2. **ONNX Runtime size and CUDA ABI churn**: `load-dynamic` + versioned runtime packs; CPU pack always works; CUDA pack must pass the benchmark before selection.
3. **4 GB machines**: int8/q4f16 models, one model resident at a time, memory governor, thumbnails as model input, usearch mmap `view`.
4. **macOS notarisation of dlopen'd libs and sidecars**: sign every Mach-O with hardened runtime + `disable-library-validation` entitlement; notarise in CI on tags.
5. **Face model licensing**: YuNet+SFace default; insightface opt-in with licence text.
6. **Index/DB consistency after crashes**: SQLite authoritative; usearch rebuilt from `embeddings.vec` on `index_rev` mismatch; leased, idempotent jobs.
7. **Vector volume at 1M**: cap keyframes per video; f16 storage (~1 GB for 1M × 512-d).

## Critical files (to be created)

- `Cargo.toml` — workspace, feature gates, dev profile tuned for the small dev box
- `crates/orb-db/migrations/0001_init.sql` — schema above
- `crates/orb-jobs/src/scheduler.rs` — lease queue, stage pools, batching, memory governor
- `crates/orb-ml/models.toml` and `crates/orb-ml/src/device.rs` — registry, downloader, EP probe/benchmark
- `crates/orb-index/src/search.rs` — bitmap pre-filter, ANN, FTS, RRF fusion
- `apps/desktop/src-tauri/src/protocol.rs` and `tauri.conf.json` — orb-thumb scheme, externalBin, CSP, updater

## Immediate next step

Execute Phase 0: install the toolchain on this machine, create the workspace skeleton, fetch ffmpeg sidecars, get the Tauri hello window and `orb-cli --version` building, and push CI config.
