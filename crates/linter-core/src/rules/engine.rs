use crate::models::finding::{Finding, FindingCategory, FindingEvidence, Severity};
use crate::models::listing::StoreListing;
use crate::parser::html::{count_characters, count_words, strip_html_tags};
use crate::rules::profile::AuditProfile;
use std::collections::HashSet;

pub const RULESET_VERSION: &str = "1.0.0";

pub struct AuditEngine;

impl AuditEngine {
    pub fn audit(listing: &StoreListing, profile: AuditProfile) -> Vec<Finding> {
        let mut findings = Vec::new();

        Self::audit_descriptions(listing, profile, &mut findings);
        Self::audit_screenshots(listing, profile, &mut findings);
        Self::audit_trailers(listing, profile, &mut findings);
        Self::audit_localization(listing, profile, &mut findings);
        Self::audit_support(listing, profile, &mut findings);
        Self::audit_features(listing, profile, &mut findings);
        Self::audit_platforms(listing, profile, &mut findings);
        Self::audit_general_metadata(listing, profile, &mut findings);

        findings
    }

    fn audit_descriptions(
        listing: &StoreListing,
        profile: AuditProfile,
        findings: &mut Vec<Finding>,
    ) {
        // 1. Short description
        match &listing.short_description {
            None => {
                findings.push(Finding {
                    id: "description.short.missing".into(),
                    category: FindingCategory::Description,
                    severity: Severity::Critical,
                    title: "Short description missing".into(),
                    message: "Your listing has no short description. This is displayed at the top right of your store page and in search results.".into(),
                    recommendation: "Provide a concise summary (around 150-300 characters) outlining your game's genre, hook, and core premise.".into(),
                    evidence: FindingEvidence {
                        field: Some("short_description".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: Some("Concise text under 300 characters".into()),
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(short_raw) => {
                let clean_short = strip_html_tags(short_raw);
                let word_count = count_words(&clean_short);
                let char_count = count_characters(&clean_short);

                if clean_short.is_empty() {
                    findings.push(Finding {
                        id: "description.short.empty".into(),
                        category: FindingCategory::Description,
                        severity: Severity::Critical,
                        title: "Short description contains no readable text".into(),
                        message:
                            "The short description contains only whitespace or empty HTML tags."
                                .into(),
                        recommendation: "Write a clear, hook-driven elevator pitch for your game."
                            .into(),
                        evidence: FindingEvidence {
                            field: Some("short_description".into()),
                            actual: Some(serde_json::json!(0)),
                            expected: Some("Non-empty text".into()),
                            threshold: Some(serde_json::json!(1)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else if word_count < 8 || char_count < 40 {
                    findings.push(Finding {
                        id: "description.short.unusually_short".into(),
                        category: FindingCategory::Description,
                        severity: Severity::Warning,
                        title: "Short description is unusually brief".into(),
                        message: format!(
                            "Your short description is only {} words ({} characters). It may not provide enough context for prospective players.",
                            word_count, char_count
                        ),
                        recommendation: "Expand your elevator pitch to explain what players do in the game and why it is engaging.".into(),
                        evidence: FindingEvidence {
                            field: Some("short_description".into()),
                            actual: Some(serde_json::json!(word_count)),
                            expected: Some(">= 10 words".into()),
                            threshold: Some(serde_json::json!(10)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else if char_count > 350 {
                    findings.push(Finding {
                        id: "description.short.exceeds_recommended_length".into(),
                        category: FindingCategory::Description,
                        severity: Severity::Info,
                        title: "Short description exceeds recommended length".into(),
                        message: format!(
                            "Your short description is {} characters long. Steam store cards may truncate text longer than 300 characters.",
                            char_count
                        ),
                        recommendation: "Consider trimming the short description so that it fits naturally without truncation in search tooltips.".into(),
                        evidence: FindingEvidence {
                            field: Some("short_description".into()),
                            actual: Some(serde_json::json!(char_count)),
                            expected: Some("<= 300 characters".into()),
                            threshold: Some(serde_json::json!(300)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    findings.push(Finding {
                        id: "description.short.pass".into(),
                        category: FindingCategory::Description,
                        severity: Severity::Pass,
                        title: "Short description present and well-proportioned".into(),
                        message: format!("Short description has {} words ({} characters).", word_count, char_count),
                        recommendation: "Maintain accurate descriptions reflecting updates and gameplay features.".into(),
                        evidence: FindingEvidence {
                            field: Some("short_description".into()),
                            actual: Some(serde_json::json!(word_count)),
                            expected: None,
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                }
            }
        }

        // 2. Detailed description
        match &listing.detailed_description {
            None => {
                findings.push(Finding {
                    id: "description.detailed.missing".into(),
                    category: FindingCategory::Description,
                    severity: Severity::Critical,
                    title: "Detailed description is missing".into(),
                    message: "Your listing has no detailed description (\"About This Game\").".into(),
                    recommendation: "Provide a comprehensive 'About This Game' section with headings, gameplay breakdowns, and animated GIFs/images.".into(),
                    evidence: FindingEvidence {
                        field: Some("detailed_description".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: Some("Rich detailed description".into()),
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(detailed_raw) => {
                let clean_detailed = strip_html_tags(detailed_raw);
                let word_count = count_words(&clean_detailed);
                let char_count = count_characters(&clean_detailed);

                // Placeholder detection
                let lower = clean_detailed.to_lowercase();
                let is_placeholder = lower.contains("lorem ipsum")
                    || lower.contains("placeholder")
                    || lower == "coming soon"
                    || lower == "to be announced"
                    || lower == "tbd"
                    || lower == "work in progress";

                if clean_detailed.is_empty() {
                    findings.push(Finding {
                        id: "description.detailed.empty".into(),
                        category: FindingCategory::Description,
                        severity: Severity::Critical,
                        title: "Detailed description is empty".into(),
                        message: "The detailed description field is present but contains only whitespace or empty tags.".into(),
                        recommendation: "Write a complete 'About This Game' section detailing your mechanics, world, and features.".into(),
                        evidence: FindingEvidence {
                            field: Some("detailed_description".into()),
                            actual: Some(serde_json::json!(0)),
                            expected: Some(">= 50 words".into()),
                            threshold: Some(serde_json::json!(50)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else if is_placeholder {
                    findings.push(Finding {
                        id: "description.detailed.placeholder".into(),
                        category: FindingCategory::Description,
                        severity: Severity::Critical,
                        title: "Detailed description appears to contain placeholder text".into(),
                        message: "The detailed description contains apparent placeholder wording such as 'Coming soon', 'TBD', or 'Lorem ipsum'.".into(),
                        recommendation: "Replace placeholder copy with genuine game descriptions before launching or driving store traffic.".into(),
                        evidence: FindingEvidence {
                            field: Some("detailed_description".into()),
                            actual: Some(serde_json::json!(clean_detailed)),
                            expected: Some("Production description text".into()),
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    let min_words = if profile == AuditProfile::Strict {
                        100
                    } else {
                        50
                    };
                    if word_count < min_words {
                        findings.push(Finding {
                            id: "description.detailed.too_brief".into(),
                            category: FindingCategory::Description,
                            severity: Severity::Warning,
                            title: "Detailed description is unusually brief".into(),
                            message: format!(
                                "Your detailed description contains only {} words ({} characters). Comparable games typically provide more detailed breakdowns.",
                                word_count, char_count
                            ),
                            recommendation: "Consider expanding with subsections describing core mechanics, game loops, progression, and story background.".into(),
                            evidence: FindingEvidence {
                                field: Some("detailed_description".into()),
                                actual: Some(serde_json::json!(word_count)),
                                expected: Some(format!(">= {} words", min_words)),
                                threshold: Some(serde_json::json!(min_words)),
                            },
                            rule_version: RULESET_VERSION.into(),
                        });
                    } else {
                        findings.push(Finding {
                            id: "description.detailed.pass".into(),
                            category: FindingCategory::Description,
                            severity: Severity::Pass,
                            title: "Detailed description is well-populated".into(),
                            message: format!(
                                "Detailed description has {} words ({} characters).",
                                word_count, char_count
                            ),
                            recommendation:
                                "Regularly update with major content patches and roadmap updates."
                                    .into(),
                            evidence: FindingEvidence {
                                field: Some("detailed_description".into()),
                                actual: Some(serde_json::json!(word_count)),
                                expected: None,
                                threshold: None,
                            },
                            rule_version: RULESET_VERSION.into(),
                        });
                    }
                }
            }
        }
    }

    fn audit_screenshots(
        listing: &StoreListing,
        profile: AuditProfile,
        findings: &mut Vec<Finding>,
    ) {
        match &listing.screenshots {
            None => {
                findings.push(Finding {
                    id: "screenshots.missing_data".into(),
                    category: FindingCategory::Screenshots,
                    severity: Severity::Warning,
                    title: "Screenshot metadata unavailable".into(),
                    message: "The listing source did not provide screenshot records.".into(),
                    recommendation:
                        "Verify that screenshots are uploaded to Steamworks and marked as public."
                            .into(),
                    evidence: FindingEvidence {
                        field: Some("screenshots".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: Some("Screenshot list".into()),
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(shots) => {
                let count = shots.len();
                if count == 0 {
                    findings.push(Finding {
                        id: "screenshots.zero".into(),
                        category: FindingCategory::Screenshots,
                        severity: Severity::Critical,
                        title: "No screenshots listed".into(),
                        message: "Your listing currently has 0 screenshots. Store visitors cannot preview gameplay without screenshots.".into(),
                        recommendation: "Upload at least 5 high-resolution 1080p or 4K in-game screenshots showing actual gameplay, UI, and environments.".into(),
                        evidence: FindingEvidence {
                            field: Some("screenshots".into()),
                            actual: Some(serde_json::json!(0)),
                            expected: Some(">= 5 screenshots".into()),
                            threshold: Some(serde_json::json!(5)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    let min_recommended = if profile == AuditProfile::Strict {
                        8
                    } else {
                        5
                    };
                    if count < min_recommended {
                        findings.push(Finding {
                            id: "screenshots.too_few".into(),
                            category: FindingCategory::Screenshots,
                            severity: Severity::Warning,
                            title: "Limited screenshot coverage".into(),
                            message: format!(
                                "Your listing contains {} screenshot(s). Most published games maintain at least {}.",
                                count, min_recommended
                            ),
                            recommendation: "Consider reviewing whether additional screenshots would help explain gameplay, combat/systems, and variety.".into(),
                            evidence: FindingEvidence {
                                field: Some("screenshots".into()),
                                actual: Some(serde_json::json!(count)),
                                expected: Some(format!(">= {} screenshots", min_recommended)),
                                threshold: Some(serde_json::json!(min_recommended)),
                            },
                            rule_version: RULESET_VERSION.into(),
                        });
                    } else {
                        findings.push(Finding {
                            id: "screenshots.pass".into(),
                            category: FindingCategory::Screenshots,
                            severity: Severity::Pass,
                            title: "Healthy screenshot coverage".into(),
                            message: format!("Listing contains {} screenshot(s).", count),
                            recommendation: "Ensure the first 4 screenshots display actual gameplay rather than concept art or title cards.".into(),
                            evidence: FindingEvidence {
                                field: Some("screenshots".into()),
                                actual: Some(serde_json::json!(count)),
                                expected: None,
                                threshold: None,
                            },
                            rule_version: RULESET_VERSION.into(),
                        });
                    }

                    // Check duplicate screenshot URLs
                    let mut seen = HashSet::new();
                    let mut duplicates = 0;
                    for s in shots {
                        if !s.url_full.is_empty() && !seen.insert(&s.url_full) {
                            duplicates += 1;
                        }
                    }
                    if duplicates > 0 {
                        findings.push(Finding {
                            id: "screenshots.duplicates_detected".into(),
                            category: FindingCategory::Screenshots,
                            severity: Severity::Warning,
                            title: "Duplicate screenshot references detected".into(),
                            message: format!("Found {} duplicate screenshot URL(s) in listing records.", duplicates),
                            recommendation: "Remove duplicate screenshot uploads in Steamworks to ensure diverse media presentation.".into(),
                            evidence: FindingEvidence {
                                field: Some("screenshots".into()),
                                actual: Some(serde_json::json!(duplicates)),
                                expected: Some("0 duplicates".into()),
                                threshold: Some(serde_json::json!(0)),
                            },
                            rule_version: RULESET_VERSION.into(),
                        });
                    }
                }
            }
        }
    }

    fn audit_trailers(listing: &StoreListing, _profile: AuditProfile, findings: &mut Vec<Finding>) {
        match &listing.movies {
            None => {
                findings.push(Finding {
                    id: "trailers.unknown".into(),
                    category: FindingCategory::Trailers,
                    severity: Severity::Info,
                    title: "Trailer metadata unavailable".into(),
                    message: "Trailer and video information was not returned by the data source.".into(),
                    recommendation: "Ensure trailer media is uploaded and set to public visibility in Steamworks.".into(),
                    evidence: FindingEvidence {
                        field: Some("movies".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: None,
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(movies) => {
                if movies.is_empty() {
                    findings.push(Finding {
                        id: "trailers.none_listed".into(),
                        category: FindingCategory::Trailers,
                        severity: Severity::Info,
                        title: "No trailer or video listed".into(),
                        message: "The listing currently has no video trailers uploaded.".into(),
                        recommendation: "Adding a gameplay trailer that shows real mechanics within the first 5 seconds can help players evaluate the game.".into(),
                        evidence: FindingEvidence {
                            field: Some("movies".into()),
                            actual: Some(serde_json::json!(0)),
                            expected: Some(">= 1 trailer".into()),
                            threshold: Some(serde_json::json!(1)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    findings.push(Finding {
                        id: "trailers.pass".into(),
                        category: FindingCategory::Trailers,
                        severity: Severity::Pass,
                        title: "Trailer present".into(),
                        message: format!("Listing has {} video trailer(s) available.", movies.len()),
                        recommendation: "Ensure the primary trailer immediately demonstrates gameplay rather than lengthy logos.".into(),
                        evidence: FindingEvidence {
                            field: Some("movies".into()),
                            actual: Some(serde_json::json!(movies.len())),
                            expected: None,
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                }
            }
        }
    }

    fn audit_localization(
        listing: &StoreListing,
        profile: AuditProfile,
        findings: &mut Vec<Finding>,
    ) {
        match &listing.supported_languages {
            None => {
                findings.push(Finding {
                    id: "localization.interface.unknown".into(),
                    category: FindingCategory::Localization,
                    severity: Severity::Warning,
                    title: "Interface localization data missing".into(),
                    message: "The data source did not specify supported interface languages.".into(),
                    recommendation: "Configure supported languages in Steamworks Store Page Admin under Basic Info.".into(),
                    evidence: FindingEvidence {
                        field: Some("supported_languages".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: Some("Language list".into()),
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(langs) => {
                let count = langs.len();
                if count == 0 {
                    findings.push(Finding {
                        id: "localization.interface.empty".into(),
                        category: FindingCategory::Localization,
                        severity: Severity::Warning,
                        title: "No interface languages declared".into(),
                        message: "Listing lists 0 supported interface languages.".into(),
                        recommendation:
                            "Specify at least the primary language (e.g. English) in Steamworks."
                                .into(),
                        evidence: FindingEvidence {
                            field: Some("supported_languages".into()),
                            actual: Some(serde_json::json!(0)),
                            expected: Some(">= 1 language".into()),
                            threshold: Some(serde_json::json!(1)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else if count == 1 {
                    findings.push(Finding {
                        id: "localization.single_language".into(),
                        category: FindingCategory::Localization,
                        severity: Severity::Info,
                        title: "Single language support declared".into(),
                        message: format!("Only 1 language is currently supported ({}).", langs[0]),
                        recommendation: "Consider reviewing player demographics in your genre to see if adding Simplified Chinese, German, Spanish, or Japanese makes sense for your launch.".into(),
                        evidence: FindingEvidence {
                            field: Some("supported_languages".into()),
                            actual: Some(serde_json::json!(1)),
                            expected: None,
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    findings.push(Finding {
                        id: "localization.interface.pass".into(),
                        category: FindingCategory::Localization,
                        severity: Severity::Pass,
                        title: "Multi-language interface coverage".into(),
                        message: format!("Listing supports {} interface language(s).", count),
                        recommendation: "Ensure store descriptions are also localized in Steamworks for each supported language.".into(),
                        evidence: FindingEvidence {
                            field: Some("supported_languages".into()),
                            actual: Some(serde_json::json!(count)),
                            expected: None,
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                }
            }
        }

        // Full audio localization
        if let Some(audio) = &listing.full_audio_languages {
            if audio.is_empty() {
                findings.push(Finding {
                    id: "localization.audio.none".into(),
                    category: FindingCategory::Localization,
                    severity: Severity::Info,
                    title: "No full-audio voiceover declared".into(),
                    message: "Listing indicates no full-audio localization. This is completely standard for many indie and text-based titles.".into(),
                    recommendation: "If your game features full voice acting, verify that audio checkboxes are checked in Steamworks language settings.".into(),
                    evidence: FindingEvidence {
                        field: Some("full_audio_languages".into()),
                        actual: Some(serde_json::json!(0)),
                        expected: None,
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            } else {
                findings.push(Finding {
                    id: "localization.audio.pass".into(),
                    category: FindingCategory::Localization,
                    severity: Severity::Pass,
                    title: "Full audio voiceover declared".into(),
                    message: format!(
                        "Listing includes full audio voiceover in {} language(s).",
                        audio.len()
                    ),
                    recommendation:
                        "Maintain subtitle options alongside voiceover for accessibility.".into(),
                    evidence: FindingEvidence {
                        field: Some("full_audio_languages".into()),
                        actual: Some(serde_json::json!(audio.len())),
                        expected: None,
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
        } else if profile == AuditProfile::Strict {
            findings.push(Finding {
                id: "localization.audio.unknown".into(),
                category: FindingCategory::Localization,
                severity: Severity::Info,
                title: "Full audio data unverified".into(),
                message: "Source language string does not explicitly declare full audio status."
                    .into(),
                recommendation:
                    "Verify language table in Steamworks Store Admin to confirm audio indicators."
                        .into(),
                evidence: FindingEvidence {
                    field: Some("full_audio_languages".into()),
                    actual: Some(serde_json::Value::Null),
                    expected: None,
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        }
    }

    fn audit_support(listing: &StoreListing, profile: AuditProfile, findings: &mut Vec<Finding>) {
        let has_url = listing
            .support_url
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let has_email = listing
            .support_email
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let has_website = listing
            .website
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);

        if !has_url && !has_email {
            findings.push(Finding {
                id: "support.missing_url_and_email".into(),
                category: FindingCategory::Support,
                severity: Severity::Warning,
                title: "No dedicated support URL or email provided".into(),
                message: "Listing does not specify a support contact URL or customer service email address.".into(),
                recommendation: "Add a support email or bug-tracker/Discord link in Steamworks so players encountering launch issues can contact you.".into(),
                evidence: FindingEvidence {
                    field: Some("support_info".into()),
                    actual: Some(serde_json::json!({"url": has_url, "email": has_email})),
                    expected: Some("Support URL or email".into()),
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        } else if !has_url {
            let severity = if profile == AuditProfile::Strict {
                Severity::Warning
            } else {
                Severity::Info
            };
            findings.push(Finding {
                id: "support.missing_support_url".into(),
                category: FindingCategory::Support,
                severity,
                title: "Support email provided, but no dedicated support website".into(),
                message: "Listing has a support email, but no dedicated support URL or knowledge base link.".into(),
                recommendation: "Consider adding a link to your FAQ, community hub, or Discord server in the support URL field.".into(),
                evidence: FindingEvidence {
                    field: Some("support_url".into()),
                    actual: Some(serde_json::Value::Null),
                    expected: Some("Valid support URL".into()),
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        } else {
            findings.push(Finding {
                id: "support.pass".into(),
                category: FindingCategory::Support,
                severity: Severity::Pass,
                title: "Support contact details available".into(),
                message: "Dedicated support contact information is configured.".into(),
                recommendation: "Ensure support links remain monitored around game updates and sales events.".into(),
                evidence: FindingEvidence {
                    field: Some("support_info".into()),
                    actual: Some(serde_json::json!({"has_url": has_url, "has_email": has_email, "has_website": has_website})),
                    expected: None,
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        }
    }

    fn audit_features(listing: &StoreListing, _profile: AuditProfile, findings: &mut Vec<Finding>) {
        match &listing.categories {
            None => {
                findings.push(Finding {
                    id: "features.categories.unknown".into(),
                    category: FindingCategory::Features,
                    severity: Severity::Info,
                    title: "Feature categories data unavailable".into(),
                    message: "Steam category metadata was not returned by the data source.".into(),
                    recommendation: "Configure feature categories (e.g. Single-player, Steam Achievements, Controller support) in Steamworks.".into(),
                    evidence: FindingEvidence {
                        field: Some("categories".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: Some("Category list".into()),
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(cats) => {
                if cats.is_empty() {
                    findings.push(Finding {
                        id: "features.categories.none".into(),
                        category: FindingCategory::Features,
                        severity: Severity::Info,
                        title: "No feature categories declared".into(),
                        message: "The listing currently has zero declared Steam categories. Categories appear on the right sidebar and help filter games.".into(),
                        recommendation: "Check applicable categories such as Single-player, Controller support, Steam Cloud, or Achievements.".into(),
                        evidence: FindingEvidence {
                            field: Some("categories".into()),
                            actual: Some(serde_json::json!(0)),
                            expected: Some(">= 1 category".into()),
                            threshold: Some(serde_json::json!(1)),
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    findings.push(Finding {
                        id: "features.categories.pass".into(),
                        category: FindingCategory::Features,
                        severity: Severity::Pass,
                        title: "Feature categories declared".into(),
                        message: format!("Listing has {} Steam feature category(ies) tagged.", cats.len()),
                        recommendation: "Audit category tags to ensure controller and cloud save support are accurately reflected.".into(),
                        evidence: FindingEvidence {
                            field: Some("categories".into()),
                            actual: Some(serde_json::json!(cats.len())),
                            expected: None,
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                }
            }
        }
    }

    fn audit_platforms(
        listing: &StoreListing,
        _profile: AuditProfile,
        findings: &mut Vec<Finding>,
    ) {
        match &listing.platforms {
            None => {
                findings.push(Finding {
                    id: "platforms.unknown".into(),
                    category: FindingCategory::Platforms,
                    severity: Severity::Info,
                    title: "Platform compatibility unknown".into(),
                    message: "Platform compatibility details were not available.".into(),
                    recommendation:
                        "Ensure executable depots are linked to supported platforms in Steamworks."
                            .into(),
                    evidence: FindingEvidence {
                        field: Some("platforms".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: None,
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(p) => {
                if !p.windows {
                    findings.push(Finding {
                        id: "platforms.windows_missing".into(),
                        category: FindingCategory::Platforms,
                        severity: Severity::Warning,
                        title: "Windows support not declared".into(),
                        message: "Windows support is not checked. Over 96% of Steam users run Windows.".into(),
                        recommendation: "If your build runs on Windows, ensure the Windows platform box is enabled in Store Admin.".into(),
                        evidence: FindingEvidence {
                            field: Some("platforms.windows".into()),
                            actual: Some(serde_json::json!(false)),
                            expected: Some("true".into()),
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    findings.push(Finding {
                        id: "platforms.status".into(),
                        category: FindingCategory::Platforms,
                        severity: Severity::Pass,
                        title: "Platform support configured".into(),
                        message: format!(
                            "Windows: Supported | macOS: {} | Linux: {}",
                            if p.mac { "Supported" } else { "Not supported" },
                            if p.linux { "Supported" } else { "Not supported" }
                        ),
                        recommendation: "Note: Steam Deck verification and Proton compatibility may enable Linux players to enjoy Windows builds.".into(),
                        evidence: FindingEvidence {
                            field: Some("platforms".into()),
                            actual: Some(serde_json::json!({"windows": p.windows, "mac": p.mac, "linux": p.linux})),
                            expected: None,
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                }
            }
        }
    }

    fn audit_general_metadata(
        listing: &StoreListing,
        profile: AuditProfile,
        findings: &mut Vec<Finding>,
    ) {
        // Developers
        if listing.developers.is_empty() {
            findings.push(Finding {
                id: "general.developers.missing".into(),
                category: FindingCategory::General,
                severity: Severity::Warning,
                title: "Developer name missing".into(),
                message: "No developer name is listed. Developers appear on your store header and creator homepage.".into(),
                recommendation: "Specify your studio or individual developer name in Basic Info.".into(),
                evidence: FindingEvidence {
                    field: Some("developers".into()),
                    actual: Some(serde_json::json!(0)),
                    expected: Some(">= 1 developer".into()),
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        } else {
            findings.push(Finding {
                id: "general.developers.pass".into(),
                category: FindingCategory::General,
                severity: Severity::Pass,
                title: "Developer information present".into(),
                message: format!("Developer: {}", listing.developers.join(", ")),
                recommendation: "Set up a Steam Creator Homepage to cross-promote other titles."
                    .into(),
                evidence: FindingEvidence {
                    field: Some("developers".into()),
                    actual: Some(serde_json::json!(listing.developers)),
                    expected: None,
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        }

        // Publishers
        if listing.publishers.is_empty() {
            findings.push(Finding {
                id: "general.publishers.missing".into(),
                category: FindingCategory::General,
                severity: Severity::Info,
                title: "Publisher not specified".into(),
                message: "No publisher is specified. For self-published indie games, enter your developer/studio name.".into(),
                recommendation: "Fill in the publisher field in Basic Info.".into(),
                evidence: FindingEvidence {
                    field: Some("publishers".into()),
                    actual: Some(serde_json::json!(0)),
                    expected: Some(">= 1 publisher".into()),
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        }

        // Header image
        if listing.header_image.is_none() {
            findings.push(Finding {
                id: "general.header_image.missing".into(),
                category: FindingCategory::General,
                severity: Severity::Critical,
                title: "Header capsule image is missing".into(),
                message: "No header capsule image is available. The header capsule is the primary banner for your game across Steam.".into(),
                recommendation: "Upload a 460x215px header capsule in Graphical Assets.".into(),
                evidence: FindingEvidence {
                    field: Some("header_image".into()),
                    actual: Some(serde_json::Value::Null),
                    expected: Some("Valid image URL".into()),
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        } else {
            findings.push(Finding {
                id: "general.header_image.pass".into(),
                category: FindingCategory::General,
                severity: Severity::Pass,
                title: "Header capsule image present".into(),
                message: "Header capsule image is specified.".into(),
                recommendation:
                    "Ensure key art title text is readable at small mobile preview sizes.".into(),
                evidence: FindingEvidence {
                    field: Some("header_image".into()),
                    actual: Some(serde_json::json!(listing.header_image)),
                    expected: None,
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        }

        // Genres
        match &listing.genres {
            None => {
                findings.push(Finding {
                    id: "general.genres.missing".into(),
                    category: FindingCategory::General,
                    severity: Severity::Warning,
                    title: "Genre tags missing".into(),
                    message: "No primary genres are specified on the listing.".into(),
                    recommendation: "Select primary genres in Steamworks Store Admin to aid store discovery algorithms.".into(),
                    evidence: FindingEvidence {
                        field: Some("genres".into()),
                        actual: Some(serde_json::Value::Null),
                        expected: Some(">= 1 genre".into()),
                        threshold: None,
                    },
                    rule_version: RULESET_VERSION.into(),
                });
            }
            Some(genres) => {
                if genres.is_empty() {
                    findings.push(Finding {
                        id: "general.genres.empty".into(),
                        category: FindingCategory::General,
                        severity: Severity::Warning,
                        title: "No genres assigned".into(),
                        message: "The genres list is empty.".into(),
                        recommendation: "Assign at least one primary genre in Store Admin.".into(),
                        evidence: FindingEvidence {
                            field: Some("genres".into()),
                            actual: Some(serde_json::json!(0)),
                            expected: Some(">= 1 genre".into()),
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                } else {
                    findings.push(Finding {
                        id: "general.genres.pass".into(),
                        category: FindingCategory::General,
                        severity: Severity::Pass,
                        title: "Primary genres assigned".into(),
                        message: format!("Genres: {}", genres.join(", ")),
                        recommendation: "In addition to genres, apply user tags via the store page for fine-grained discoverability.".into(),
                        evidence: FindingEvidence {
                            field: Some("genres".into()),
                            actual: Some(serde_json::json!(genres)),
                            expected: None,
                            threshold: None,
                        },
                        rule_version: RULESET_VERSION.into(),
                    });
                }
            }
        }

        // Release date
        if listing.release_date.is_none() {
            let severity = if profile == AuditProfile::Strict {
                Severity::Warning
            } else {
                Severity::Info
            };
            findings.push(Finding {
                id: "general.release_date.missing".into(),
                category: FindingCategory::General,
                severity,
                title: "Release date not specified".into(),
                message: "No public release date or quarter is specified.".into(),
                recommendation: "Provide a target release date or launch quarter ('Coming Soon', 'Q4 2024') to enable wishlist tracking.".into(),
                evidence: FindingEvidence {
                    field: Some("release_date".into()),
                    actual: Some(serde_json::Value::Null),
                    expected: Some("Release date string".into()),
                    threshold: None,
                },
                rule_version: RULESET_VERSION.into(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::listing::ListingSource;

    #[test]
    fn test_audit_minimal_empty_listing() {
        let listing = StoreListing::new(12345, "Test Game", ListingSource::JsonUpload);
        let findings = AuditEngine::audit(&listing, AuditProfile::Balanced);

        assert!(findings
            .iter()
            .any(|f| f.id == "description.short.missing" && f.severity == Severity::Critical));
        assert!(findings
            .iter()
            .any(|f| f.id == "description.detailed.missing" && f.severity == Severity::Critical));
        assert!(findings
            .iter()
            .any(|f| f.id == "screenshots.missing_data" && f.severity == Severity::Warning));
        assert!(findings
            .iter()
            .any(|f| f.id == "general.developers.missing"));
    }

    #[test]
    fn test_audit_placeholder_detailed() {
        let mut listing = StoreListing::new(12345, "Test Game", ListingSource::JsonUpload);
        listing.detailed_description = Some("Lorem ipsum dolor sit amet".into());
        let findings = AuditEngine::audit(&listing, AuditProfile::Balanced);

        assert!(findings.iter().any(
            |f| f.id == "description.detailed.placeholder" && f.severity == Severity::Critical
        ));
    }
}
