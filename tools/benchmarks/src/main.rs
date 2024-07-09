use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bzip2::read::BzDecoder;
use dump_table::get_table_for_debug_name_and_id;
use flate2::read::GzDecoder;
use query_api::{query_api, DebugId};
use tar::Archive;
use tempfile::tempdir;

fn main() -> anyhow::Result<()> {
    prepare(
        big_fixtures_dir().join("win64-ci").join("xul.pdb"),
        "https://storage.googleapis.com/profiler-get-symbols-fixtures/win64-ci-xul.pdb.gz",
        FileType::Gzip,
    )?;
    prepare(
        big_fixtures_dir().join("win64-local").join("xul.pdb"),
        "https://storage.googleapis.com/profiler-get-symbols-fixtures/win64-local-xul.pdb.gz",
        FileType::Gzip,
    )?;
    prepare(
        big_fixtures_dir().join("macos-ci").join("XUL.dSYM"),
        "https://storage.googleapis.com/profiler-get-symbols-fixtures/macos-ci-XUL.dSYM.tar.bz2",
        FileType::TarBz2,
    )?;
    prepare(
        big_fixtures_dir().join("macos-local"),
        "https://storage.googleapis.com/profiler-get-symbols-fixtures/macos-local.tar.bz2",
        FileType::TarBz2,
    )?;
    prepare(
        big_fixtures_dir().join("linux64-ci").join("libxul.so.dbg"),
        "https://storage.googleapis.com/profiler-get-symbols-fixtures/linux64-ci-libxul.so.dbg.gz",
        FileType::Gzip,
    )?;
    prepare(
        big_fixtures_dir()
            .join("android32-ci")
            .join("libxul.so.dbg"),
            "https://storage.googleapis.com/profiler-get-symbols-fixtures/android32-ci-libxul.so.dbg.gz",
        FileType::Gzip,
    )?;
    prepare(
        big_fixtures_dir()
            .join("android64-ci")
            .join("libxul.so.dbg"),
            "https://storage.googleapis.com/profiler-get-symbols-fixtures/android64-ci-libxul.so.dbg.gz",
        FileType::Gzip,
    )?;

    let timings = vec![
        // Windows
        Timing {
            platform: "win64",
            action: "dump-table",
            duration: run_dump_table_benchmark(
                "xul.pdb",
                Some("4C8C9680FAECFDC64C4C44205044422E1".into()),
                big_fixtures_dir().join("win64-ci"),
            ),
        },
        Timing {
            platform: "win64",
            action: "query-api v5",
            duration: run_api_query_benchmark(
                "/symbolicate/v5",
                &fixtures_dir().join("requests").join("win64-ci-xul.json"),
                big_fixtures_dir().join("win64-ci"),
            )?,
        },
        Timing {
            platform: "win64-local",
            action: "dump-table",
            duration: run_dump_table_benchmark(
                "xul.pdb",
                Some("518A025063D22EEA4C4C44205044422E1".into()),
                big_fixtures_dir().join("win64-local"),
            ),
        },
        Timing {
            platform: "win64-local",
            action: "query-api v5",
            duration: run_api_query_benchmark(
                "/symbolicate/v5",
                &fixtures_dir().join("requests").join("win64-local-xul.json"),
                big_fixtures_dir().join("win64-local"),
            )?,
        },
        // macOS
        Timing {
            platform: "macos",
            action: "dump-table",
            duration: run_dump_table_benchmark(
                "XUL",
                Some("D2139EE3190B37028A98D55519AA0B870".into()),
                big_fixtures_dir().join("macos-ci"),
            ),
        },
        Timing {
            platform: "macos",
            action: "query-api v5",
            duration: run_api_query_benchmark(
                "/symbolicate/v5",
                &fixtures_dir().join("requests").join("macos-ci-xul.json"),
                big_fixtures_dir().join("macos-ci"),
            )?,
        },
        // macOS, local with object files referenced by OSO stab symbols
        Timing {
            platform: "macos-local",
            action: "dump-table",
            duration: run_dump_table_benchmark(
                "XUL",
                Some("8A1F954DAA533D219943F2FBC8E6FA2E0".into()),
                big_fixtures_dir().join("macos-local"),
            ),
        },
        Timing {
            platform: "macos-local",
            action: "query-api v5",
            duration: run_api_query_benchmark(
                "/symbolicate/v5",
                &fixtures_dir().join("requests").join("macos-local-xul.json"),
                big_fixtures_dir().join("macos-local"),
            )?,
        },
        // Linux
        Timing {
            platform: "linux64",
            action: "dump-table",
            duration: run_dump_table_benchmark(
                "libxul.so",
                Some("F33E37832964290A31906802CE8F3C9C0".into()),
                big_fixtures_dir().join("linux64-ci"),
            ),
        },
        Timing {
            platform: "linux64",
            action: "query-api v5",
            duration: run_api_query_benchmark(
                "/symbolicate/v5",
                &fixtures_dir().join("requests").join("linux64-ci-xul.json"),
                big_fixtures_dir().join("linux64-ci"),
            )?,
        },
        // Android 32 bit
        Timing {
            platform: "android32",
            action: "dump-table",
            duration: run_dump_table_benchmark(
                "libxul.so",
                Some("CA89B171348FDEF3A6A365AC6CDF07BF0".into()),
                big_fixtures_dir().join("android32-ci"),
            ),
        },
        Timing {
            platform: "android32",
            action: "query-api v5",
            duration: run_api_query_benchmark(
                "/symbolicate/v5",
                &fixtures_dir()
                    .join("requests")
                    .join("android32-ci-xul.json"),
                big_fixtures_dir().join("android32-ci"),
            )?,
        },
        // Android 64 bit
        Timing {
            platform: "android64",
            action: "dump-table",
            duration: run_dump_table_benchmark(
                "libxul.so",
                Some("B560E04259EBFBB96D6D6BB5D69F0DCE0".into()),
                big_fixtures_dir().join("android64-ci"),
            ),
        },
        Timing {
            platform: "android64",
            action: "query-api v5",
            duration: run_api_query_benchmark(
                "/symbolicate/v5",
                &fixtures_dir()
                    .join("requests")
                    .join("android64-ci-xul.json"),
                big_fixtures_dir().join("android64-ci"),
            )?,
        },
    ];

    eprintln!();
    eprintln!("Results:");
    for Timing {
        platform,
        action,
        duration,
    } in timings
    {
        eprintln!("  - {platform:12} {action:16} {duration:?}");
    }

    Ok(())
}

