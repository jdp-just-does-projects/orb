use serde::Serialize;

#[derive(Serialize)]
pub struct AppInfo {
    version: &'static str,
    data_dir: String,
    cache_dir: String,
}

#[tauri::command]
pub fn app_info() -> Result<AppInfo, String> {
    let paths = orb_core::paths::AppPaths::discover().map_err(|e| e.to_string())?;
    Ok(AppInfo {
        version: orb_core::VERSION,
        data_dir: paths.data.to_string(),
        cache_dir: paths.cache.to_string(),
    })
}
