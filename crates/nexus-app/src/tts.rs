//! Text-to-speech on a background thread. Windows: WinRT speech synthesis via the `tts` crate.
//! Linux: speech-dispatcher's `spd-say`, which desktop distributions ship; without it the app stays quiet.
//! macOS: the system's `say`, in the voice chosen in System Settings › Accessibility › Spoken Content.

use crossbeam_channel::{Receiver, Sender};

pub enum TtsCmd {
    Say(String),
    Rate(f32),
    Volume(f32),
}

#[derive(Clone)]
pub struct Speaker {
    tx: Sender<TtsCmd>,
}

impl Speaker {
    pub fn start() -> Speaker {
        let (tx, rx) = crossbeam_channel::unbounded();
        let _ = std::thread::Builder::new().name("nexus-tts".into()).spawn(move || run(rx));
        Speaker { tx }
    }

    pub fn say(&self, text: impl Into<String>) {
        let _ = self.tx.send(TtsCmd::Say(text.into()));
    }

    pub fn set_rate(&self, rate: f32) {
        let _ = self.tx.send(TtsCmd::Rate(rate));
    }

    pub fn set_volume(&self, v: f32) {
        let _ = self.tx.send(TtsCmd::Volume(v));
    }
}

#[cfg(windows)]
fn run(rx: Receiver<TtsCmd>) {
    let mut engine = match tts::Tts::default() {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("text-to-speech unavailable: {e}");
            // Drain so senders never block or error noisily.
            while rx.recv().is_ok() {}
            return;
        }
    };
    let (min_rate, max_rate, normal_rate) = (engine.min_rate(), engine.max_rate(), engine.normal_rate());
    while let Ok(cmd) = rx.recv() {
        match cmd {
            TtsCmd::Say(text) => {
                // Queue rather than interrupt, so back-to-back alerts are all heard.
                if let Err(e) = engine.speak(text, false) {
                    tracing::warn!("tts speak failed: {e}");
                }
            }
            TtsCmd::Rate(r) => {
                let rate = (normal_rate * r).clamp(min_rate, max_rate);
                let _ = engine.set_rate(rate);
            }
            TtsCmd::Volume(v) => {
                let _ = engine.set_volume(v.clamp(0.0, 1.0));
            }
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn run(rx: Receiver<TtsCmd>) {
    let (mut rate, mut volume) = (1.0, 1.0);
    while let Ok(cmd) = rx.recv() {
        match cmd {
            TtsCmd::Say(text) => {
                // -w waits until the line is spoken, so back-to-back alerts queue rather than interrupt.
                let spoken = std::process::Command::new("spd-say")
                    .args(["-w", "-r", &spd_rate(rate).to_string(), "-i", &spd_volume(volume).to_string(), "--", &text])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                if let Err(e) = spoken {
                    tracing::warn!("text-to-speech unavailable (spd-say, from speech-dispatcher): {e}");
                    while rx.recv().is_ok() {}
                    return;
                }
            }
            TtsCmd::Rate(r) => rate = r,
            TtsCmd::Volume(v) => volume = v,
        }
    }
}

/// The speed setting (1.0 = normal) as spd-say's rate, -100 to 100 with 0 normal.
#[cfg(not(any(windows, target_os = "macos")))]
fn spd_rate(rate: f32) -> i32 {
    ((rate - 1.0) * 100.0).clamp(-100.0, 100.0).round() as i32
}

/// The volume setting (0 to 1) as spd-say's, -100 (silent) to 100 (full).
#[cfg(not(any(windows, target_os = "macos")))]
fn spd_volume(volume: f32) -> i32 {
    (volume.clamp(0.0, 1.0) * 200.0 - 100.0).round() as i32
}

#[cfg(target_os = "macos")]
fn run(rx: Receiver<TtsCmd>) {
    let (mut rate, mut volume) = (1.0, 1.0);
    while let Ok(cmd) = rx.recv() {
        match cmd {
            TtsCmd::Say(text) => {
                if let Err(e) = say("/usr/bin/say", &text, rate, volume) {
                    tracing::warn!("text-to-speech unavailable (say): {e}");
                    while rx.recv().is_ok() {}
                    return;
                }
            }
            TtsCmd::Rate(r) => rate = r,
            TtsCmd::Volume(v) => volume = v,
        }
    }
}

/// Speaks one line with macOS's `say` and returns when it has been spoken, so back-to-back alerts queue rather
/// than interrupt. The text goes in on standard input, so a line starting with "-" can't be taken for an option.
#[cfg(any(target_os = "macos", all(test, unix)))]
fn say(program: &str, text: &str, rate: f32, volume: f32) -> std::io::Result<std::process::ExitStatus> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut cmd = Command::new(program);
    if let Some(wpm) = say_rate(rate) {
        cmd.args(["-r", &wpm.to_string()]);
    }
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        // A failed write (say quit early) still leaves its exit status to wait for.
        let _ = stdin.write_all(say_input(text, volume).as_bytes());
    }
    child.wait()
}

