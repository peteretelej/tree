use rust_tree::rust_tree::cli::run_cli;
use rust_tree::rust_tree::utils::is_broken_pipe_error;

fn main() {
    match run_cli() {
        Ok(()) => {}
        Err(err) => {
            if is_broken_pipe_error(&err) {
                // silently terminate for broken pipe to gracefully handle SIGPIPE
                std::process::exit(0);
            } else {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
    }
}
