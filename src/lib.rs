use std::{
    collections::HashMap,
    ffi::CStr,
    fs::File,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, LazyLock, Mutex},
};

use flate2::read::GzDecoder;
use glib::MainLoop;
use libc::c_char;
use log::{info, warn};
type CpuId = (u8, u8, u8);
struct Config {
    log: Option<File>,
    debug: bool,
    monitor_ms: u64,
    config: PathBuf,
    force: bool,
}

impl Config {
    fn new() -> Self {
        Config {
            log: None,
            debug: false,
            monitor_ms: 1000,
            config: PathBuf::from("/etc/throttled.conf"),
            force: false,
        }
    }
}

// fn read_supported_cpus(x: &CPU_Id) -> Option<&'static str> {
static CPUMAP: LazyLock<HashMap<CpuId, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ((6, 26, 1), "Nehalem"),
        ((6, 26, 2), "Nehalem-EP"),
        ((6, 26, 4), "Bloomfield"),
        ((6, 28, 2), "Silverthorne"),
        ((6, 28, 10), "PineView"),
        ((6, 29, 0), "Dunnington-6C"),
        ((6, 29, 1), "Dunnington"),
        ((6, 30, 0), "Lynnfield"),
        ((6, 30, 5), "Lynnfield_CPUID"),
        ((6, 31, 1), "Auburndale"),
        ((6, 37, 2), "Clarkdale"),
        ((6, 38, 1), "TunnelCreek"),
        ((6, 39, 2), "Medfield"),
        ((6, 42, 2), "SandyBridge"),
        ((6, 42, 6), "SandyBridge"),
        ((6, 42, 7), "Sandy Bridge-DT"),
        ((6, 44, 1), "Westmere-EP"),
        ((6, 44, 2), "Gulftown"),
        ((6, 45, 5), "Sandy Bridge-EP"),
        ((6, 45, 6), "Sandy Bridge-E"),
        ((6, 46, 4), "Beckton"),
        ((6, 46, 5), "Beckton"),
        ((6, 46, 6), "Beckton"),
        ((6, 47, 2), "Eagleton"),
        ((6, 53, 1), "Cloverview"),
        ((6, 54, 1), "Cedarview-D"),
        ((6, 54, 9), "Centerton"),
        ((6, 55, 3), "Bay Trail-D"),
        ((6, 55, 8), "Silvermont"),
        ((6, 58, 9), "Ivy Bridge-DT"),
        ((6, 60, 3), "Haswell-DT"),
        ((6, 61, 4), "Broadwell-U"),
        ((6, 62, 3), "IvyBridgeEP"),
        ((6, 62, 4), "Ivy Bridge-E"),
        ((6, 63, 2), "Haswell-EP"),
        ((6, 69, 1), "HaswellULT"),
        ((6, 70, 1), "Crystal Well-DT"),
        ((6, 71, 1), "Broadwell-H"),
        ((6, 76, 3), "Braswell"),
        ((6, 77, 8), "Avoton"),
        ((6, 78, 3), "Skylake"),
        ((6, 79, 1), "BroadwellE"),
        ((6, 85, 4), "SkylakeXeon"),
        ((6, 85, 6), "CascadeLakeSP"),
        ((6, 85, 7), "CascadeLakeXeon2"),
        ((6, 86, 2), "BroadwellDE"),
        ((6, 86, 4), "BroadwellDE"),
        ((6, 87, 0), "KnightsLanding"),
        ((6, 87, 1), "KnightsLanding"),
        ((6, 90, 0), "Moorefield"),
        ((6, 92, 9), "Apollo Lake"),
        ((6, 93, 1), "SoFIA"),
        ((6, 94, 0), "Skylake"),
        ((6, 94, 3), "Skylake-S"),
        ((6, 95, 1), "Denverton"),
        ((6, 102, 3), "Cannon Lake-U"),
        ((6, 117, 10), "Spreadtrum"),
        ((6, 122, 1), "Gemini Lake-D"),
        ((6, 122, 8), "GoldmontPlus"),
        ((6, 126, 5), "IceLakeY"),
        ((6, 138, 1), "Lakefield"),
        ((6, 140, 1), "TigerLake-U"),
        ((6, 140, 2), "TigerLake-U"),
        ((6, 141, 1), "TigerLake-H"),
        ((6, 142, 9), "KabyLake"),
        ((6, 142, 10), "KabyLake"),
        ((6, 142, 11), "WhiskeyLake"),
        ((6, 142, 12), "CometLake-U"),
        ((6, 151, 2), "AlderLake-S/HX"),
        ((6, 151, 5), "AlderLake-S"),
        ((6, 154, 3), "AlderLake-P/H"),
        ((6, 154, 4), "AlderLake-U"),
        ((6, 156, 0), "JasperLake"),
        ((6, 158, 9), "KabyLakeG"),
        ((6, 158, 10), "CoffeeLake"),
        ((6, 158, 11), "CoffeeLake"),
        ((6, 158, 12), "CoffeeLake"),
        ((6, 158, 13), "CoffeeLake"),
        ((6, 165, 2), "CometLake"),
        ((6, 165, 4), "CometLake"),
        ((6, 165, 5), "CometLake-S"),
        ((6, 166, 0), "CometLake"),
        ((6, 167, 1), "RocketLake"),
        ((6, 170, 4), "MeteorLake"),
        ((6, 183, 1), "RaptorLake-HX"),
        ((6, 186, 2), "RaptorLake"),
        ((6, 186, 3), "RaptorLake-U"),
        ((6, 189, 1), "LunarLake"),
    ])
});
//     CPUMAP.get(x).copied()
// }

