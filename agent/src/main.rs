use clap::Parser;
use execute::Execute;
use machine_uid;
use port_scanner::scan_port;
use reqwest::{self, Client};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::ops::Sub;
use std::process::{Command, Stdio};
use std::time::Duration;
use sysinfo::{self, CpuExt, DiskExt, NetworkExt, ProcessExt, System, SystemExt};
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    token: Option<String>,

    #[arg(short, long)]
    url: Option<String>,

    #[arg(short, long, default_value_t = true)]
    dev: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let url = if args.url.is_some() {
        args.url.unwrap()
    } else if args.dev {
        "http://localhost:3000".to_string()
    } else {
        "http://localhost:3000".to_string()
    };

    let client = reqwest::Client::new();

    let mut sys = System::new_all();

    let id = machine_uid::get().unwrap();
    let host_name = sys.host_name();
    let os = sys.name();
    let version = sys.os_version();
    let kernel = sys.kernel_version();

    let ip = {
        let result = reqwest::get("https://api.myip.com").await.ok();
        if let Some(result) = result {
            let result = result.json::<Value>().await.ok();
            if let Some(result) = result {
                let ip = result.get("ip");
                let ip = ip.map(|ip| ip.as_str().unwrap_or("unknow") );
                let ip = ip.as_deref().unwrap();
                Some(ip.to_string())
            } else {
                None
            }
        } else {
            None
        }
    };
    let ip =  ip.unwrap_or("unknow".to_string());

    loop {
        let reponse = client
            .post(format!("{}/server/{}", url, id))
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "token": args.token,
                    "name": host_name,
                    "os": os,
                    "version": version,
                    "kernel": kernel,
                    "ip": ip
                })
                .to_string(),
            )
            .send()
            .await;

        match reponse {
            Ok(result) => {
                if result.status() == 200 {
                    println!("inited");
                    break;
                } else {
                    println!(
                        "Error: {}",
                        result.text().await.unwrap_or("unknow error".to_string())
                    );
                    println!("will be retry in 10s to connect to server...");
                    sleep(Duration::from_secs(10)).await;
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                println!("will be retry in 10s to connect to server...");
                sleep(Duration::from_secs(10)).await;
            }
        }
    }

    let mut states: HashMap<String, i64> = HashMap::new();
    let mut infos: HashMap<String, String> = HashMap::new();

    loop {
        let updated = read(&mut sys, &mut states).await;
        sleep(Duration::from_secs(5)).await;

        let _response = client
            .post(format!("{}/server/{}/state", url, id))
            .header("Content-Type", "application/json")
            .body(json!(updated).to_string())
            .send()
            .await;
    }
}

