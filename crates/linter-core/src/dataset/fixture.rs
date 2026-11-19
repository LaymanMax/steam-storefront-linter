use crate::dataset::db::{BenchmarkDb, BenchmarkGameRow, DbError};
use crate::dataset::models::DatasetMetadata;
use chrono::Utc;
use std::path::Path;

/// Generates a realistic sample benchmark database with ~200 games across genres and years (2020-2024),
/// including sparse and playtest records to test exclusions and cohort calculations.
pub fn generate_sample_benchmark_db(db_path: impl AsRef<Path>) -> Result<DatasetMetadata, DbError> {
    let db = BenchmarkDb::open(db_path)?;

    let genres_pool: [Vec<String>; 8] = [
        vec!["Action".into(), "Indie".into(), "RPG".into()],
        vec!["Indie".into(), "Action".into(), "Roguelike".into()],
        vec!["Adventure".into(), "Indie".into(), "Casual".into()],
        vec!["Strategy".into(), "Indie".into(), "Simulation".into()],
        vec!["Action".into(), "Shooter".into()],
        vec!["RPG".into(), "Adventure".into()],
        vec!["Indie".into(), "Puzzle".into(), "Casual".into()],
        vec!["Simulation".into(), "Building".into(), "Indie".into()],
    ];

    let mut eligible_count = 0;
    let mut excluded_count = 0;

    for i in 1..=220 {
        let is_playtest = i == 15 || i == 75 || i == 145;
        let is_sparse = i == 30 || i == 90;

        let name = if is_playtest {
            format!("Mystery Game {} Playtest", i)
        } else if is_sparse {
            format!("Barebones Title {}", i)
        } else {
            format!("Indie Quest {}", i)
        };

        let is_excluded = is_playtest || is_sparse;
        let exclusion_reason = if is_playtest {
            Some("title_contains_playtest".to_string())
        } else if is_sparse {
            Some("sparse_metadata".to_string())
        } else {
            None
        };

        if is_excluded {
            excluded_count += 1;
        } else {
            eligible_count += 1;
        }

        let release_year = Some(2020 + (i % 5) as i32); // 2020, 2021, 2022, 2023, 2024
        let genres = genres_pool[i % genres_pool.len()].clone();
        let is_indie = genres.iter().any(|g| g.eq_ignore_ascii_case("indie"));

        // Realistic variations in screenshot counts (typically 4 to 18)
        let screenshots_count = if is_sparse {
            None
        } else {
            Some(4 + (i * 7 % 15))
        };

        let movies_count = if is_sparse {
            None
        } else {
            Some(if i % 4 == 0 { 0 } else { 1 + (i % 3) })
        };
        let has_trailer = movies_count.map(|c| c > 0);

        let short_desc_words = if is_sparse {
            None
        } else {
            Some(20 + (i * 3 % 40))
        };
        let short_desc_chars = short_desc_words.map(|w| w * 6);

        let detailed_desc_words = if is_sparse {
            None
        } else {
            Some(120 + (i * 19 % 450))
        };
        let detailed_desc_chars = detailed_desc_words.map(|w| w * 6);

        // Languages: 1 to 12
        let interface_languages_count = if is_sparse {
            None
        } else {
            Some(1 + (i * 5 % 11))
        };
        let full_audio_languages_count = if is_sparse {
            None
        } else if i % 2 == 0 {
            Some(1)
        } else {
            Some(0)
        };

        let has_support_url = Some(i % 3 != 0);
        let has_support_email = Some(i % 2 == 0);
        let has_support_contact =
            Some(has_support_url == Some(true) || has_support_email == Some(true));

        let supports_windows = Some(true);
        let supports_mac = Some(i % 3 == 0);
        let supports_linux = Some(i % 4 == 0);

        let categories_count = if is_sparse { None } else { Some(2 + (i % 6)) };

        let row = BenchmarkGameRow {
            app_id: 100000 + i as u32,
            name,
            release_year,
            is_indie,
            genres,
            screenshots_count,
            movies_count,
            has_trailer,
            short_desc_words,
            short_desc_chars,
            detailed_desc_words,
            detailed_desc_chars,
            interface_languages_count,
            full_audio_languages_count,
            has_support_contact,
            has_support_url,
            has_support_email,
            supports_windows,
            supports_mac,
            supports_linux,
            categories_count,
            is_excluded,
            exclusion_reason,
        };

        db.insert_game(&row)?;
    }

    let metadata = DatasetMetadata {
        source_url: "https://huggingface.co/datasets/FronkonGames/steam-games-dataset".into(),
        dataset_id: "FronkonGames/steam-games-dataset (Sample Benchmark Cohort)".into(),
        revision: Some("6c21457b9ebf0a6a7f5b91e11281d65d456ea30e".into()),
        downloaded_at: Some(Utc::now()),
        ingested_at: Some(Utc::now()),
        source_row_count: 220,
        unique_appid_count: 220,
        duplicate_count: 0,
        excluded_count,
        eligible_benchmark_count: eligible_count,
        file_size_bytes: 450_000,
        license: "MIT".into(),
        attribution: "FronkonGames/steam-games-dataset (MIT License)".into(),
    };

    db.save_metadata(&metadata)?;
    Ok(metadata)
}
