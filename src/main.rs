// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;
use std::convert::Infallible;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use serde_json::json;
use dns_lookup::lookup_addr;

async fn getip(req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    let empty_header = "".parse().unwrap();
    let real_ip = req.headers().get("X-Real-IP").unwrap_or(&empty_header).to_str().unwrap_or("");
    let real_port = req.headers().get("X-Real-Port").unwrap_or(&empty_header).to_str().unwrap_or("");
    let remote_addr = if real_ip != "" {
        real_ip.to_owned()
    } else {
        addr.ip().to_string()
    };
    let port = if real_port != "" {
        real_port.to_owned()
    } else {
        addr.port().to_string()
    };
    let ip: IpAddr = remote_addr.parse().unwrap_or([0, 0, 0, 0].into());
    let hostname = lookup_addr(&ip).unwrap_or("".to_string());
    let result = json!({
        "ip": remote_addr,
        "port": port,
        "hostname": hostname,
    });
    let response = Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(result.to_string()))
        .unwrap();
    Ok(response)
}

#[tokio::main]
async fn main() {
    let addr_string = env::args().nth(1).unwrap_or("".to_string());
    let addr = SocketAddr::from_str(&addr_string).unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8080)));

    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let addr = conn.remote_addr();
        async move {
            let addr = addr.clone();
            Ok::<_, Infallible>(service_fn(move |req : Request<Body>| {
                getip(req, addr)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}