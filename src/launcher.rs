use crate::config::Config;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn open_bookmark(url: &str, config: &Config) -> Result<(), String> {
    let browser = resolve_browser();
    let mut args: Vec<String> = vec![format!("--app={}", url)];
    args.extend(config.launcher.args.clone());

    log_line(&format!("open_bookmark url={}", url));
    log_line(&format!("resolved_browser={}", browser));
    log_line(&format!("args={:?}", args));

    // 1) Best option for Walker/GUI launchers: systemd-run user service
    match launch_with_systemd_run(&browser, &args) {
        Ok(()) => {
            log_line("launch_with_systemd_run ok");
            return Ok(());
        }
        Err(e) => log_line(&format!("launch_with_systemd_run failed: {}", e)),
    }

    // 2) Fallback: uwsm-app + setsid
    if let Some((setsid, uwsm_app)) = resolve_uwsm_stack() {
        log_line(&format!("trying uwsm launch: setsid={} uwsm-app={}", setsid, uwsm_app));
        match Command::new(&setsid)
            .arg(&uwsm_app)
            .arg("--")
            .arg(&browser)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => {
                log_line(&format!("uwsm spawn ok pid={}", child.id()));
                return Ok(());
            }
            Err(e) => {
                log_line(&format!("uwsm spawn failed: {}", e));
            }
        }
    } else {
        log_line("uwsm stack not found");
    }

    // 3) Fallback: setsid direct
    if let Some(setsid) = resolve_cmd("setsid", &["/usr/bin/setsid", "/bin/setsid"]) {
        log_line(&format!("trying setsid direct: {}", setsid));
        match Command::new(&setsid)
            .arg(&browser)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => {
                log_line(&format!("setsid direct spawn ok pid={}", child.id()));
                return Ok(());
            }
            Err(e) => {
                log_line(&format!("setsid direct failed: {}", e));
            }
        }
    } else {
        log_line("setsid not found");
    }

    // 4) Last fallback: direct
    log_line("trying direct browser spawn");
    match Command::new(&browser)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            log_line(&format!("direct spawn ok pid={}", child.id()));
            Ok(())
        }
        Err(e) => {
            log_line(&format!("direct spawn failed: {}", e));
            Err(format!("Failed to launch browser '{}': {}", browser, e))
        }
    }
}

fn launch_with_systemd_run(browser: &str, args: &[String]) -> Result<(), String> {
    let systemd_run = resolve_cmd(
        "systemd-run",
        &["/usr/bin/systemd-run", "/bin/systemd-run"],
    )
    .ok_or_else(|| "systemd-run not found".to_string())?;

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("time error: {}", e))?
        .as_millis();

    let unit = format!("edbookmark-webapp-{}", millis);

    let envs = collect_envs();

    log_line(&format!("trying systemd-run unit={}", unit));

    let mut cmd = Command::new(systemd_run);
    cmd.arg("--user")
        .arg("--quiet")
        .arg("--collect")
        .arg("--service-type=exec")
        .arg(format!("--unit={}", unit));

    for (k, v) in envs {
        cmd.arg(format!("--setenv={}={}", k, v));
    }

    cmd.arg(browser)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // IMPORTANT:
    // use status(), not spawn(), so systemd-run fully registers the service
    // before edbookmark exits. This is crucial for Walker-launched sessions.
    let status = cmd
        .status()
        .map_err(|e| format!("systemd-run exec failed: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("systemd-run returned status {}", status))
    }
}

fn collect_envs() -> Vec<(String, String)> {
    let keys = [
        "PATH",
        "HOME",
        "WAYLAND_DISPLAY",
        "DISPLAY",
        "XDG_RUNTIME_DIR",
        "DBUS_SESSION_BUS_ADDRESS",
        "XDG_CURRENT_DESKTOP",
    ];

    let mut out = Vec::new();
    for key in keys {
        if let Ok(val) = std::env::var(key) {
            if !val.is_empty() {
                out.push((key.to_string(), val));
            }
        }
    }
    out
}

fn resolve_browser() -> String {
    if let Some(desktop_name) = resolve_default_browser_desktop() {
        log_line(&format!("default_browser_desktop={}", desktop_name));
        if let Some(exec_bin) = resolve_exec_from_desktop(&desktop_name) {
            log_line(&format!("desktop_exec_bin={}", exec_bin));
            return exec_bin;
        }
    } else {
        log_line("xdg-settings returned no default browser");
    }

    for candidate in [
        "/usr/bin/chromium",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/google-chrome",
        "/usr/bin/brave-browser",
        "chromium",
    ] {
        if let Some(found) = resolve_cmd(candidate, &[candidate]) {
            log_line(&format!("fallback_browser={}", found));
            return found;
        }
    }

    "chromium".to_string()
}

fn resolve_default_browser_desktop() -> Option<String> {
    let xdg_settings = resolve_cmd("xdg-settings", &["/usr/bin/xdg-settings", "/bin/xdg-settings"])?;
    let out = Command::new(xdg_settings)
        .args(["get", "default-web-browser"])
        .output()
        .ok()?;

    let desktop = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if desktop.is_empty() {
        None
    } else {
        Some(desktop)
    }
}

fn resolve_exec_from_desktop(desktop_name: &str) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let paths = [
        format!("{}/.local/share/applications/{}", home, desktop_name),
        format!("{}/.nix-profile/share/applications/{}", home, desktop_name),
        format!("/usr/share/applications/{}", desktop_name),
    ];

    for path in paths {
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                if let Some(exec_line) = line.strip_prefix("Exec=") {
                    if let Some(token) = first_exec_token(exec_line) {
                        if let Some(found) = resolve_cmd(&token, &[&token]) {
                            return Some(found);
                        }
                        return Some(token);
                    }
                }
            }
        }
    }

    None
}

fn first_exec_token(exec_line: &str) -> Option<String> {
    let mut token = String::new();
    let mut in_quotes = false;

    for ch in exec_line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => break,
            c => token.push(c),
        }
    }

    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn resolve_uwsm_stack() -> Option<(String, String)> {
    let setsid = resolve_cmd("setsid", &["/usr/bin/setsid", "/bin/setsid"])?;
    let uwsm_app = resolve_cmd("uwsm-app", &["/usr/bin/uwsm-app", "/bin/uwsm-app"])?;
    Some((setsid, uwsm_app))
}

fn resolve_cmd(name: &str, abs_candidates: &[&str]) -> Option<String> {
    if name.contains('/') && Path::new(name).exists() {
        return Some(name.to_string());
    }

    for candidate in abs_candidates {
        if Path::new(candidate).exists() {
            return Some((*candidate).to_string());
        }
    }

    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let full = dir.join(name);
            if full.exists() {
                return Some(full.to_string_lossy().to_string());
            }
        }
    }

    None
}

fn log_line(msg: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = format!("{}/.local/state/edbookmark", home);
    let _ = fs::create_dir_all(&dir);
    let file = format!("{}/launcher.log", dir);

    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(file) {
        let _ = writeln!(f, "{}", msg);
    }
}

pub fn yank_to_clipboard(text: &str) -> Result<(), String> {
    use std::process::Stdio;

    let wl_copy = resolve_cmd("wl-copy", &["/usr/bin/wl-copy", "/bin/wl-copy"])
        .ok_or_else(|| "wl-copy not found".to_string())?;

    let mut child = Command::new(wl_copy)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run wl-copy: {}", e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("Failed to write to wl-copy: {}", e))?;
    }

    child
        .wait()
        .map_err(|e| format!("wl-copy failed: {}", e))?;

    Ok(())
}