// fn msr_dict(x:&'static str) -> Option<u64> {
static MSR_DICT: LazyLock<HashMap<&'static str, u64>> = LazyLock::new(|| {
    HashMap::from([
        ("MSR_PLATFORM_INFO", 0xCE),
        ("MSR_OC_MAILBOX", 0x150),
        ("IA32_PERF_STATUS", 0x198),
        ("IA32_THERM_STATUS", 0x19C),
        ("MSR_TEMPERATURE_TARGET", 0x1A2),
        ("MSR_POWER_CTL", 0x1FC),
        ("MSR_RAPL_POWER_UNIT", 0x606),
        ("MSR_PKG_POWER_LIMIT", 0x610),
        ("MSR_INTEL_PKG_ENERGY_STATUS", 0x611),
        ("MSR_DRAM_ENERGY_STATUS", 0x619),
        ("MSR_PP1_ENERGY_STATUS", 0x641),
        ("MSR_CONFIG_TDP_CONTROL", 0x64B),
        ("IA32_HWP_REQUEST", 0x774),
    ])
});
//     MSR_DICT.get(x).copied()
// }

// fn msr_dict(x:&'static str) -> Option<u64> {
//     static MSR_DICT : LazyLock<HashMap<&'static str, u64>> = LazyLock::new(|| HashMap::from([
//     ]));
//     MSR_DICT.get(x).copied()
// }
// fn voltage_planes(x:&'static str) -> Option<u64> {
static VOLTAGE_PLANES: LazyLock<HashMap<&'static str, u64>> = LazyLock::new(|| {
    HashMap::from([
        ("CORE", 0),
        ("GPU", 1),
        ("CACHE", 2),
        ("UNCORE", 3),
        ("ANALOGIO", 4),
    ])
});
//     VOLTAGE_PLANES.get(x).copied()
// }
// fn current_planes(x:&'static str) -> Option<u64> {
static CURRENT_PLANES: LazyLock<HashMap<&'static str, u64>> =
    LazyLock::new(|| HashMap::from([("CORE", 0), ("GPU", 1), ("CACHE", 2)]));
//     CURRENT_PLANES.get(x).copied()
// }

const TRIP_TEMP_RANGE: (i32, i32) = (40, 97);
const UNDERVOLT_KEYS: (&str, &str, &str) = ("UNDERVOLT", "UNDERVOLT.AC", "UNDERVOLT.BATTERY");
const ICCMAX_KEYS: (&str, &str, &str) = ("ICCMAX", "ICCMAX.AC", "ICCMAX.BATTERY");
const HWP_PERFOLRMANCE_VALUE: i32 = 0x20;
const HWP_DEFAULT_VALUE: i32 = 0x80;
const HWP_INTERVAL: i32 = 60;

