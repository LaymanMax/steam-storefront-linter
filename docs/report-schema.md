# Report Schema Reference

Schema Version: `1.0.0`

Reports exported in JSON format conform to the stable schema below:

```json
{
  "schema_version": "1.0.0",
  "audit_id": "a4e1a7e4-a637-448f-a97a-143fe1366f18",
  "generated_at": "2024-11-19T04:27:51.368187Z",
  "audit_profile": "balanced",
  "listing": {
    "app_id": 2539430,
    "name": "Dragon Forge: Chronicles",
    "short_description": "Forge legendary gear, delve into ancient dungeons, and tame dragons in this indie action-RPG crafting adventure.",
    "detailed_description": "<h2>Enter the Realm of the Dragon Forge</h2>...",
    "developers": ["Emberstone Studios"],
    "publishers": ["Emberstone Studios"],
    "release_date": "24 Oct, 2024",
    "release_year": 2024,
    "genres": ["Action", "Adventure", "Indie", "RPG"],
    "categories": [
      "Single-player",
      "Online Co-op",
      "Steam Achievements",
      "Full controller support"
    ],
    "supported_languages": [
      "English",
      "French",
      "German",
      "Spanish - Spain",
      "Japanese",
      "Simplified Chinese"
    ],
    "full_audio_languages": ["English"],
    "platforms": {
      "windows": true,
      "mac": true,
      "linux": true
    },
    "screenshots": [
      {
        "id": 0,
        "url_thumbnail": "https://example.com/screens/thumb1.jpg",
        "url_full": "https://example.com/screens/full1.jpg"
      }
    ],
    "movies": [
      {
        "id": 0,
        "name": "Official Gameplay Reveal Trailer"
      }
    ],
    "website": "https://emberstone.example.com/dragonforge",
    "support_url": "https://emberstone.example.com/support",
    "support_email": "support@emberstone.example.com",
    "header_image": "https://example.com/header.jpg",
    "store_url": "https://store.steampowered.com/app/2539430/",
    "source": "json_upload"
  },
  "summary": {
    "critical": 0,
    "warnings": 0,
    "info": 0,
    "passed": 12,
    "unknown": 0,
    "total_checks": 12,
    "section_coverage": {
      "Description": 100.0,
      "Screenshots": 100.0,
      "Trailers & Video": 100.0,
      "Localization": 100.0,
      "Support & Links": 100.0,
      "Features & Categories": 100.0,
      "Platform Support": 100.0,
      "General Metadata": 100.0
    }
  },
  "findings": [
    {
      "id": "screenshots.pass",
      "category": "screenshots",
      "severity": "pass",
      "title": "Healthy screenshot coverage",
      "message": "Listing contains 7 screenshot(s).",
      "recommendation": "Ensure the first 4 screenshots display actual gameplay rather than concept art or title cards.",
      "evidence": {
        "field": "screenshots",
        "actual": 7
      },
      "rule_version": "1.0.0"
    }
  ],
  "benchmarks": [
    {
      "metric_id": "screenshots.count",
      "metric_name": "Screenshot Count",
      "category": "screenshots",
      "listing_value_display": "7 Screenshots",
      "status": "calculated",
      "distribution": {
        "sample_size": 33,
        "usable_records": 33,
        "coverage_pct": 100.0,
        "listing_value": 7.0,
        "min": 4.0,
        "max": 18.0,
        "p25": 7.0,
        "median": 12.0,
        "p75": 17.0,
        "percentile_rank": 16.7,
        "share_above": 66.7,
        "share_below": 15.2
      },
      "cohort_definition": "Indie released in 2024 games sharing genre(s) [Action, Adventure, Indie, RPG]",
      "explanation": "Your listing's screenshot count (7) is at the 17th percentile. Approximately 67% of comparable games have a higher value."
    }
  ],
  "cohort": {
    "cohort_name": "Indie released in 2024 games sharing genre(s) [Action, Adventure, Indie, RPG]",
    "total_games_in_cohort": 33,
    "genres": ["Action", "Adventure", "Indie", "RPG"],
    "release_year": 2024,
    "is_indie_restricted": true,
    "fallback_applied": false,
    "explanation": "Specific cohort matching target genres (Action, Adventure, Indie, RPG) and release year (Some(2024))."
  },
  "methodology": {
    "dataset_source": "FronkonGames/steam-games-dataset",
    "dataset_license": "MIT",
    "snapshot_date": "2024-11",
    "total_dataset_games": 215,
    "min_sample_size_threshold": 30,
    "percentile_convention": "Linear rank interpolation (percentile rank = [less + 0.5 * equal] / total)",
    "disclaimer": "Benchmark distributions are descriptive references based on public Steam games. Listing completeness does not predict sales, commercial revenue, or review sentiment."
  },
  "data_quality": {
    "fields_evaluated": 13,
    "fields_known": 13,
    "fields_missing_from_source": 0,
    "data_completeness_pct": 100.0
  },
  "provenance": {
    "screenshots": {
      "field": "screenshots",
      "known": true,
      "source": "json_upload"
    }
  }
}
```
