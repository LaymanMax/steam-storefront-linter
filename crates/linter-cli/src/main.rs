use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;
use linter_core::dataset::{BenchmarkDb, DatasetDownloader, DatasetIngester, DEFAULT_DATASET_URL};
use linter_core::models::benchmark::BenchmarkStatus;
use linter_core::models::finding::Severity;
use linter_core::models::listing::StoreListing;
use linter_core::models::report::AuditReport;
use linter_core::parser::json_import::import_listing_json;
use linter_core::parser::steam_url::parse_app_id_or_url;
use linter_core::report::{generate_html_report, generate_json_report};
use linter_core::rules::profile::AuditProfile;
use linter_core::steam::SteamClient;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "steam-storefront-linter",
    author = "Steam Storefront Linter Contributors",
    version,
    about = "Deterministic completeness auditor and peer benchmarking tool for Steam game listings",
    long_about = "Steam Storefront Linter audits the metadata completeness of Steam store listings and compares them against similar games using the FronkonGames/steam-games-dataset.\n\nNote: This is an auditor for listing completeness and descriptive benchmarking. It does NOT predict game sales, commercial revenue, or review scores."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Audit a Steam listing from an AppID, store URL, or local JSON file
    Audit(AuditArgs),
    /// Manage the benchmark dataset (download, ingest, inspect, validate)
    Dataset(DatasetCommand),
    /// Inspect benchmark cohorts and distributions
    Benchmark(BenchmarkCommand),
}

#[derive(Args)]
struct AuditArgs {
    /// Steam AppID or Steam store URL (e.g. 2539430 or https://store.steampowered.com/app/2539430/)
    #[arg(value_name = "APP_ID_OR_URL")]
    target: Option<String>,

    /// Path to a JSON file containing listing metadata instead of connecting to Steam
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input_file: Option<PathBuf>,

    /// Country code for Steam Store API (e.g. 'us', 'de', 'jp')
    #[arg(short = 'c', long = "country", default_value = "us")]
    country: String,

    /// Language for Steam Store API (e.g. 'english', 'french', 'schinese')
    #[arg(short = 'l', long = "language", default_value = "english")]
    language: String,

    /// Audit strictness profile
    #[arg(short = 'p', long = "profile", value_enum, default_value = "balanced")]
    profile: ProfileArg,

    /// Output format
    #[arg(short = 'f', long = "format", value_enum, default_value = "terminal")]
    format: FormatArg,

    /// Output report file path (writes JSON or HTML depending on format, or stdout if omitted)
    #[arg(short = 'o', long = "output", value_name = "PATH")]
    output_path: Option<PathBuf>,

    /// Path to the benchmark SQLite database
    #[arg(long = "db", default_value = "data/processed/benchmark.db")]
    db_path: PathBuf,

    /// Minimum usable sample size required to display comparative percentiles
    #[arg(long = "min-samples", default_value_t = 30)]
    min_samples: usize,
}

#[derive(Args)]
struct DatasetCommand {
    #[command(subcommand)]
    action: DatasetAction,
}

#[derive(Subcommand)]
enum DatasetAction {
    /// Download the public Parquet dataset from Hugging Face
    Download {
        /// Custom download URL
        #[arg(long, default_value = DEFAULT_DATASET_URL)]
        url: String,
        /// Destination path
        #[arg(
            short = 'o',
            long = "output",
            default_value = "data/raw/train-00000-of-00001.parquet"
        )]
        output: PathBuf,
        /// Force re-download even if the file already exists
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Ingest and normalize the raw Parquet dataset into the SQLite benchmark database
    Ingest {
        /// Raw Parquet input file
        #[arg(
            short = 'r',
            long = "raw",
            default_value = "data/raw/train-00000-of-00001.parquet"
        )]
        raw: PathBuf,
        /// Processed SQLite database output path
        #[arg(
            short = 'o',
            long = "output",
            default_value = "data/processed/benchmark.db"
        )]
        output: PathBuf,
        /// Manifest summary output path
        #[arg(
            short = 'm',
            long = "manifest",
            default_value = "data/processed/ingestion_manifest.json"
        )]
        manifest: PathBuf,
    },
    /// Initialize a sample benchmark database for local development and offline testing
    Sample {
        /// Processed SQLite database output path
        #[arg(
            short = 'o',
            long = "output",
            default_value = "data/processed/benchmark.db"
        )]
        output: PathBuf,
    },
    /// Inspect metadata, row counts, and data quality of the processed dataset
    Inspect {
        /// Benchmark SQLite database path
        #[arg(long = "db", default_value = "data/processed/benchmark.db")]
        db: PathBuf,
    },
    /// Validate the processed database integrity and index coverage
    Validate {
        /// Benchmark SQLite database path
        #[arg(long = "db", default_value = "data/processed/benchmark.db")]
        db: PathBuf,
    },
}