struct Timing {
    platform: &'static str,
    action: &'static str,
    duration: Duration,
}

fn run_api_query_benchmark(
    url: &str,
    request_json_filename: &Path,
    symbol_directory: PathBuf,
) -> anyhow::Result<Duration> {
    eprintln!("Starting query API benchmark for {url}, {request_json_filename:?}.");
    let request_json = std::fs::read_to_string(request_json_filename)?;
    let start = Instant::now();
    let _result = futures::executor::block_on(query_api(url, &request_json, symbol_directory));
    let duration = start.elapsed();
    eprintln!("Finished query API benchmark for {url}, {request_json_filename:?}.");
    eprintln!("Elapsed time: {duration:?}");
    Ok(duration)
}

fn run_dump_table_benchmark(
    debug_name: &str,
    breakpad_id: Option<String>,
    symbol_directory: PathBuf,
) -> std::time::Duration {
    let debug_id = breakpad_id
        .as_deref()
        .and_then(|debug_id| DebugId::from_breakpad(debug_id).ok());
    eprintln!(
        "Starting dump_table benchmark for {debug_name}, {breakpad_id:?}, {symbol_directory:?}."
    );
    let start = Instant::now();
    let _result = futures::executor::block_on(get_table_for_debug_name_and_id(
        debug_name,
        debug_id,
        symbol_directory.clone(),
    ))
    .unwrap();
    let duration = start.elapsed();
    eprintln!(
        "Finished dump_table benchmark for {debug_name}, {breakpad_id:?}, {symbol_directory:?}."
    );
    eprintln!("Elapsed time: {duration:?}");
    duration
}

