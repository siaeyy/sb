use std::error::Error;
use std::{env, process,};

use arboard::{Clipboard, SetExtLinux};

#[cfg(target_os = "linux")]
const CLIPBOARD_DAEMON_SYMBOL: &str = "__clipboard_daemon_symbol__";

#[cfg(target_os = "linux")]
pub fn handle_clipboard_request() -> Result<(), arboard::Error>{
    let args: Vec<String> = env::args().skip(1).collect();

    let (arg1, arg2) = match args.as_slice() {
        [a, b] => (a, b),
        _ => return Ok(()),
    };

	if arg1 == CLIPBOARD_DAEMON_SYMBOL {
		Clipboard::new()?.set().wait().text(arg2)?;
        process::exit(0);
	}

	Ok(())
}

pub fn clipboard_copy(content: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    if cfg!(not(target_os = "linux")) {
        return Ok(Clipboard::new()?.set_text(content)?);
    }

    process::Command::new(env::current_exe()?)
		.args([
            CLIPBOARD_DAEMON_SYMBOL,
            content,
        ])
		.stdin(process::Stdio::null())
		.stdout(process::Stdio::null())
		.stderr(process::Stdio::null())
		.current_dir("/")
		.spawn()?;

    Ok(())
}