fn parse_args() -> Config {
    Config::new()
}

fn set_msr_allow_writes() {
    if !Path::new("/sys/module/msr").exists()
        && !Command::new("modprobe")
            .arg("msr")
            .status()
            .is_ok_and(|exit| exit.success())
    {
        warn!("cannot load msr module");
        return;
    }
    let p = Path::new("/sys/module/msr/parameters/allow_writes");
    if let Ok(mut fd) = File::open(p) {
        if let Err(e) = fd.write(b"on") {
            warn!("Unable to set MSR allow_writes to on. You might experience warnings in kernel logs. {:?}", e);
        }
    }
}

fn test_msr_rw_capabilities(
    test_msr: Arc<Mutex<bool>>,
    unsupported_features: &mut Vec<&'static str>,
) {
    if let Ok(mut data) = test_msr.lock() {
        *data = true;
    }
    info!("Testing if undervolt is supported...");
    let res = get_undervolt(unsupported_features, None, false, test_msr.clone());
    if res.is_err() {
        warn!("Undervolt seems not to be supported on your system, disabling.");
        unsupported_features.push("UNDERVOLT");
    }
    if let Ok(mut data) = test_msr.lock() {
        *data = false;
    }
}

fn get_power_source() -> ! {
    todo!()
}

fn get_platform_info() -> ! {
    todo!()
}

fn parse_config() -> ! {
    todo!()
}

fn get_reg_values() -> ! {
    todo!()
}

fn get_undervolt(
    unsupported_features: &Vec<&'static str>,
    plane: Option<&'static str>,
    convert: bool,
    test_msr: Arc<Mutex<bool>>,
) -> Result<HashMap<&'static str, i64>, String> {
    if !unsupported_features.contains(&"UNDERVOLT") {
        return Err("Undervolt is not supported".to_owned());
    }
    let mut out = HashMap::new();
    let planes = plane
        .map(|x| {
            VOLTAGE_PLANES
                .get_key_value(x)
                .map(|(k, v)| HashMap::from([(*k, *v)]))
                .unwrap_or_else(|| {
                    fatal(&format!("Error: plane {} not found in VOLTAGE_PLANES", x))
                })
        })
        .unwrap_or(VOLTAGE_PLANES.clone());
    for (k, v) in planes {
        writemsr("MSR_OC_MAILBOX", 0x8000001000000000 | (v << 40));
        let read_result = readmsr_flat("MSR_OC_MAILBOX", None, None);
        let read_value = match read_result {
            Ok(value) => value & 0xFFFFFFFF,
            Err(e) => {
                if test_msr.lock().is_ok_and(|x| *x) {
                    return Err(format!("raise e: mutex taken {}", e));
                } else {
                    match e.kind() {
                        io::ErrorKind::PermissionDenied => fatal(&format!(
                            "Unable to read {}. Try to disable Secure Boot.",
                            e
                        )),
                        _ => return Err(format!("Unknown error while reading msr: {}", e)),
                    }
                };
            }
        };
        let val = if convert {
            calc_undervolt_mv(read_value)
        } else {
            read_value as i64
        };
        out.insert(k, val);
    }
    Ok(out)
}

fn calc_undervolt_mv(read_value: u64) -> i64 {
    let offset = ((read_value & 0xFFE00000) >> 21).try_into().unwrap();
    let res: i32 = if offset <= 0x400 {
        offset
    } else {
        -(0x800 - offset)
    };
    ((res as f64) / 1.024).round() as i64
}

