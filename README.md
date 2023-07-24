<p align="center">
    <img src="https://raw.githubusercontent.com/RouHim/super-shell-rs/main/logo.png" width="500"/>
</p>

<p align="center">
    <a href="https://crates.io/crates/super-shell"><img src="https://img.shields.io/crates/d/super-shell"/></a>
    <a href="https://crates.io/crates/super-shell"><img src="https://img.shields.io/crates/v/super-shell"/></a>
    <a href="https://github.com/RouHim/super-shell-rs/releases"><img src="https://img.shields.io/github/release-date/rouhim/super-shell-rs"/></a>
    <a href="https://github.com/RouHim/super-shell-rs/actions"><img src="https://img.shields.io/github/actions/workflow/status/rouhim/super-shell-rs/pipe.yaml"/></a>
</p>

<p align="center">
    <i>This library provides basic super-user shell access in rust.</i>
</p>

## Example usage
```rust
use super_shell::RootShell;

fn main() {
    // Super user privileges are requested once via pkexec as default.
    let mut root_shell = RootShell::new().expect("Failed to crate root shell");
    
    // All subsequent requests are executed as root user
    println!("{}", root_shell.execute("echo Hello $USER"));
    
    // Each command blocks until the response is fully received
    println!("{}", root_shell.execute("echo sleeping for 3s ..."));
    root_shell.execute("sleep 3");
}
```

## Limitations
* Currently only Linux is supported
* No Stderr handling
