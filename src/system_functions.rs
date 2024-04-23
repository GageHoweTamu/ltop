use std::time;
use tokio::*;
use std::thread;
use sysinfo::NetworkExt;
use sysinfo::NetworksExt;
use sysinfo::System;
use sysinfo::SystemExt;

async fn get_total_bytes() -> (u64, u64) { // (sent, received)
    let mut system = System::new_all();
    system.refresh_all();
    let mut total_outcome :u64 = 0;
    let mut total_income :u64 = 0;
    let mut starting_outcome :u64 = 0;
    let mut starting_income :u64 = 0;
    let mut network_data = Networks::new_with_refreshed_list();
    network_data.iter().for_each(|(name, data)| {
    total_outcome += data.total_received();
    total_income += data.total_transmitted();
    println!("{:?}: {:?} received, {:?} sent", name, data.total_received(), data.total_transmitted());
    });
    println!("Total received: {:?}, Total sent: {:?}", bytes_to_string(total_outcome), bytes_to_string(total_income));
}

// Get the average core usage
fn get_cpu_use(req_sys: &sysinfo::System) -> f32
{
    // Put all of the core loads into a vector
    let mut cpus: Vec<f32> = Vec::new();
    for core in req_sys.get_processors() { cpus.push(core.get_cpu_usage()); }

    // Get the average load
    let cpu_tot: f32 = cpus.iter().sum();
    let cpu_avg: f32 = cpu_tot / cpus.len() as f32;

    cpu_avg
}

// Divide the used RAM by the total RAM
fn get_ram_use(req_sys: &sysinfo::System) -> f32
{
    (req_sys.get_used_memory() as f32) / (req_sys.get_total_memory() as f32) * 100.
}

// Divide the used swap by the total swap
fn get_swp_use(req_sys: &sysinfo::System) -> f32
{
    (req_sys.get_used_swap() as f32) / (req_sys.get_total_swap() as f32) * 100.
}

// Get the total network (down) usage
fn get_ntwk_dwn(req_sys: &sysinfo::System) -> i32
{
    // Get the total bytes recieved by every network interface
    let mut rcv_tot: Vec<i32> = Vec::new();
    for (_interface_name, ntwk) in req_sys.get_networks() { rcv_tot.push(ntwk.get_received() as i32); }

    // Total them and convert the bytes to KB
    let ntwk_tot: i32 = rcv_tot.iter().sum();
    let ntwk_processed = (ntwk_tot / 128) as i32;

    ntwk_processed
}

// Get the total network (up) usage
fn get_ntwk_up(req_sys: &sysinfo::System) -> i32
{
    // Get the total bytes sent by every network interface
    let mut snd_tot: Vec<i32> = Vec::new();
    for (_interface_name, ntwk) in req_sys.get_networks() { snd_tot.push(ntwk.get_transmitted() as i32); }

    // Total them and convert the bytes to KB
    let ntwk_tot: i32 = snd_tot.iter().sum();
    let ntwk_processed = (ntwk_tot / 128) as i32;

    ntwk_processed
}

fn get_temp(req_sys: &sysinfo::System) -> i32
{
    // For every component, if it's the CPU, put its temperature in variable to return
    let mut wanted_temp: f32 = -1.;
    for comp in req_sys.get_components() { if comp.get_label() == "CPU" { wanted_temp = comp.get_temperature(); } }
    
    wanted_temp as i32
}