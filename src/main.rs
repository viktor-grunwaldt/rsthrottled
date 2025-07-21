use std::{
    ffi::CStr,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use flate2::read::GzDecoder;
use glib::MainLoop;
use libc::c_char;
use log::warn;

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

fn main() {
    let args = parse_args();

    let cpuid: Option<u8> = if !args.force {
        check_kernel();
        check_cpu();
    } else {
        None
    };

    set_msr_allow_writes();
    test_msr_rw_capabilities();
    // dbus stuff
    let power_source = get_power_source();
    let platform_info = get_platform_info();
    let config = parse_config();
    let regs = get_reg_values();

    undervolt();
    set_icc_max();
    set_hwp();

    // start glib loop
    let main_loop = MainLoop::new(None, false);
    main_loop.run();
}

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
        eprintln!("cannot load msr module");
        return;
    }
    let p = Path::new("/sys/module/msr/parameters/allow_writes");
    if let Ok(mut fd) = File::open(p) {
        if let Err(e) = fd.write(b"on") {
            eprintln!("{:?}", e);
        }
    }
}

fn test_msr_rw_capabilities() -> ! {
    todo!()
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

fn undervolt() -> ! {
    todo!()
}

fn set_icc_max() -> ! {
    todo!()
}

fn set_hwp() -> ! {
    todo!()
}

fn check_cpu() -> ! {
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
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            // On other platforms, the field might not exist or be relevant.
            // We just return None.
            None
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
            Command::new("modprobe").arg("configs").status();
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
