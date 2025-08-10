use aw_client_rust::AwClient;
use aw_models::{Bucket, Event};
use chrono::TimeDelta;
use serde_json::{Map, Value};
use tokio::signal;

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
    print!("{:?}", vdesktop);

    let vdesktop_name = vdesktop.get_name().unwrap();
    if vdesktop_name.is_empty() {
        let vdesktop_id = vdesktop.get_index().unwrap();
        return format!("Desktop {}", vdesktop_id + 1);
    }

    vdesktop_name
}

#[tokio::main]
async fn main() {
    let port = 5666; // the testing port 
    let aw_client = AwClient::new("localhost", port, "aw-watcher-win-vdestkop").unwrap();
    let bucket_id = format!("aw-watcher-win-vdesktop_{}", aw_client.hostname);
 
    println!("Creating bucket: {}", bucket_id);
    create_bucket(&aw_client, bucket_id.clone()).await.unwrap();

    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        let mut vdesktop_data = Map::new();
        let curr_vdesktop_name = get_current_vdesktop();
        vdesktop_data.insert(
            "vdesktop".to_string(),
            Value::String(curr_vdesktop_name),
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
