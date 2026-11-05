fn main() {
    emit_storage_layout();
    emit_build_time();
    emit_build_version();
    linker_be_nice();
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

/// Splits the `nvs` partition of partition-table.csv into page and schedule storage,
/// so the flash addresses follow the partition table instead of being hard-coded.
fn emit_storage_layout() {
    const TABLE: &str = "partition-table.csv";
    const SECTOR: u32 = 0x1000;
    println!("cargo:rerun-if-changed={TABLE}");

    let table = std::fs::read_to_string(TABLE).unwrap_or_else(|e| panic!("reading {TABLE}: {e}"));
    let fields: Vec<&str> = table
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split(',').map(str::trim).collect::<Vec<_>>())
        .find(|fields| fields.first() == Some(&"nvs"))
        .unwrap_or_else(|| panic!("{TABLE} has no `nvs` partition"));
    let offset = parse_size(fields[3]);
    let half = parse_size(fields[4]) / 2;

    // sequential-storage works on whole flash sectors and needs at least two per region.
    assert!(
        offset % SECTOR == 0 && half % SECTOR == 0 && half >= 2 * SECTOR,
        "{TABLE}: `nvs` must be sector aligned and hold two regions of at least two sectors"
    );

    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("storage_layout.rs");
    std::fs::write(
        out,
        format!(
            "pub const PAGE_STORAGE_BEGIN: u32 = {offset:#x};\n\
             pub const PAGE_STORAGE_SIZE: u32 = {half:#x};\n\
             pub const SCHEDULE_STORAGE_BEGIN: u32 = {:#x};\n\
             pub const SCHEDULE_STORAGE_SIZE: u32 = {half:#x};\n",
            offset + half
        ),
    )
    .unwrap();
}

/// Parses a partition table number: "0x9000", "36864", "24K" or "1M".
fn parse_size(value: &str) -> u32 {
    let (digits, scale) = match value.as_bytes().last() {
        Some(b'K' | b'k') => (&value[..value.len() - 1], 1024),
        Some(b'M' | b'm') => (&value[..value.len() - 1], 1024 * 1024),
        _ => (value, 1),
    };
    let number = match digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => digits.parse(),
    };
    number.unwrap_or_else(|_| panic!("invalid number `{value}` in partition table")) * scale
}

fn emit_build_version() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    println!("cargo:rerun-if-env-changed=FORCE_VERSION_REBUILD");

    let hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let version = if dirty { format!("{hash}-dirty") } else { hash };

    println!("cargo:rustc-env=BUILD_VERSION={version}");
}

fn emit_build_time() {
    let now = time::OffsetDateTime::now_utc();
    println!(
        "cargo:rustc-env=BUILD_TIME={:02}:{:02}:{:02}",
        now.hour(),
        now.minute(),
        now.second()
    );
    println!(
        "cargo:rustc-env=BUILD_DATE={}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    );
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!(
                        "💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`"
                    );
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
