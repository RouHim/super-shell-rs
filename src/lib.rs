use std::io::{BufRead, BufReader, Write};

use std::process::{Command, Stdio};

/// Represents a shell with root privileges.
pub struct RootShell {
    shell_process: std::process::Child,
}

/// String that is appended to the end of each command, to indicate that the command has finished.
const END_OF_COMMAND: &str = "~end-of-command~";

/// Implementation of RootShell
impl RootShell {
    /// Creates a new root shell.
    /// Defaults to pkexec as super user provider and sh as shell.
    /// Returns None if the root shell could not be created.
    /// # Example
    /// ```
    /// use super_shell::RootShell;
    /// let mut root_shell = RootShell::new().expect("Failed to crate root shell");
    /// println!("{}", root_shell.execute("echo Hello $USER"));
    /// ```
    pub fn new() -> Option<Self> {
        Self::spawn_root_shell("pkexec", "sh")
    }

    /// Creates a new root shell with the specified super user provider and shell.
    /// Returns None if the root shell could not be created.
    /// sudo as an interactive super user provider is currently not supported.
    /// # Parameter
    /// * `super_user_provider` - The command to use to get super user privileges
    /// * `shell` - The shell to use
    /// # Example
    /// ```
    /// use super_shell::RootShell;
    /// let mut root_shell = RootShell::new_custom("gksu", "bash").expect("Failed to crate root shell");
    /// println!("{}", root_shell.execute("echo Hello $USER"));
    /// ```
    pub fn new_custom(super_user_provider: &str, shell: &str) -> Option<Self> {
        Self::spawn_root_shell(super_user_provider, shell)
    }

    /// Creates a new root shell with the specified super user provider and shell.
    /// Returns None if the root shell could not be created.
    /// # Parameter
    /// * `super_user_provider` - The command to use to get super user privileges
    /// * `shell` - The shell to use
    fn spawn_root_shell(super_user_provider: &str, shell: &str) -> Option<RootShell> {
        let shell_process = Command::new(super_user_provider)
            .arg(shell)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();

        let mut root_shell = Self { shell_process };

        // Make sure we are root now
        let user = root_shell.execute("whoami");
        if !user.trim().eq("root") {
            return None;
        }

        Some(root_shell)
    }

    /// Executes a command in the root shell and returns the output trimmed.
    /// Blocks the current thread until the command is finished.
    /// # Parameter
    /// * `command` - The command to execute
    /// # Example
    /// ```
    /// use super_shell::RootShell;
    /// let mut root_shell = RootShell::new().expect("Failed to crate root shell");
    /// assert!(root_shell.execute("echo Hello $USER").trim().eq("Hello root"));
    /// ```
    pub fn execute(&mut self, command: impl AsRef<str>) -> String {
        // Append end of command string to the command
        let command = command.as_ref().to_string();

        // Write the actual command to stdin of the root shell
        let mut shell_stdin = self.shell_process.stdin.as_mut().unwrap();
        writeln!(&mut shell_stdin, "{}", command).unwrap();
        shell_stdin.flush().unwrap();

        // Write "end of command" string to stdin of the root shell
        writeln!(&mut shell_stdin, "echo {}", END_OF_COMMAND).unwrap();
        shell_stdin.flush().unwrap();

        // Read piped stdout from the root shell
        let stdout = self.shell_process.stdout.as_mut().unwrap();
        let mut stdout_reader = BufReader::new(stdout);

        // Read until the "end of command" string is found
        let mut string_data = String::new();
        loop {
            let mut line = String::new();
            stdout_reader.read_line(&mut line).unwrap();
            string_data.push_str(&line);

            if line.contains(END_OF_COMMAND) {
                break;
            }
        }

        // Clean up the string
        let cmd_response = string_data.replace(END_OF_COMMAND, "");
        let cmd_response = cmd_response.trim().to_string();

        cmd_response
    }

    /// Exits the root shell and waits for the process to finish.
    pub fn exit(&mut self) {
        let mut shell_stdin = self.shell_process.stdin.as_mut().unwrap();
        writeln!(&mut shell_stdin, "exit").unwrap();
        shell_stdin.flush().unwrap();
        self.shell_process.wait().expect("failed to wait on child");
    }
}

/// Drop implementation for RootShell.
impl Drop for RootShell {
    /// Exits the root shell and waits for the process to finish.
    fn drop(&mut self) {
        self.exit();
        self.shell_process.wait().expect("failed to wait on child");
    }
}

#[cfg(test)]
mod tests {
    use crate::RootShell;

    #[test]
    fn test_custom() {
        let mut root_shell = RootShell::new_custom("sudo", "bash").unwrap();
        assert!(root_shell
            .execute("echo Hello $USER")
            .trim()
            .eq("Hello root"));
    }
}