fn fixtures_dir() -> PathBuf {
    let this_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    this_dir.join("..").join("..").join("fixtures")
}

fn big_fixtures_dir() -> PathBuf {
    let this_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    this_dir
        .join("..")
        .join("..")
        .join("big-benchmark-fixtures")
}

enum FileType {
    #[allow(unused)]
    CabArchive,
    Gzip,
    TarBz2,
}

fn prepare(local_path: PathBuf, download_url: &str, ftype: FileType) -> anyhow::Result<()> {
    if fs::metadata(&local_path).is_ok() {
        // Path exists.
        return Ok(());
    };

    let fname = local_path.file_name().unwrap();

    let client = reqwest::blocking::Client::builder().timeout(None).build()?;
    eprint!("Downloading {download_url} into memory...");
    let response = client.get(download_url).send()?.bytes()?;
    eprintln!(" done.");
    let dir = tempdir()?;

    let temp_file_path = dir.path().join(fname);
    match &ftype {
        FileType::CabArchive => {
            let cursor = std::io::Cursor::new(&response);
            let mut cabinet = cab::Cabinet::new(cursor)?;
            let file_name_in_cab = {
                // Only pick the first file we encounter. That's the PDB.
                let folder = cabinet.folder_entries().next().unwrap();
                let file = folder.file_entries().next().unwrap();
                file.name().to_string()
            };
            eprint!("Extracting {file_name_in_cab:?} to {temp_file_path:?}...");
            let mut reader = cabinet.read_file(&file_name_in_cab).unwrap();
            let mut file = File::create(&temp_file_path)?;
            std::io::copy(&mut reader, &mut file).unwrap();
            eprintln!(" done.");
        }
        FileType::Gzip => {
            eprint!("Extracting contents to {temp_file_path:?}...");
            let cursor = std::io::Cursor::new(&response);
            let mut reader = GzDecoder::new(cursor);
            let mut file = File::create(&temp_file_path)?;
            std::io::copy(&mut reader, &mut file).unwrap();
            eprintln!(" done.");
        }
        FileType::TarBz2 => {
            let dir_path = dir.path();
            eprint!("Extracting contents to {dir_path:?}...");
            let cursor = std::io::Cursor::new(&response);
            let tar = BzDecoder::new(cursor);

            // .dSYM archives look like files in Finder, but they're actually
            // packages with a directory structure. Extract all files and
            // directories, and then make sure the root directory of that
            // structure is what we expect.
            let mut archive = Archive::new(tar);
            let mut roots: HashSet<OsString> = HashSet::new();
            for mut entry in (archive.entries()?).flatten() {
                let path = entry.path()?;
                let root = path.components().next().unwrap();
                if let std::path::Component::Normal(root) = root {
                    roots.insert(root.into());
                } else {
                    panic!("weird path component in bz2: {root:?}");
                }
                entry.unpack_in(&dir)?;
            }
            eprintln!(" done.");
            // This created a directory structure. Make sure that there's only
            // one root directory, and that its name is the name we expect (fname).
            assert_eq!(roots.len(), 1);
            let root = roots.iter().next().unwrap();
            assert_eq!(root, fname)
        }
    };
    eprint!("Moving {temp_file_path:?} to {local_path:?}...");
    fs::create_dir_all(local_path.parent().unwrap())?;
    fs::rename(temp_file_path, local_path)?;
    drop(dir);
    eprintln!(" done.");

    Ok(())
}
