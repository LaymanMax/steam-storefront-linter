# Audit Rules Reference

Every audit rule evaluated by `linter-core` is rule-based, deterministic, and versioned.

Rules evaluate against the normalized `StoreListing` model and produce structured `Finding` records.

---

## Severity Definitions

| Severity     | Meaning                                                                              | CI Exit Code   |
| ------------ | ------------------------------------------------------------------------------------ | -------------- |
| **Critical** | Significant listing defect that severely impairs store conversion or discoverability | Non-zero (`1`) |
| **Warning**  | Meaningful completeness issue worth reviewing before launch                          | `0` (clean)    |
| **Info**     | Informational observation or optional opportunity                                    | `0` (clean)    |
| **Pass**     | Verification passed successfully                                                     | `0` (clean)    |

---

## Audit Categories & Rules

### 1. Description Completeness

- `description.short.missing` (Critical): Short description is omitted.
- `description.short.empty` (Critical): Short description contains only whitespace or tags.
- `description.short.unusually_short` (Warning): Less than 8 words or 40 characters.
- `description.short.exceeds_recommended_length` (Info): Greater than 350 characters (Steam tooltips may truncate text > 300 chars).
- `description.short.pass` (Pass): Concise and well-proportioned.
- `description.detailed.missing` (Critical): Detailed description ("About This Game") missing.
- `description.detailed.empty` (Critical): Detailed description contains no readable text.
- `description.detailed.placeholder` (Critical): Apparent placeholder text detected ("Lorem Ipsum", "Coming soon", "TBD", "Work in progress").
- `description.detailed.too_brief` (Warning): Under 50 words (Balanced profile) or under 100 words (Strict profile).
- `description.detailed.pass` (Pass): Detailed description is populated.

### 2. Screenshot Completeness

- `screenshots.missing_data` (Warning): Screenshot metadata unavailable from source.
- `screenshots.zero` (Critical): 0 screenshots uploaded.
- `screenshots.too_few` (Warning): Fewer than 5 screenshots (Balanced) or fewer than 8 screenshots (Strict).
- `screenshots.duplicates_detected` (Warning): Duplicate full-resolution URLs detected.
- `screenshots.pass` (Pass): 5 or more screenshots available.

### 3. Trailers & Video

- `trailers.unknown` (Info): Video metadata unavailable.
- `trailers.none_listed` (Info): No trailer uploaded. (Informational, not penalized as a failure).
- `trailers.pass` (Pass): At least 1 video trailer uploaded.

### 4. Localization Coverage

- `localization.interface.unknown` (Warning): Interface language metadata missing.
- `localization.interface.empty` (Warning): 0 interface languages declared.
- `localization.single_language` (Info): 1 language supported.
- `localization.interface.pass` (Pass): 2 or more interface languages supported.
- `localization.audio.none` (Info): No full audio voiceover declared (standard for text/indie games).
- `localization.audio.pass` (Pass): Full audio voiceover declared.

### 5. Support Information

- `support.missing_url_and_email` (Warning): Neither support URL nor support email provided.
- `support.missing_support_url` (Info / Warning in Strict): Email present, but no dedicated support URL / knowledgebase / Discord.
- `support.pass` (Pass): Support contact details available.

### 6. Features & Categories

- `features.categories.unknown` (Info): Category tags unavailable.
- `features.categories.none` (Info): No Steam categories declared.
- `features.categories.pass` (Pass): Feature categories declared (e.g. Single-player, Controller support).

### 7. Platform Support

- `platforms.windows_missing` (Warning): Windows platform compatibility not checked.
- `platforms.status` (Pass): Platform compatibility explicitly indicated for Windows, macOS, and Linux.

### 8. General Metadata

- `general.developers.missing` (Warning): No developer name provided.
- `general.developers.pass` (Pass): Developer name present.
- `general.publishers.missing` (Info): No publisher name specified.
- `general.header_image.missing` (Critical): 460x215px header capsule image missing.
- `general.header_image.pass` (Pass): Header capsule image present.
- `general.genres.missing` (Warning): No primary genres assigned.
- `general.genres.pass` (Pass): Primary genres assigned.
- `general.release_date.missing` (Info / Warning in Strict): Public release date not specified.
