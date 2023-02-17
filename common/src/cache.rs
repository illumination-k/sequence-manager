use std::{
    fs::metadata,
    path::Path,
    time::{Duration, SystemTime},
};

pub fn is_cache_update_required(path: &Path, interval_secs: u64) -> bool {
    let meta = metadata(path);

    if let Ok(meta) = meta {
        let modified_time = meta.modified().expect("Should get modified time");
        SystemTime::now()
            .duration_since(modified_time)
            .expect("Should get systemtime")
            >= Duration::from_secs(interval_secs)
    } else {
        false
    }
}
