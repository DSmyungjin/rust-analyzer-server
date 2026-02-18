// This binary is no longer needed.
// The main rust-analyzer-mcp binary now runs as a standalone HTTP server.
// This is kept as a no-op to avoid breaking cargo workspace builds.

fn main() {
    eprintln!("test-support-server is deprecated. Use `rust-analyzer-mcp` directly.");
    std::process::exit(0);
}
