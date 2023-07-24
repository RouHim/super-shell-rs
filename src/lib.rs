use std::io::{BufRead, BufReader, Write};

use std::process::{Command, Stdio};

/// Represents a shell with root privileges
/// The shell is automatically closed when the struct is dropped
/// The shell is opened with the pkexec command
/// It uses the sh shell
pub struct RootShell {
    shell_process: std::process::Child,
}

/// String that is appended to the end of each command, to indicate that the command has finished
const END_OF_COMMAND: &str = "~end-of-command~";

/// Implementation of RootShell
impl RootShell {
    /// Creates a new root shell
    /// Returns None if the root shell could not be created
    /// or if the user did not enter the password
    pub fn new() -> Option<Self> {
        let shell_process = Command::new("pkexec")
            .arg("sh")
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

    /// Executes a command in the root shell and returns the output trimmed
    /// Blocks the current thread until the command is finished
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

    /// Exits the root shell and waits for the process to finish
    pub fn exit(&mut self) {
        let mut shell_stdin = self.shell_process.stdin.as_mut().unwrap();
        writeln!(&mut shell_stdin, "exit").unwrap();
        shell_stdin.flush().unwrap();
        self.shell_process.wait().expect("failed to wait on child");
    }
}

/// Drop implementation for RootShell
impl Drop for RootShell {
    /// Exits the root shell and waits for the process to finish
    fn drop(&mut self) {
        self.exit();
        self.shell_process.wait().expect("failed to wait on child");
    }
}
