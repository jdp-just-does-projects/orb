use camino::Utf8PathBuf;
use directories::ProjectDirs;

/// Standard on-disk locations for orb's own state. Never inside a library root.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Database and config.
    pub data: Utf8PathBuf,
    /// Thumbnails, previews, ANN index files (all rebuildable).
    pub cache: Utf8PathBuf,
    /// Downloaded models and ONNX Runtime packs.
    pub models: Utf8PathBuf,
}

impl AppPaths {
    pub fn discover() -> anyhow::Result<Self> {
        let dirs = ProjectDirs::from("dev", "orb", "orb")
            .ok_or_else(|| anyhow::anyhow!("no home directory available"))?;
        let to_utf8 = |p: &std::path::Path| {
            Utf8PathBuf::from_path_buf(p.to_path_buf())
                .map_err(|p| anyhow::anyhow!("non-UTF-8 path: {}", p.display()))
        };
        let data = to_utf8(dirs.data_dir())?;
        let cache = to_utf8(dirs.cache_dir())?;
        let models = data.join("models");
        Ok(Self {
            data,
            cache,
            models,
        })
    }

    pub fn db_file(&self) -> Utf8PathBuf {
        self.data.join("orb.sqlite3")
    }

    pub fn ensure(&self) -> std::io::Result<()> {
        for p in [&self.data, &self.cache, &self.models] {
            std::fs::create_dir_all(p)?;
        }
        Ok(())
    }
}
