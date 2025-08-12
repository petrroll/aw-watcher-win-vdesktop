use std::env::current_exe;

use auto_launch::AutoLaunchBuilder;
use aw_client_rust::AwClient;
use aw_models::{Bucket, Event};
use chrono::TimeDelta;
use serde_json::{Map, Value};
use tokio::signal;
use clap::{Parser};

#[derive(Debug, Parser)]
#[command(name = "aw-watcher-win-vdesktop", version, about = "ActivityWatch watcher for Windows virtual desktops")] 
struct Args {
    /// Port to connect to aw-server. If specified, overrides --tesitng.
    #[arg(long)]
    port: Option<u16>,

    /// Testing mode: uses port 5666 unless --port is provided
    #[arg(long)]
    testing: bool,

    /// Enable auto-run
    #[arg(long)]
    auto_run: bool,
}

async fn create_bucket(
    aw_client: &AwClient,
    bucket_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let hostname = hostname::get()?.into_string().unwrap();

    let res = aw_client
        .create_bucket(&Bucket {
            id: bucket_id,
            bid: None,
            _type: "vdestkop-name".to_string(),
            data: Map::new(),
            metadata: Default::default(),
            last_updated: None,
            hostname: hostname,
            client: "aw-watcher-win-vdestkop".to_string(),
            created: None,
            events: None,
        })
        .await?;

    Ok(res)
}

fn get_current_vdesktop() -> String {
    let vdesktop = winvd::get_current_desktop().unwrap();

    let vdesktop_name = vdesktop.get_name().unwrap();
    if vdesktop_name.is_empty() {
        let vdesktop_id = vdesktop.get_index().unwrap();
        return format!("Desktop {}", vdesktop_id + 1);
    }

    vdesktop_name
}

fn setup_autorun() -> anyhow::Result<()> {
    let exe_path_buf = current_exe()?;
    let exe_path = exe_path_buf.to_string_lossy().into_owned();
    
    // Extract the filename without extension for the app name
    let app_name = exe_path_buf
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("aw-watcher-win-vdesktop");
    
    let auto_launch = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&exe_path)
        .set_use_launch_agent(true)
        .build()?;

    auto_launch.enable()?;
    if !auto_launch.is_enabled()? {
        return Err(anyhow::anyhow!(
            "Failed to enable auto-launch for {}",
            app_name
        ));
    }

    println!("Auto-launch enabled for {}", app_name);
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    let port = if let Some(p) = cli.port { p } else if cli.testing { 5666 } else { 5600 };
    println!("Using port: {}", port);

    if cli.auto_run {
        if let Err(e) = setup_autorun() {
            eprintln!("Failed to set up auto-run: {}", e);
        }
    }

    let aw_client = AwClient::new("localhost", port, "aw-watcher-win-vdestkop").unwrap();
    println!("Connected to to ActivityWatch server at {}:{}", aw_client.hostname, port);
    
    let bucket_id = format!("aw-watcher-win-vdesktop_{}", aw_client.hostname);
    create_bucket(&aw_client, bucket_id.clone()).await.unwrap();
    println!("Created bucket: {}", bucket_id);

    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        let mut vdesktop_data = Map::new();
        let curr_vdesktop_name = get_current_vdesktop();
        vdesktop_data.insert(
            "vdesktop".to_string(),
            Value::String(curr_vdesktop_name.clone()),
        );
        vdesktop_data.insert(
            "title".to_string(),
            Value::String(curr_vdesktop_name.clone()),
        );

        let now = chrono::Utc::now();
        let shutdown_event = Event {
            id: None,
            timestamp: now,
            duration: TimeDelta::seconds(12),
            data: vdesktop_data,
        };

        aw_client.heartbeat(&bucket_id, &shutdown_event, 10.0).await.unwrap();

        tokio::select! {
            _ = &mut ctrl_c => {
                println!("Ctrl+C received, exiting...");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(8)) => {
                // Your periodic code here
            }
        }
    }

    let events = aw_client.get_events(&bucket_id, None, None, Some(1)).await.unwrap();
    print!("{:?}", events); // prints a single event

}
