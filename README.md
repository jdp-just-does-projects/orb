# orb

Fast, local-first photo and video manager for Windows, Linux and macOS.

- Reads EXIF / IPTC / XMP / QuickTime / Matroska metadata to tag, sort and group media.
- Builds a **local** content index: visual embeddings (CLIP/SigLIP), faces, speech transcripts
  (whisper) and sound tags (YAMNet), all computed on your machine (CPU, optional GPU) and stored
  locally. No network access except explicit model downloads and updates.
- Never modifies originals. Tags, ratings and people are kept in orb's SQLite database and
  optionally written to `.xmp` sidecars.

## Layout

| Path | Purpose |
|---|---|
| `crates/orb-core` | shared ids, errors, config, paths |
| `crates/orb-db` | SQLite schema and repositories (source of truth) |
| `crates/orb-meta` | metadata extraction, XMP sidecars |
| `crates/orb-media` | decoding, thumbnails, ffmpeg wrappers |
| `crates/orb-ml` | ONNX Runtime, model registry, device probing, whisper |
| `crates/orb-index` | usearch ANN indexes, hybrid search |
| `crates/orb-jobs` | job queue, worker pools, watcher |
| `crates/orb-cli` | headless `orb` binary for CI and benchmarks |
| `apps/desktop` | Tauri 2 + SolidJS desktop app |
| `fixtures` | test media corpus (one file per format) |

## Development

Prerequisites: Rust stable (≥ 1.88), Node 20, pnpm, `cargo install tauri-cli just`, and the
[Tauri Linux packages](https://v2.tauri.app/start/prerequisites/) on Linux (`mold` + `clang` are used
as the linker; delete `.cargo/config.toml` if you cannot install them).

```sh
pnpm install
just fetch-sidecars   # ffmpeg/ffprobe for your platform
just dev              # desktop app
just cli info         # headless CLI
just check            # fmt + clippy + tests
```

See `docs/PLAN.md` for the architecture and phased roadmap.
