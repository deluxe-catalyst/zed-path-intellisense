mod completion;
mod jsonrpc;
mod server;

use server::Server;

fn main() {
    let mut server = Server::new();
    server.run();
}