#[derive(Args)]
struct BenchmarkCommand {
    #[command(subcommand)]
    action: BenchmarkAction,
}

#[derive(Subcommand)]
enum BenchmarkAction {
    /// Display benchmark database status and availability
    Status {
        /// Benchmark SQLite database path
        #[arg(long = "db", default_value = "data/processed/benchmark.db")]
        db: PathBuf,
    },
    /// Inspect metric distributions for a specific cohort
    Inspect {
        /// Genre name (e.g. 'Action', 'RPG', 'Indie')
        #[arg(short = 'g', long = "genre")]
        genre: Option<String>,
        /// Release year (e.g. 2024)
        #[arg(short = 'y', long = "year")]
        year: Option<i32>,
        /// Restrict cohort to Indie games
        #[arg(long, default_value_t = true)]
        indie: bool,
        /// Benchmark SQLite database path
        #[arg(long = "db", default_value = "data/processed/benchmark.db")]
        db: PathBuf,
        /// Minimum sample size threshold
        #[arg(long = "min-samples", default_value_t = 30)]
        min_samples: usize,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ProfileArg {
    Balanced,
    Strict,
}

impl From<ProfileArg> for AuditProfile {
    fn from(p: ProfileArg) -> Self {
        match p {
            ProfileArg::Balanced => AuditProfile::Balanced,
            ProfileArg::Strict => AuditProfile::Strict,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FormatArg {
    Terminal,
    Json,
    Html,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit(args) => handle_audit(args).await,
        Commands::Dataset(cmd) => handle_dataset(cmd).await,
        Commands::Benchmark(cmd) => handle_benchmark(cmd).await,
    }
}

async fn handle_audit(args: AuditArgs) -> ExitCode {
    let profile: AuditProfile = args.profile.into();

    // 1. Acquire listing
    let listing_res = if let Some(input_file) = &args.input_file {
        match fs::read_to_string(input_file) {
            Ok(content) => import_listing_json(&content)
                .map_err(|e| format!("Failed to parse JSON file {}: {}", input_file.display(), e)),
            Err(e) => Err(format!(
                "Could not read input file {}: {}",
                input_file.display(),
                e
            )),
        }
    } else if let Some(target) = &args.target {
        match parse_app_id_or_url(target) {
            Ok(app_id) => {
                let client = SteamClient::default();
                match client
                    .fetch_listing(app_id, Some(&args.country), Some(&args.language))
                    .await
                {
                    Ok(listing) => Ok(listing),
                    Err(e) => Err(format!("Steam fetch failed for AppID {}: {}", app_id, e)),
                }
            }
            Err(e) => Err(format!("Invalid AppID or Steam URL '{}': {}", target, e)),
        }
    } else {
        eprintln!(
            "{}",
            "Error: Must provide either a Steam AppID/URL or an --input JSON file."
                .red()
                .bold()
        );
        eprintln!("Run 'steam-storefront-linter audit --help' for usage instructions.");
        return ExitCode::from(2);
    };

    let listing = match listing_res {
        Ok(l) => l,
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            return ExitCode::from(2);
        }
    };

    // 2. Open benchmark database if present
    let benchmark_db = if args.db_path.exists() {
        match BenchmarkDb::open(&args.db_path) {
            Ok(db) => Some(db),
            Err(e) => {
                if args.format == FormatArg::Terminal {
                    eprintln!("{} Benchmark DB at {} could not be opened ({}). Running audit without benchmark comparisons.", "Warning:".yellow().bold(), args.db_path.display(), e);
                }
                None
            }
        }
    } else {
        if args.format == FormatArg::Terminal {
            eprintln!("{} No benchmark database found at {}. Running completeness audit without peer distributions.\nTo enable peer benchmarking, run:\n  steam-storefront-linter dataset download\n  steam-storefront-linter dataset ingest", "Note:".cyan(), args.db_path.display());
        }
        None
    };

    // 3. Run audit
    let report = linter_core::run_audit(listing, benchmark_db.as_ref(), profile, args.min_samples);

    // 4. Output results
    match args.format {
        FormatArg::Terminal => {
            render_terminal_report(&report);
            if let Some(out_path) = &args.output_path {
                if let Ok(json) = generate_json_report(&report) {
                    let _ = fs::write(out_path, json);
                    println!(
                        "\nSaved JSON report to {}",
                        out_path.display().to_string().cyan()
                    );
                }
            }
        }
        FormatArg::Json => match generate_json_report(&report) {
            Ok(json) => {
                if let Some(out_path) = &args.output_path {
                    if let Err(e) = fs::write(out_path, &json) {
                        eprintln!("Failed to write report to {}: {}", out_path.display(), e);
                        return ExitCode::from(2);
                    }
                } else {
                    println!("{}", json);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize report: {}", e);
                return ExitCode::from(2);
            }
        },
        FormatArg::Html => {
            let html = generate_html_report(&report);
            if let Some(out_path) = &args.output_path {
                if let Err(e) = fs::write(out_path, &html) {
                    eprintln!(
                        "Failed to write HTML report to {}: {}",
                        out_path.display(),
                        e
                    );
                    return ExitCode::from(2);
                }
                println!(
                    "Saved standalone HTML report to {}",
                    out_path.display().to_string().cyan()
                );
            } else {
                println!("{}", html);
            }
        }
    }

    if report.summary.critical > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

async fn handle_dataset(cmd: DatasetCommand) -> ExitCode {
    match cmd.action {
        DatasetAction::Download { url, output, force } => {
            println!("Downloading benchmark dataset from Hugging Face...");
            match DatasetDownloader::download(Some(&url), &output, force).await {
                Ok(path) => {
                    println!(
                        "{} Dataset downloaded to {}",
                        "Success:".green().bold(),
                        path.display()
                    );
                    println!("Next step: run 'steam-storefront-linter dataset ingest' to build the benchmark index.");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to download dataset: {}",
                        "Error:".red().bold(),
                        e
                    );
                    ExitCode::from(2)
                }
            }
        }
        DatasetAction::Ingest {
            raw,
            output,
            manifest,
        } => {
            if !raw.exists() {
                eprintln!(
                    "{} Raw Parquet file not found at {}.",
                    "Error:".red().bold(),
                    raw.display()
                );
                eprintln!("Please run 'steam-storefront-linter dataset download' first.");
                return ExitCode::from(2);
            }

            println!("Ingesting dataset from {}...", raw.display());
            if let Some(parent) = output.parent() {
                let _ = fs::create_dir_all(parent);
            }

            match DatasetIngester::ingest(&raw, &output, Some(&manifest)) {
                Ok(summary) => {
                    println!(
                        "{} Dataset ingested successfully into {}",
                        "Success:".green().bold(),
                        output.display()
                    );
                    println!(
                        "- Source rows processed: {}",
                        summary.metadata.source_row_count
                    );
                    println!(
                        "- Eligible games indexed: {}",
                        summary.metadata.eligible_benchmark_count
                    );
                    println!(
                        "- Duplicates deduplicated: {}",
                        summary.metadata.duplicate_count
                    );
                    println!(
                        "- Excluded playtests/sparse: {}",
                        summary.metadata.excluded_count
                    );
                    println!("- Ingestion manifest saved to: {}", manifest.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{} Ingestion failed: {}", "Error:".red().bold(), e);
                    ExitCode::from(2)
                }
            }
        }
        DatasetAction::Sample { output } => {
            if let Some(parent) = output.parent() {
                let _ = fs::create_dir_all(parent);
            }
            match linter_core::dataset::generate_sample_benchmark_db(&output) {
                Ok(meta) => {
                    println!(
                        "{} Initialized sample benchmark database at {}",
                        "Success:".green().bold(),
                        output.display()
                    );
                    println!("- Eligible games: {}", meta.eligible_benchmark_count);
                    println!("- Excluded games: {}", meta.excluded_count);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to generate sample database: {}",
                        "Error:".red().bold(),
                        e
                    );
                    ExitCode::from(2)
                }
            }
        }
        DatasetAction::Inspect { db } => {
            if !db.exists() {
                eprintln!(
                    "{} Database not found at {}.",
                    "Error:".red().bold(),
                    db.display()
                );
                return ExitCode::from(2);
            }
            match BenchmarkDb::open(&db) {
                Ok(bench_db) => match bench_db.get_metadata() {
                    Ok(Some(meta)) => {
                        println!("{}", "=== Steam Benchmark Dataset Information ===".bold());
                        println!("Source Dataset: {}", meta.dataset_id.cyan());
                        println!("License:        {}", meta.license.green());
                        println!("Source URL:     {}", meta.source_url);
                        println!(
                            "Ingested At:    {:?}",
                            meta.ingested_at.map(|d| d.to_string()).unwrap_or_default()
                        );
                        println!("Source Rows:    {}", meta.source_row_count);
                        println!("Unique AppIDs:  {}", meta.unique_appid_count);
                        println!("Eligible Pool:  {}", meta.eligible_benchmark_count);
                        println!("Exclusions:     {}", meta.excluded_count);

                        if let Ok(genres) = bench_db.get_available_genres() {
                            println!("\nTop Genres:");
                            for (g, count) in genres.iter().take(10) {
                                println!("  - {:<18} ({} games)", g, count);
                            }
                        }

                        if let Ok(years) = bench_db.get_available_years() {
                            println!("\nRecent Release Years:");
                            for (y, count) in years.iter().take(8) {
                                println!("  - {:<6} ({} games)", y, count);
                            }
                        }
                        ExitCode::SUCCESS
                    }
                    Ok(None) => {
                        eprintln!("Database exists but contains no metadata.");
                        ExitCode::from(2)
                    }
                    Err(e) => {
                        eprintln!("Database query error: {}", e);
                        ExitCode::from(2)
                    }
                },
                Err(e) => {
                    eprintln!("Failed to open database: {}", e);
                    ExitCode::from(2)
                }
            }
        }
        DatasetAction::Validate { db } => {
            if !db.exists() {
                eprintln!(
                    "{} Database not found at {}.",
                    "Error:".red().bold(),
                    db.display()
                );
                return ExitCode::from(2);
            }
            match BenchmarkDb::open(&db) {
                Ok(bench_db) => {
                    let meta = bench_db.get_metadata().ok().flatten();
                    let genres = bench_db.get_available_genres().unwrap_or_default();
                    let years = bench_db.get_available_years().unwrap_or_default();

                    let valid = meta.is_some() && !genres.is_empty() && !years.is_empty();
                    if valid {
                        let m = meta.unwrap();
                        println!(
                            "{} Benchmark database validation PASSED.",
                            "Valid:".green().bold()
                        );
                        println!("- Eligible games: {}", m.eligible_benchmark_count);
                        println!("- Indexed genres: {}", genres.len());
                        println!("- Release years:  {}", years.len());
                        ExitCode::SUCCESS
                    } else {
                        eprintln!(
                            "{} Benchmark database is incomplete or empty.",
                            "Invalid:".red().bold()
                        );
                        ExitCode::from(2)
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open database: {}", e);
                    ExitCode::from(2)
                }
            }
        }
    }
}

async fn handle_benchmark(cmd: BenchmarkCommand) -> ExitCode {
    match cmd.action {
        BenchmarkAction::Status { db } => {
            let exists = db.exists();
            println!("{}", "=== Benchmark System Status ===".bold());
            println!("Database Location: {}", db.display());
            println!(
                "Database Exists:   {}",
                if exists { "YES".green() } else { "NO".red() }
            );

            if exists {
                if let Ok(bench_db) = BenchmarkDb::open(&db) {
                    if let Ok(Some(meta)) = bench_db.get_metadata() {
                        println!("Dataset:           {}", meta.dataset_id);
                        println!("Eligible Pool:     {} games", meta.eligible_benchmark_count);
                        println!("Source Rows:       {} rows", meta.source_row_count);
                        println!(
                            "Ingestion Date:    {:?}",
                            meta.ingested_at.map(|d| d.to_string()).unwrap_or_default()
                        );
                    }
                }
            } else {
                println!("\nTo initialize the benchmark database, run:\n  steam-storefront-linter dataset download\n  steam-storefront-linter dataset ingest");
            }
            ExitCode::SUCCESS
        }
        BenchmarkAction::Inspect {
            genre,
            year,
            indie,
            db,
            min_samples,
        } => {
            if !db.exists() {
                eprintln!(
                    "{} Benchmark database not found at {}.",
                    "Error:".red().bold(),
                    db.display()
                );
                return ExitCode::from(2);
            }

            let bench_db = match BenchmarkDb::open(&db) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to open DB: {}", e);
                    return ExitCode::from(2);
                }
            };

            let target_genres = genre.map(|g| vec![g]).unwrap_or_default();
            match bench_db.select_cohort(&target_genres, year, indie, min_samples) {
                Ok((cohort, games)) => {
                    println!("{}", "=== Benchmark Cohort Inspection ===".bold());
                    println!("Cohort Name:    {}", cohort.cohort_name.cyan());
                    println!("Sample Size:    {} games", games.len());
                    println!(
                        "Fallback Used:  {}",
                        if cohort.fallback_applied {
                            "Yes".yellow()
                        } else {
                            "No".green()
                        }
                    );
                    println!("Explanation:    {}", cohort.explanation);

                    // Create synthetic dummy listing to inspect distributions
                    let dummy = StoreListing::new(
                        0,
                        "Inspection Dummy",
                        linter_core::models::listing::ListingSource::SampleFixture,
                    );
                    let results = BenchmarkDb::calculate_benchmark_metrics(
                        &dummy,
                        &cohort,
                        &games,
                        min_samples,
                    );

                    println!("\n{}", "Metric Distributions:".bold());
                    for r in results {
                        if let Some(dist) = r.distribution {
                            if r.status == BenchmarkStatus::Calculated {
                                println!(
                                    "  - {:<30} Median: {:<4.0} IQR: [{:.0} - {:.0}] Range: [{:.0} - {:.0}] ({} usable)",
                                    r.metric_name,
                                    dist.median.unwrap_or(0.0),
                                    dist.p25.unwrap_or(0.0),
                                    dist.p75.unwrap_or(0.0),
                                    dist.min.unwrap_or(0.0),
                                    dist.max.unwrap_or(0.0),
                                    dist.usable_records
                                );
                            }
                        } else if let Some(cat) = r.categorical {
                            if r.status == BenchmarkStatus::Calculated {
                                println!(
                                    "  - {:<30} Adoption: {:.0}% ({}/{} usable)",
                                    r.metric_name,
                                    cat.true_percentage,
                                    cat.true_count,
                                    cat.usable_records
                                );
                            }
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Cohort selection failed: {}", e);
                    ExitCode::from(2)
                }
            }
        }
    }
}

fn render_terminal_report(report: &AuditReport) {
    println!(
        "\n{}",
        "============================================================".bright_black()
    );
    println!(
        "{} {} (AppID: {})",
        "STEAM STOREFRONT AUDIT:".bold(),
        report.listing.name.bold().white(),
        report.listing.app_id.to_string().cyan()
    );
    println!(
        "Store URL:  {}",
        format!(
            "https://store.steampowered.com/app/{}/",
            report.listing.app_id
        )
        .underline()
    );
    println!(
        "Profile:    {} | Source: {} | Date: {}",
        report.audit_profile.as_str().cyan(),
        report.listing.source.as_str(),
        report.generated_at.format("%Y-%m-%d %H:%M UTC")
    );
    println!(
        "{}",
        "============================================================".bright_black()
    );

    println!(
        "Summary: {} critical | {} warnings | {} info | {} passed | {:.0}% completeness",
        report.summary.critical.to_string().red().bold(),
        report.summary.warnings.to_string().yellow().bold(),
        report.summary.info.to_string().blue(),
        report.summary.passed.to_string().green(),
        report.data_quality.data_completeness_pct
    );
    println!(
        "{}",
        "------------------------------------------------------------".bright_black()
    );

    println!("\n{}", "FINDINGS:".bold());
    for f in &report.findings {
        let (badge, title) = match f.severity {
            Severity::Critical => ("[CRITICAL]".red().bold(), f.title.red().bold()),
            Severity::Warning => ("[WARNING] ".yellow().bold(), f.title.yellow().bold()),
            Severity::Info => ("[INFO]    ".blue(), f.title.white()),
            Severity::Pass => ("[PASS]    ".green(), f.title.bright_black()),
        };

        if f.severity == Severity::Pass {
            println!(
                "  {} [{}] {}",
                badge,
                f.category.display_name().bright_black(),
                title
            );
        } else {
            println!(
                "\n  {} [{}] {}",
                badge,
                f.category.display_name().cyan(),
                title
            );
            println!("    {}", f.message);
            if let Some(actual) = &f.evidence.actual {
                println!("    Current value: {}", actual.to_string().bright_yellow());
            }
            println!(
                "    {} {}",
                "Recommendation:".bold(),
                f.recommendation.bright_cyan()
            );
        }
    }

    if !report.benchmarks.is_empty() {
        println!(
            "\n{}",
            "============================================================".bright_black()
        );
        println!("{}", "PEER BENCHMARKS:".bold());
        if let Some(cohort) = &report.cohort {
            println!(
                "Cohort: {} ({} games)",
                cohort.cohort_name.cyan(),
                cohort.total_games_in_cohort
            );
        }

        for b in &report.benchmarks {
            if let Some(dist) = &b.distribution {
                if b.status == BenchmarkStatus::Calculated {
                    let rank_str =
                        format!("{:.0}th percentile", dist.percentile_rank.unwrap_or(0.0));
                    println!(
                        "\n  * {:<32} | Your game: {:<14} | Rank: {:<16}",
                        b.metric_name.bold(),
                        b.listing_value_display.cyan(),
                        rank_str.yellow()
                    );
                    println!(
                        "    Median: {:.0} | IQR: [{:.0} - {:.0}] | Cohort sample: {} usable",
                        dist.median.unwrap_or(0.0),
                        dist.p25.unwrap_or(0.0),
                        dist.p75.unwrap_or(0.0),
                        dist.usable_records
                    );
                    if let Some(exp) = &b.explanation {
                        println!("    {}", exp.bright_black());
                    }
                }
            } else if let Some(cat) = &b.categorical {
                if b.status == BenchmarkStatus::Calculated {
                    println!(
                        "\n  * {:<32} | Your game: {:<14} | Adoption: {:.0}% ({}/{})",
                        b.metric_name.bold(),
                        b.listing_value_display.cyan(),
                        cat.true_percentage,
                        cat.true_count,
                        cat.usable_records
                    );
                }
            }
        }
    }

    println!(
        "\n{}",
        "============================================================".bright_black()
    );
    println!("{}", "Methodology: Benchmark distributions are descriptive references based on public Steam games. Listing completeness does not predict commercial revenue or review scores.".bright_black());
    println!(
        "{}",
        "============================================================".bright_black()
    );
}
