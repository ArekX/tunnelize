use std::error::Error;

mod proxy;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let command_type: String = String::from("server");

    match command_type.as_str() {
        "server" => {
            server::start_server().await;
        }
        "proxy" => {
            proxy::start_proxy().await?;
        }
        _ => {
            println!("Invalid command type");
        }
    }

    Ok(())
}
