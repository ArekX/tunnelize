use common::{
    cli::{parse_command, Commands},
    logger::initialize_logger,
};
use log::{debug, info};

mod common;
mod server;
mod tunnel;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let command = parse_command();

    initialize_logger(&command);

    if let Err(e) = run_command(command).await {
        debug!("Error running command: {:?}", e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

async fn run_command(command: Commands) -> Result<(), std::io::Error> {
    match command {
        Commands::Init => {
            // write_tunnel_config(Configuration {
            //     server: Some(get_default_server_config()),
            //     tunnel: Some(get_default_tunnel_config()),
            // })?;
            return Ok(());
        }
        Commands::Server { init } => {
            if init {
                // write_tunnel_config(Configuration {
                //     server: Some(get_default_server_config()),
                //     tunnel: None,
                // })?;
                return Ok(());
            }

            // let config = get_configuration();

            info!("Starting server...");

            server::start().await?;

            // if let Some(server) = config.server {
            //     // server::start(server).await?;
            // } else {
            //     error!("No server configuration found, cannot start a server. Exiting...");
            // }
        }
        Commands::Tunnel { init, .. } => {
            if init {
                // write_tunnel_config(Configuration {
                //     server: None,
                //     tunnel: Some(get_default_tunnel_config()),
                // })?;
                return Ok(());
            }

            tunnel::start().await?;

            // let config = get_configuration();

            // info!("Starting client...");

            // if let Some(tunnel) = config.tunnel {
            //     // tunnel::start(tunnel).await?;
            // } else {
            //     error!("No tunel configuration found, cannot start a tunnel. Exiting...");
            // }
        }
    }

    Ok(())
}

// async fn test_tls() -> tokio::io::Result<()> {
//     let mut root_cert_store = RootCertStore::empty();
//     root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
//     let config = ClientConfig::builder()
//         .with_root_certificates(root_cert_store)
//         .with_no_client_auth();
//     let connector = TlsConnector::from(Arc::new(config));
//     let dnsname = ServerName::try_from("noteme.arekxv.name").unwrap();

//     let addr = "noteme.arekxv.name:443"
//         .to_owned()
//         .to_socket_addrs()?
//         .next()
//         .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

//     let stream = TcpStream::connect(&addr).await?;
//     let mut stream = connector.connect(dnsname, stream).await?;

//     let content = format!("GET / HTTP/1.0\r\nHost: noteme.arekxv.name\r\n\r\n");

//     stream.write_all(content.as_bytes()).await?;

//     let mut buf = vec![0; 1024];
//     let n = stream.read(&mut buf).await?;

//     println!("{}", String::from_utf8_lossy(&buf[..n]));

//     Ok(())
// }
