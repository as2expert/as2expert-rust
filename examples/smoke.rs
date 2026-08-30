//! E2E smoke test: `AS2EXPERT_TOKEN=... cargo run --example smoke`.
use as2expert::AS2ExpertClient;
use serde_json::json;

#[tokio::main]
async fn main() -> as2expert::Result<()> {
    let token = std::env::var("AS2EXPERT_TOKEN").expect("set AS2EXPERT_TOKEN");
    let client = AS2ExpertClient::builder(token).environment("free").build()?;
    println!("base_url: {}", client.base_url());

    let edi = "UNB+UNOC:3+A+B+260830:1000+1'UNH+1+ORDERS:D:96A:UN'BGM+220+PO-RS'UNT+2+1'UNZ+1+1'";
    let out = client.edifact.convert(edi, "json").await?;
    println!(
        "convert -> filename={} content_len={}",
        out["filename"],
        out["content"].as_str().map(|s| s.len()).unwrap_or(0)
    );

    let ack = client
        .edifact
        .acknowledge("UNA:+.?*'UNB+UNOC:3+A:14+B:14+260830:1000+X9'UNH+M1+ORDERS:D:96A:UN'BGM+220+P'UNT+2+M1'UNZ+1+X9'")
        .await?;
    println!("acknowledge -> kind={} ctrl={}", ack["kind"], ack["control_reference"]);

    let msgs = client.messages.list(json!({ "limit": 3 })).await?;
    println!("messages.list -> {} items", msgs.len());
    Ok(())
}
