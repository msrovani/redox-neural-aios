//! memory — cliente CLI do scheme memory: (TCP ou scheme file).

use memory_core::MemoryClient;

fn usage() -> ! {
    eprintln!(
        "Uso:\n  \
         memory remember <texto> [--scope SCOPE]\n  \
         memory recall <query> [--scope SCOPE] [-k N]\n  \
         memory health\n  \
         memory ping\n\n  \
         REDOX_MEMORY_BACKEND=tcp|scheme\n  \
         REDOX_SGDB_SOCKET (tcp, default 127.0.0.1:7741)\n  \
         REDOX_MEMORY_SCHEME_ROOT (scheme, default /scheme/memory)"
    );
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let client = MemoryClient::new();

    let result = match args[1].as_str() {
        "remember" => {
            if args.len() < 3 {
                usage();
            }
            let text = args[2..]
                .iter()
                .take_while(|a| !a.starts_with("--"))
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            let scope = args
                .iter()
                .position(|a| a == "--scope")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str);
            client.remember(&text, scope)
        }
        "recall" => {
            if args.len() < 3 {
                usage();
            }
            let query = args[2..]
                .iter()
                .take_while(|a| !a.starts_with('-'))
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            let scope = args
                .iter()
                .position(|a| a == "--scope")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str);
            let k: Option<usize> = args
                .iter()
                .position(|a| a == "-k" || a == "--k")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok());
            client.recall(&query, scope, k)
        }
        "health" => client.health(),
        "ping" => client.ping(),
        _ => usage(),
    };

    match result {
        Ok(val) => println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default()),
        Err(e) => {
            eprintln!("memory: {e}");
            std::process::exit(1);
        }
    }
}
