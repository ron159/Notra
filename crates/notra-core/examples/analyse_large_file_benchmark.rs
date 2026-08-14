use std::fs::{self, File};
use std::hint::black_box;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use memmap2::MmapOptions;
use notra_core::analyse::{
    AnalyseEngine, AnalysePattern, AnalyseRunInput, AnalyseSearchType, AnalyseSelection,
    CancellationToken,
};

const MIB: u64 = 1024 * 1024;
const RECORD_BYTES: usize = 128;
const RECORDS_PER_BLOCK: usize = 8192;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Arguments::parse()?;
    let generated = arguments.path.is_none();
    let path = arguments.path.unwrap_or_else(|| {
        std::env::temp_dir().join(format!(
            "notra-analyse-benchmark-{}-{}mb.txt",
            std::process::id(),
            arguments.size_mb
        ))
    });
    if generated {
        write_fixture(&path, arguments.size_mb * MIB)?;
    }

    let metadata = fs::metadata(&path)?;
    let started = Instant::now();
    let (load_ms, analyse_ms, total_matches, result_lines) = match arguments.mode.as_str() {
        "owned" => run_owned(&path)?,
        "mmap" => run_mmap(&path)?,
        mode => return Err(format!("unsupported mode: {mode}").into()),
    };
    let elapsed_ms = started.elapsed().as_millis();
    println!(
        "{{\"mode\":\"{}\",\"bytes\":{},\"load_ms\":{},\"analyse_ms\":{},\"elapsed_ms\":{},\"total_matches\":{},\"result_lines\":{}}}",
        arguments.mode,
        metadata.len(),
        load_ms,
        analyse_ms,
        elapsed_ms,
        total_matches,
        result_lines
    );

    if generated && !arguments.keep {
        fs::remove_file(path)?;
    }
    Ok(())
}

struct Arguments {
    size_mb: u64,
    mode: String,
    path: Option<PathBuf>,
    keep: bool,
}

impl Arguments {
    fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let mut size_mb = 100;
        let mut mode = "owned".to_owned();
        let mut path = None;
        let mut keep = false;
        let mut arguments = std::env::args().skip(1);
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--size-mb" => {
                    size_mb = arguments
                        .next()
                        .ok_or("--size-mb requires a value")?
                        .parse()?;
                }
                "--mode" => mode = arguments.next().ok_or("--mode requires a value")?,
                "--path" => {
                    path = Some(PathBuf::from(
                        arguments.next().ok_or("--path requires a value")?,
                    ));
                }
                "--keep" => keep = true,
                value => return Err(format!("unknown argument: {value}").into()),
            }
        }
        if size_mb == 0 || size_mb > 1024 {
            return Err("--size-mb must be between 1 and 1024".into());
        }
        Ok(Self {
            size_mb,
            mode,
            path,
            keep,
        })
    }
}

fn write_fixture(path: &Path, target_bytes: u64) -> std::io::Result<()> {
    let mut normal = vec![b'x'; RECORD_BYTES];
    normal[RECORD_BYTES - 1] = b'\n';
    normal[..23].copy_from_slice(b"INFO code=1234 payload=");
    let mut error = normal.clone();
    error[..24].copy_from_slice(b"ERROR code=9001 payload=");

    let mut block = Vec::with_capacity(RECORD_BYTES * RECORDS_PER_BLOCK);
    block.extend_from_slice(&error);
    for _ in 1..RECORDS_PER_BLOCK {
        block.extend_from_slice(&normal);
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);
    let full_blocks = target_bytes / block.len() as u64;
    for _ in 0..full_blocks {
        writer.write_all(&block)?;
    }
    let remaining = (target_bytes % block.len() as u64) as usize;
    writer.write_all(&block[..remaining])?;
    writer.flush()
}

fn run_owned(path: &Path) -> Result<(u128, u128, usize, usize), Box<dyn std::error::Error>> {
    let load_started = Instant::now();
    let decoded = notra_core::fs::read_text(path)?;
    let load_ms = load_started.elapsed().as_millis();
    let (analyse_ms, matches, lines) = run_engine(&decoded.text)?;
    Ok((load_ms, analyse_ms, matches, lines))
}

fn run_mmap(path: &Path) -> Result<(u128, u128, usize, usize), Box<dyn std::error::Error>> {
    let load_started = Instant::now();
    let file = File::open(path)?;
    // SAFETY: the benchmark owns the generated fixture and never mutates it while mapped.
    let mapped = unsafe { MmapOptions::new().map(&file)? };
    let text = std::str::from_utf8(&mapped)?;
    let load_ms = load_started.elapsed().as_millis();
    let (analyse_ms, matches, lines) = run_engine(text)?;
    Ok((load_ms, analyse_ms, matches, lines))
}

fn run_engine(text: &str) -> Result<(u128, usize, usize), Box<dyn std::error::Error>> {
    let patterns = vec![
        AnalysePattern {
            id: 1,
            search_text: "ERROR".to_owned(),
            match_case: true,
            selection: AnalyseSelection::Line,
            ..Default::default()
        },
        AnalysePattern {
            id: 2,
            search_text: r"(?<=code=)9001".to_owned(),
            search_type: AnalyseSearchType::Regex,
            match_case: true,
            selection: AnalyseSelection::Text,
            ..Default::default()
        },
    ];
    let engine = AnalyseEngine::default();
    let analyse_started = Instant::now();
    let result = engine.run(
        AnalyseRunInput {
            run_id: 1,
            document_id: 1,
            document_revision: 1,
            pattern_revision: 1,
            text,
            patterns: &patterns,
        },
        &CancellationToken::new(),
    )?;
    let analyse_ms = analyse_started.elapsed().as_millis();
    Ok((
        analyse_ms,
        black_box(result.total_matches),
        black_box(result.lines.len()),
    ))
}
