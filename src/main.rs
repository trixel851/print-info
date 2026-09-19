use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::io;
use std::fs;

fn read_entire_file(path: &str) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn print_uptime() -> () {
    let uptime_secs = sysinfo::System::uptime();

    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    let seconds = uptime_secs % 60;

    println!("Uptime:       {}d {}h {}m {}s", days, hours, minutes, seconds);
}

fn print_graphics_info(adapter: wgpu::Adapter, sys: &mut sysinfo::System) {
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
        println!("Cores:        {} (Logical)", sys.cpus().len());
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

#[derive(Debug, Clone)]
pub struct UtsName {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String,
}

impl UtsName {
    pub fn get() -> io::Result<Self> {
        let mut raw_info: libc::utsname = unsafe { MaybeUninit::zeroed().assume_init() };
        let res = unsafe { libc::uname(&mut raw_info) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        let parse_field = |field: &[std::os::raw::c_char]| -> String {
            unsafe {
                CStr::from_ptr(field.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            }
        };

        Ok(UtsName {
            sysname: parse_field(&raw_info.sysname),
            nodename: parse_field(&raw_info.nodename),
            release: parse_field(&raw_info.release),
            version: parse_field(&raw_info.version),
            machine: parse_field(&raw_info.machine),
        })
    }
}

fn main() {
    let name = UtsName::get().unwrap();
    println!("Hostname:     {}", name.nodename);
    println!("OS:           {}", name.sysname);
    println!("Machine:      {}", name.machine);
    println!("Release:      {}", name.release);
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    pollster::block_on(async {
        let wgpu_instance = wgpu::Instance::default();
        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate GPU adapter");

        print_graphics_info(adapter, &mut sys);
    });
    print_bios_info();
    print_motherboard_info();
    print_cpu_info(&mut sys);
    print_uptime();
}