async fn read(sys: &mut System, states: &mut HashMap<String, i64>) -> HashMap<String, i64> {
    sys.refresh_all();

    let mut updated = HashMap::new();

    let _updated = &mut updated;
    let mut update_state = move |key: String, value: i64| {
        let old = states.insert(key.clone(), value);

        let diff = {
            if let Some(old) = old {
                (old.abs() - value.abs()).abs() as f64 / value as f64
            } else {
                1f64
            }
        };

        if diff > 0.1 || true {
            _updated.insert(key, value);
        }
    };

    // let mut update_info= move |key: String, value: String| {
    //     infos.insert(key, value);
    // };

    //update_info("system::os::version".to_string(), sys.os_version().unwrap_or("unknow".to_string()) );
    //update_info("system::os::kernel".to_string(), sys.kernel_version().unwrap_or("unknow".to_string()) );

    update_state("system::os::uptime".to_string(), sys.uptime() as i64);
    update_state(
        "system::load::1m".to_string(),
        sys.load_average().one as i64,
    );
    update_state(
        "system::load::5m".to_string(),
        sys.load_average().five as i64,
    );
    update_state(
        "system::load::15m".to_string(),
        sys.load_average().fifteen as i64,
    );
    update_state(
        "system::cpu::used".to_string(),
        sys.global_cpu_info().cpu_usage() as i64,
    );

    let mem_total = sys.total_memory();
    let mem_used = sys.used_memory();
    let mem_percent = ((mem_used as f64 / mem_total as f64) * 100f64) as i64;
    let swap_total = sys.total_swap();
    let swap_used = sys.used_swap();
    let swap_percent = if swap_total == 0 {
        0
    } else {
        ((swap_used as f64 / swap_total as f64) * 100f64) as i64
    };

    // println!("{}", (sys.used_memory() as f64 / mem_total as f64));
    // println!("{}", (sys.available_memory() as f64 / mem_total as f64));
    // println!("{}", (sys.free_memory() as f64 / mem_total as f64));

    update_state("system::mem::used".to_string(), mem_used as i64);
    update_state("system::mem::total".to_string(), mem_total as i64);
    update_state("system::mem::percent".to_string(), mem_percent);
    update_state("system::swap::used".to_string(), swap_used as i64);
    update_state("system::swap::total".to_string(), swap_total as i64);
    update_state("system::swap::percent".to_string(), swap_percent);

    sys.cpus().iter().for_each(|cpu| {
        let key = format!("cpu::{}::used", cpu.name());
        update_state(key, cpu.cpu_usage() as i64);
    });

    update_state("system::cpu::core".to_string(), sys.cpus().len() as i64);

    sys.disks().iter().enumerate().for_each(|(i, disk)| {
        // update_info(format!("disk::{}::mount", disk.name().to_str().unwrap()), disk.mount_point().to_str().unwrap().to_string());
        update_state(
            format!("disk::{}::percent", i),
            (disk.available_space() as f64 / disk.total_space() as f64 * 100f64) as i64,
        );
        update_state(
            format!("disk::{}::used", i),
            (disk.available_space() as f64 * 0.000001) as i64,
        );
        update_state(
            format!("disk::{}::total", i),
            (disk.total_space() as f64 * 0.000001) as i64,
        );
    });
    // //df -khi

    sys.networks().into_iter().for_each(|(name, data)| {
        // update_info(format!("network::{}::mac", name), data.mac_address().to_string());
        update_state(
            format!("network::{}::rx", name),
            (data.received() as f64 * 0.000001) as i64,
        );
        update_state(
            format!("network::{}::tx", name),
            (data.transmitted() as f64 * 0.000001) as i64,
        );
    });

    let programs = vec![
        ("MySql", 3306),
        ("PostgreSQL", 5432),
        ("LiteSpeed", 7080),
        ("SSH", 22),
        ("HTTP", 80),
        ("TLS/SSL", 443),
        ("Redis", 6379),
    ];

    let mut cmd = Command::new("cat");
    cmd.arg("/etc/services");
    cmd.stdout(Stdio::piped());
    let output = cmd.execute_output().unwrap();
    let services = String::from_utf8(output.stdout).unwrap();

    let mut up = 0;
    let mut down = 0;
    let mut nope = 0;
    for (name, port) in programs {
        let exist = services.contains(name) && services.contains(port.to_string().as_str());
        let exist = if exist { 1 } else { 0 };
        let running = if scan_port(port) { 1 } else { 0 };

        // println!("{} {} {}", name, exist, running);
        if exist == 1 && running == 1 {
            up += 1
        } else if exist == 1 && running == 0 {
            down += 1
        } else {
            nope += 1
        };

        update_state(format!("service::{}::port", name), port as i64);
        update_state(format!("service::{}::running", name), running);
        update_state(format!("service::{}::exist", name), exist);
    }

    update_state("service::all::up".to_string(), up);
    update_state("service::all::down".to_string(), down);
    update_state("service::all::nope".to_string(), nope);

    // let processes = sys.processes();
    // processes.iter().for_each(|(pid, data)| {
    //     let name = format!("{} - {}", pid, data.name().to_string());
    //     // let pid = pid.to_string().parse::<i32>().unwrap();

    //     update_state(format!("process::{}::cpu", pid ), data.cpu_usage() as i32);
    //     update_state(format!("process::{}::mem", pid ), data.memory() as i32);
    // });

    // println!("{:#?}", data);

    updated
}