fn readmsr_flat(arg: &str, from: Option<usize>, to: Option<usize>) -> Result<u64, io::Error> {
    let from = from.unwrap_or(0);
    let to = to.unwrap_or(63);
    assert!(from < to);
    assert!(to <= 63);
    // assert!(cpu.is_none_or(|x| (0..cpu_count()).contains(&x)));
    let cpu_count = cpu_count();
    let mut msr_list = (0..cpu_count)
        .map(|i| format!("/dev/cpu/{}/msr", i))
        .peekable();
    if let Some(cpu_zero) = msr_list.peek() {
        if !Path::new(cpu_zero).exists() {
            let is_msr_loaded = Command::new("modprobe")
                .arg("msr")
                .status()
                .is_ok_and(|exit| exit.success());
            if !is_msr_loaded {
                fatal("Unable to load the msr module.");
            }
        }
    }
    let mut msr_values = Vec::with_capacity(cpu_count);
    let mut buffer: [u8; 8] = [0; 8];
    let arg_addr = *MSR_DICT.get(arg).unwrap();
    for path in msr_list {
        let mut fh = File::open(path)?;
        let _ = fh.seek(io::SeekFrom::Start(arg_addr))?;
        fh.read_exact(&mut buffer)?;
        msr_values.push(u64::from_le_bytes(buffer));
    }
    match msr_values.as_slice() {
        [head, tail @ ..] => {
            if !tail.iter().all(|x| x == head) {
                warn!(
                    "multiple values for {} ({:x}) found. This should never happen.",
                    arg, arg_addr
                );
            }
            Ok(*head)
        }
        [] => fatal("No msr values found"),
    }
}

fn cpu_count() -> usize {
    num_cpus::get()
}

fn writemsr(arg: &str, plane: u64) {
    todo!()
}

fn set_icc_max() -> ! {
    todo!()
}

fn set_hwp() -> ! {
    todo!()
}

// TODO
fn fatal(msg: &str) -> ! {
    panic!("{}", msg);
}

/// Represents the information obtained from the `uname` system call.
///
/// Corresponds to the `struct utsname` in C.
#[derive(Debug)]
pub struct UnameInfo {
    pub sysname: String,     // Operating system name (e.g., "Linux", "Darwin")
    pub nodename: String,    // Network node hostname (e.g., "my-laptop")
    pub pub_release: String, // Operating system release (e.g., "5.15.0-76-generic")
    pub version: String, // Operating system version (e.g., "#83~22.04.1-Ubuntu SMP PREEMPT_DYNAMIC...")
    pub machine: String, // Hardware identifier (e.g., "x86_64", "arm64")
    // domainname is present on some systems (like Linux) but not others (like macOS).
    // We'll handle it conditionally.
    pub domainname: Option<String>,
}

/// Helper function to convert a null-terminated C char array to a Rust String.
fn c_char_array_to_string(arr: &[c_char]) -> io::Result<String> {
    // Safety: CStr::from_ptr expects a null-terminated string.
    // We assume the C array is null-terminated as per uname() contract.
    let c_str = unsafe { CStr::from_ptr(arr.as_ptr()) };
    c_str.to_str().map(|s| s.to_owned()).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid UTF-8 in uname string: {}", e),
        )
    })
}

/// Retrieves system information using the `uname` system call.
///
/// Returns a `Result` containing `UnameInfo` on success or an `io::Error` on failure.
pub fn get_uname_info() -> io::Result<UnameInfo> {
    // Create an uninitialized utsname struct.
    // std::mem::zeroed is safe here because uname() will fully initialize it.
    let mut info = unsafe { std::mem::zeroed::<libc::utsname>() };

    let result = unsafe {
        // Call the uname system function.
        // It takes a mutable pointer to the utsname struct.
        libc::uname(&mut info)
    };

    if result < 0 {
        // If uname returns a negative value, an error occurred.
        // Get the last OS error (errno) and convert it to an io::Error.
        return Err(io::Error::last_os_error());
    }

    // Convert each C string field to a Rust String.
    let sysname = c_char_array_to_string(&info.sysname)?;
    let nodename = c_char_array_to_string(&info.nodename)?;
    let release = c_char_array_to_string(&info.release)?;
    let version = c_char_array_to_string(&info.version)?;
    let machine = c_char_array_to_string(&info.machine)?;

    // The 'domainname' field is platform-specific and not always present
    // or filled. It's typically part of `utsname` on Linux, but not macOS.
    let domainname = {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            // On Linux/Android, utsname has a domainname field.
            let domain_str = c_char_array_to_string(&info.domainname)?;
            if domain_str.is_empty() {
                None
            } else {
                Some(domain_str)
            }
        }
    };

    Ok(UnameInfo {
        sysname,
        nodename,
        pub_release: release,
        version,
        machine,
        domainname,
    })
}

