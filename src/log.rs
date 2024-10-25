use eyre::Result;
use eyre::WrapErr;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use crate::journal::Journal;

  pub fn tail_journalctl(staked_or_not: Option<bool>) -> Result<()> {
      // Get the path of the current executable
      let exe_path = env::current_exe().wrap_err("Failed to get the current executable path")?;
      let exe_path_str = exe_path.to_string_lossy();

      // Prepare the grep expression to match JSON objects containing the "address" key
      // let grep_expr = r#"{[^}]*"address"[[:space:]]*:[[:space:]]*"[^"]*"[^}]*}"#;
      // Prepare the grep expression to match based on staked status
      let grep_expr = match staked_or_not {
          Some(true) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"✅"[^}]*}"#,
          Some(false) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"❌"[^}]*}"#,
          None => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"(✅|❌)"[^}]*}"#,
      };

      // Start the journalctl command
      let mut child = Command::new("journalctl")
          .args(&[
              &format!("_EXE={}", exe_path_str),
              &format!("--grep={}", grep_expr),
              "--output=cat",
              "--no-pager",
              "--follow",
              "--lines=200",
          ])
          .stdout(Stdio::piped())
          .spawn()
          .wrap_err("Failed to start journalctl")?;

      // Read stdout from journalctl
      let reader = BufReader::new(child.stdout.take().unwrap());

      for line in reader.lines() {
          let line = line.wrap_err("Failed to read line from journalctl")?;
          let journal = Journal::from_json_str(&line)?;
          println!("{}", journal.log());
      }

      // Wait for the child process to finish
      let status = child.wait().wrap_err("Failed to wait for journalctl process")?;
      if !status.success() {
          return Err(eyre::eyre!("journalctl process exited with a non-zero status"));
      }

      Ok(())
  }