/// The speed setting (1.0 = normal) as `say`'s words per minute; `None` at normal speed, which keeps the rate set
/// for the voice in System Settings.
#[cfg(any(target_os = "macos", all(test, unix)))]
fn say_rate(rate: f32) -> Option<u32> {
    /// About the rate macOS voices speak at by default.
    const NORMAL_WPM: f32 = 180.0;
    ((rate - 1.0).abs() > 0.01).then(|| (NORMAL_WPM * rate).clamp(90.0, 720.0).round() as u32)
}

/// The line for `say`: below full volume it starts with the `[[volm]]` speech command (0 to 1). Brackets in the
/// text itself are loosened so a name can't be read as a speech command.
#[cfg(any(target_os = "macos", all(test, unix)))]
fn say_input(text: &str, volume: f32) -> String {
    let text = text.replace("[[", "[ [").replace("]]", "] ]");
    let volume = volume.clamp(0.0, 1.0);
    if volume < 0.995 { format!("[[volm {volume:.2}]] {text}") } else { text }
}

#[cfg(all(test, not(any(windows, target_os = "macos"))))]
mod tests {
    #[test]
    fn spd_say_ranges() {
        assert_eq!((super::spd_rate(1.0), super::spd_rate(1.5), super::spd_rate(3.0), super::spd_rate(0.5)), (0, 50, 100, -50));
        assert_eq!((super::spd_volume(1.0), super::spd_volume(0.5), super::spd_volume(0.0)), (100, 0, -100));
    }
}

#[cfg(all(test, unix))]
mod say_tests {
    use super::{say, say_input, say_rate};

    #[test]
    fn say_rate_and_volume() {
        assert_eq!((say_rate(1.0), say_rate(1.5), say_rate(0.5), say_rate(10.0)), (None, Some(270), Some(90), Some(720)));
        assert_eq!(say_input("Hull critical", 1.0), "Hull critical");
        assert_eq!(say_input("Hull critical", 0.5), "[[volm 0.50]] Hull critical");
        assert_eq!(say_input("Body [[rate 1]] A 1", 1.0), "Body [ [rate 1] ] A 1");
    }

    /// `say` gets the rate as arguments and the line on standard input (a stand-in script records both).
    #[test]
    fn say_sends_the_line_on_stdin() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("nexus-say-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (script, out) = (dir.join("say"), dir.join("heard.txt"));
        std::fs::write(&script, format!("#!/bin/sh\nprintf '%s|' \"$@\" > '{}'\ncat >> '{}'\n", out.display(), out.display())).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let status = say(&script.to_string_lossy(), "-1 jump left", 1.5, 0.25).unwrap();
        assert!(status.success());
        assert_eq!(std::fs::read_to_string(&out).unwrap(), "-r|270|[[volm 0.25]] -1 jump left");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
