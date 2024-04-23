use sysinfo::*;
use tokio::*;

#[tokio::main]
async fn main() {
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

// function to take bytes and convert them to a human readable format
fn bytes_to_string (bytes: u64) -> String {
    let kb = bytes / 1024;
    let mb = kb / 1024;
    let gb = mb / 1024;
    let tb = gb / 1024;
    if tb > 1 {
        return format!("{:.2} TB", tb);
    } else if gb > 1 {
        return format!("{:.2} GB", gb);
    } else if mb > 1 {
        return format!("{:.2} MB", mb);
    } else if kb > 1 {
        return format!("{:.2} KB", kb);
    } else {
        return format!("{:.2} B", bytes);
    }
}