use crate::cache::{AppCacheState, ProfileCache};
use tauri::Manager;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub kind: String, // "movie" | "show" | "live"
    pub id: u64,
    pub name: String,
    pub icon: String,
}

#[derive(serde::Serialize)]
pub struct ProfileSearchResults {
    pub profile: String,
    pub results: Vec<SearchResult>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn search_profile_cache(pc: &ProfileCache, q: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut seen_live = std::collections::HashSet::new();
    let mut seen_movies = std::collections::HashSet::new();
    let mut seen_shows = std::collections::HashSet::new();

    for streams in pc.live_streams.values() {
        for s in streams {
            if s.name.to_lowercase().contains(q) && seen_live.insert(s.stream_id) {
                results.push(SearchResult { kind: "live".into(), id: s.stream_id, name: s.name.clone(), icon: s.stream_icon.clone() });
            }
        }
    }
    for streams in pc.vod_streams.values() {
        for s in streams {
            if s.name.to_lowercase().contains(q) && seen_movies.insert(s.stream_id) {
                results.push(SearchResult { kind: "movie".into(), id: s.stream_id, name: s.name.clone(), icon: s.stream_icon.clone() });
            }
        }
    }
    for items in pc.series_items.values() {
        for s in items {
            if s.name.to_lowercase().contains(q) && seen_shows.insert(s.series_id) {
                results.push(SearchResult { kind: "show".into(), id: s.series_id, name: s.name.clone(), icon: s.cover.clone() });
            }
        }
    }

    results.sort_unstable_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));
    results
}

// ─── Command ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn search_all_profiles(app: tauri::AppHandle, query: String) -> Vec<ProfileSearchResults> {
    if query.trim().is_empty() {
        return vec![];
    }
    let q = query.to_lowercase();
    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();

    let mut all: Vec<ProfileSearchResults> = cache
        .iter()
        .filter_map(|(profile, pc)| {
            let results = search_profile_cache(pc, &q);
            if results.is_empty() { None } else { Some(ProfileSearchResults { profile: profile.clone(), results }) }
        })
        .collect();

    all.sort_unstable_by(|a, b| a.profile.cmp(&b.profile));
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::ProfileCache;
    use crate::live::LiveStream;
    use crate::vods::VodStream;
    use crate::series::SeriesItem;

    fn make_cache(
        live: Vec<(&str, Vec<LiveStream>)>,
        vods: Vec<(&str, Vec<VodStream>)>,
        series: Vec<(&str, Vec<SeriesItem>)>,
    ) -> ProfileCache {
        let mut pc = ProfileCache::default();
        for (cat, streams) in live {
            pc.live_streams.insert(cat.to_string(), streams);
        }
        for (cat, streams) in vods {
            pc.vod_streams.insert(cat.to_string(), streams);
        }
        for (cat, items) in series {
            pc.series_items.insert(cat.to_string(), items);
        }
        pc
    }

    fn live_stream(id: u64, name: &str) -> LiveStream {
        LiveStream { num: id as u32, stream_id: id, name: name.to_string(), stream_icon: String::new(), epg_channel_id: String::new() }
    }

    fn vod_stream(id: u64, name: &str) -> VodStream {
        VodStream { stream_id: id, name: name.to_string(), stream_icon: String::new() }
    }

    fn series_item(id: u64, name: &str) -> SeriesItem {
        SeriesItem { series_id: id, name: name.to_string(), cover: String::new() }
    }

    #[test]
    fn returns_empty_when_no_match() {
        let pc = make_cache(
            vec![("News", vec![live_stream(1, "BBC One")])],
            vec![],
            vec![],
        );
        assert!(search_profile_cache(&pc, "xyz").is_empty());
    }

    #[test]
    fn matches_live_stream_case_insensitively() {
        let pc = make_cache(
            vec![("News", vec![live_stream(1, "BBC One"), live_stream(2, "CNN")])],
            vec![],
            vec![],
        );
        let results = search_profile_cache(&pc, "bbc");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, "live");
        assert_eq!(results[0].name, "BBC One");
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn matches_vod_and_series() {
        let pc = make_cache(
            vec![],
            vec![("Action", vec![vod_stream(10, "Batman Begins")])],
            vec![("Drama", vec![series_item(20, "Batman: The Series")])],
        );
        let results = search_profile_cache(&pc, "batman");
        assert_eq!(results.len(), 2);
        let kinds: Vec<&str> = results.iter().map(|r| r.kind.as_str()).collect();
        assert!(kinds.contains(&"movie"));
        assert!(kinds.contains(&"show"));
    }

    #[test]
    fn deduplicates_live_streams_across_categories() {
        // Same stream_id appearing in two categories should only appear once.
        let pc = make_cache(
            vec![
                ("Cat A", vec![live_stream(1, "BBC One")]),
                ("Cat B", vec![live_stream(1, "BBC One")]),
            ],
            vec![],
            vec![],
        );
        let results = search_profile_cache(&pc, "bbc");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn results_sorted_by_kind_then_name() {
        let pc = make_cache(
            vec![("News", vec![live_stream(1, "BBC One"), live_stream(2, "Al Jazeera")])],
            vec![("Action", vec![vod_stream(10, "Batman")])],
            vec![],
        );
        let results = search_profile_cache(&pc, "a");
        // kind order: live < movie < show (alphabetical)
        for w in results.windows(2) {
            let ord = w[0].kind.cmp(&w[1].kind).then(w[0].name.cmp(&w[1].name));
            assert!(ord != std::cmp::Ordering::Greater, "results not sorted: {:?} before {:?}", w[0].name, w[1].name);
        }
    }
}
