use std::{fs::File, io::Write, path::Path, process::Command};

use glib::MainLoop;

struct Config {}
fn main() {
    let args = parse_args();

    // let cpuid = if !args.force {
    //     check_kernel();
    //     check_cpu()
    // } else {
    //     None
    // };

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

fn parse_args() -> ! {
    todo!()
}

fn set_msr_allow_writes() {
    if !Path::new("/sys/module/msr").exists() {
        if !Command::new("modprobe")
            .arg("msr")
            .status()
            .map_or(false, |exit| exit.success())
        {
            eprintln!("cannot load msr module");
            return;
        }
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

fn check_kernel() -> ! {
    todo!()
}
