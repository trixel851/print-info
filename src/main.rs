use std::ffi::CStr;
use std::fs;
use sysinfo;

#[inline]
fn cstr_to_str(ptr: *const core::ffi::c_char) -> String {
    unsafe {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

fn get_shell() -> String {
    unsafe {
        let ptr = libc::getenv(c"SHELL".as_ptr());
        if ptr.is_null() {
            return "Unknown".to_string();
        }
        cstr_to_str(ptr)
    }
}

fn get_editor() -> String {
    unsafe {
        let ptr = libc::getenv(c"EDITOR".as_ptr());
        if ptr.is_null() {
            return "".to_string();
        }
        cstr_to_str(ptr)
    }
}

#[inline]
fn print_shell() {
    println!("Shell:        {}", get_shell());
}

#[inline]
fn print_editor()  {
    println!("Editor:       {}", get_editor());
}

#[inline]
fn read_entire_file(path: &str) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .expect("Unknown")
}

fn print_uptime() {
    let uptime_secs = sysinfo::System::uptime();
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    let seconds = uptime_secs % 60;
    println!("Uptime:       {}d {}h {}m {}s", days, hours, minutes, seconds);
}

fn print_graphics_info(adapter: wgpu::Adapter) {
    use wgpu::DeviceType::*;
    let info = adapter.get_info();
    println!("GPU Name:     {}", info.name);
    println!("GPU vendor:   {}", info.vendor);
    let device_type = match info.device_type {
        DiscreteGpu => "Discrete",
        IntegratedGpu => "Integrated",
        VirtualGpu => "Virtual",
        Cpu => "Cpu",
        Other => "Other",
    };
    println!("Device type:  {}", device_type);
}

fn print_available_memory(sys: &mut sysinfo::System) {
    println!("RAM Capacity: {} MiB", sys.total_memory() / 1024 / 1024);
    println!("Usable RAM:   {} MiB", sys.available_memory() / 1024 / 1024);
}

fn print_motherboard_info() {
    let board_name =
        read_entire_file("/sys/devices/virtual/dmi/id/board_name");
    let board_vendor =
        read_entire_file("/sys/devices/virtual/dmi/id/board_vendor");
    println!("Board Name:   {}", board_name);
    println!("Board Vendor: {}", board_vendor);
}

fn print_cpu_info(sys: &sysinfo::System) {
        if let Some(cpu) = sys.cpus().first() {
            println!("CPU:          {}", cpu.brand());
        }
        println!("Cores:        {}", sys.cpus().len());
}

fn print_bios_info() -> () {
    let bios_vendor =
        read_entire_file("/sys/class/dmi/id/bios_vendor");
    let bios_version =
        read_entire_file("/sys/class/dmi/id/bios_version");
    let bios_date =
        read_entire_file("/sys/class/dmi/id/bios_date");
    let bios_release =
        read_entire_file("/sys/class/dmi/id/bios_release");
    println!("Bios vendor:  {bios_vendor}");
    println!("Bios version: {bios_version}");
    println!("Bios date:    {bios_date}");
    println!("Bios release: {bios_release}");

}

#[inline]
fn get_hostname() -> String {
    nix::unistd::gethostname()
        .expect("Failed to get hostname")
        .to_string_lossy()
        .into_owned()
}

#[inline]
fn get_username() -> String {
    use nix::unistd;
    let uid = unistd::getuid();
    if let Some(user) = nix::unistd::User::from_uid(uid).unwrap() {
        user.name
    } else {
        "".to_string()
    }
}

fn print_username_and_hostname() {
    let username_and_hostname =
        format!("{}@{}", get_username(), get_hostname());
    println!("Name:         {}", username_and_hostname);
}

fn main() {
    print_username_and_hostname();
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    pollster::block_on(async {
        let wgpu_instance = wgpu::Instance::default();
        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate GPU adapter");

        print_graphics_info(adapter);
    });
    print_bios_info();
    print_motherboard_info();
    print_available_memory(&mut sys);
    print_cpu_info(&mut sys);
    print_uptime();
    print_shell();
    print_editor();
}
