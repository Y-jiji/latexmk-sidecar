#![feature(fs_try_exists)]

use regex::Regex;
use std::io::{Read, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::str::FromStr;

#[derive(Debug)]
enum Msg {
    UnknownInfo(String),
    UnknownError(String),
    NeedInstall(String),
    Empty,
}

impl Msg {
    fn new(s: String) -> Self {
        let ex = Regex::from_str(r"! LaTeX Error: File `(.*)' not found.").unwrap();
        if ex.is_match(&s) {
            let mut pkg = ex.captures(&s).unwrap().get(1).unwrap().as_str().split(".");
            let pkg = pkg.next().unwrap();
            return Msg::NeedInstall(pkg.to_string());
        }
        if s.starts_with("!") {
            return Msg::UnknownError(s);
        } else {
            return Msg::UnknownInfo(s);
        }
    }
    fn as_str(&self) -> &str {
        match &self {
            &Msg::UnknownError(e) => e.as_str(),
            &Msg::NeedInstall(e) => e.as_str(),
            &Msg::Empty => "",
            &Msg::UnknownInfo(i) => i.as_str(),
        }
    }
}

fn run_latexmk(args: Vec<String>) -> Result<Vec<Msg>, Box<dyn std::error::Error>> {
    let mut handle = Command::new("latexmk")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = handle.stdin.take().unwrap();
    while handle.try_wait()?.is_none() {
        stdin.write(b"\n").unwrap_or(0);
        stdin.flush().unwrap_or(());
    }
    let Output {
        stdout,
        stderr,
        status,
        ..
    } = handle.wait_with_output()?;
    let stdout = String::from_utf8(stdout).unwrap();
    let stderr = String::from_utf8(stderr).unwrap();
    let mut msg = Vec::new();
    for s in stdout.lines().map(|x| x.trim()) {
        msg.push(Msg::new(s.to_string()));
    }
    for s in stderr.lines().map(|x| x.trim()) {
        msg.push(Msg::new(s.to_string()));
    }
    println!("status: {status}");
    return Ok(msg);
}

fn run_tlmgr(pkg: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut handle = Command::new("tlmgr.bat").args(["install", &pkg]).spawn()?;
    handle.wait()?;
    return Ok(());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let file = args.pop()
        .expect("the file name should be the last argument");
    let root = file
        .strip_suffix(".tex")
        .expect("the file name should be the last argument and should be a *.tex file");
    let targ = root.to_string() + ".pdf";
    loop {
        let mut put = args.clone();
        put.push(String::from("-c"));
        put.push(file.clone());
        run_latexmk(put)?;
        let mut put = args.clone();
        put.push(file.clone());
        let msg = run_latexmk(put)?;
        for m in msg {
            if let Msg::NeedInstall(p) = m {
                run_tlmgr(p)?;
                println!("start tlmgr");
                break;
            } else {
                println!("{:?}", m.as_str());
            }
        }
        if std::fs::try_exists(targ.clone())? {
            break;
        }
    }
    return Ok(());
}
