async fn test_tls() -> tokio::io::Result<()> {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let dnsname = ServerName::try_from("noteme.arekxv.name").unwrap();

    let addr = "noteme.arekxv.name:443"
        .to_owned()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

    let stream = TcpStream::connect(&addr).await?;
    let mut stream = connector.connect(dnsname, stream).await?;

    let content = format!("GET / HTTP/1.0\r\nHost: noteme.arekxv.name\r\n\r\n");

    stream.write_all(content.as_bytes()).await?;

    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