fn check_kernel() {
    if unsafe { libc::geteuid() } != 0 {
        fatal("No root no party. Try again with sudo.");
    }
    let kernel_config = get_uname_info()
        .and_then(|info| {
            let path_str = format!("/boot/config-{}", info.pub_release);
            let mut file = File::open(Path::new(&path_str))?;
            let mut content = String::new();
            file.read_to_string(&mut content).map(|_| content)
        })
        .or_else(|_| {
            let proc_config = File::open("/proc/config.gz")?;
            let mut proc_gz = GzDecoder::new(proc_config);
            let mut buf = String::new();
            Command::new("modprobe").arg("configs").status()?;
            proc_gz.read_to_string(&mut buf).map(|_| buf)
        });
    let data =
        kernel_config.unwrap_or_else(|_| fatal("Unable to obtain and validate kernel config."));

    if !data.contains("CONFIG_DEVMEM=y") {
        warn!("Bad kernel config: you need CONFIG_DEVMEM=y");
    }
    if !data.contains("CONFIG_X86_MSR=y") && !data.contains("CONFIG_X86_MSR=m") {
        fatal("Bad kernel config: you need CONFIG_X86_MSR builtin or as module.");
    }
}

fn check_cpu() -> Option<CpuId> {
    let mut f =
        File::open("/proc/cpuinfo").unwrap_or_else(|_| fatal("Unable to identify CPU model."));
    let mut buf = String::new();
    f.read_to_string(&mut buf).ok()?;
    let cpuinfo: HashMap<&str, &str> = buf.lines().flat_map(|l| l.split_once(':')).collect();
    if cpuinfo
        .get("vendor_id")
        .is_none_or(|v| *v != "GenuineIntel")
    {
        fatal("This tool is designed for Intel CPUs only.");
    }
    let cpu_family = cpuinfo.get("cpu family")?.parse().ok()?;
    let model = cpuinfo.get("model")?.parse().ok()?;
    let stepping = cpuinfo.get("stepping")?.parse().ok()?;
    Some((cpu_family, model, stepping))
}

pub fn calc_icc_max_msr(plane: &str, current: f64) -> u64 {
    let plane_idx = match plane {
        "CORE" => 0,
        "GPU" => 1,
        "CACHE" => 2,
        _ => panic!("Invalid plane"),
    };
    let current_val = (current * 4.0).round() as u64;
    0x8000001700000000 | (plane_idx << 40) | current_val
}

pub fn calc_time_window_vars(t: f64, time_unit: f64) -> (u64, u64) {
    for y in 0..32 {
        for z in 0..4 {
            let val = (2.0_f64.powi(y as i32)) * (1.0 + (z as f64) / 4.0) * time_unit;
            if t <= val {
                return (y, z);
            }
        }
    }
    panic!("No window found");
}

pub fn main_loop() {
    let args = parse_args();

    let cpuid: Option<CpuId> = if !args.force {
        check_kernel();
        Some(check_cpu().unwrap_or_else(|| fatal("Unable to identify CPU model.")))
    } else {
        None
    };
    let family_name = cpuid.and_then(|x| CPUMAP.get(&x));
    match family_name {
        Some(name) => info!("Detected CPU architecture: Intel {}", name),
        None => fatal("Your CPU model is not supported."),
    };

    set_msr_allow_writes();
    let test_msr = Arc::new(Mutex::new(false));
    let mut unsupported_features: Vec<&'static str> = vec![];
    test_msr_rw_capabilities(test_msr, &mut unsupported_features);
    // dbus stuff
    let power_source = get_power_source();
    let platform_info = get_platform_info();
    let config = parse_config();
    let regs = get_reg_values();

    let _ = get_undervolt(&unsupported_features, None, false, test_msr);
    set_icc_max();
    set_hwp();

    // start glib loop
    let main_loop = MainLoop::new(None, false);
    main_loop.run();
}
