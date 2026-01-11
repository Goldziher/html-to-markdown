#![allow(missing_docs)]

use anyhow::{Context, Result};
use html_to_markdown_rs::{ConversionOptions, HeadingStyle};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use wasmtime::{Engine, Instance, Memory, Module, Store, TypedFunc};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("e2e directory")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn rustup_available() -> bool {
    Command::new("rustup")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn rustup_rustc() -> Option<PathBuf> {
    if !rustup_available() {
        return None;
    }
    let output = Command::new("rustup")
        .args(["which", "rustc", "--toolchain", "stable"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn rustup_cargo() -> Option<PathBuf> {
    if !rustup_available() {
        return None;
    }
    let mut path = PathBuf::from(env::var("HOME").ok()?);
    path.push(".cargo/bin/cargo");
    if path.exists() { Some(path) } else { None }
}

fn cargo_invocation() -> Command {
    if let Some(managed) = rustup_cargo() {
        let mut cmd = Command::new(managed);
        cmd.arg("+stable");
        cmd
    } else if let Ok(current) = env::var("CARGO") {
        Command::new(current)
    } else {
        Command::new("cargo")
    }
}

fn wasm_target_installed() -> bool {
    let mut rustc_cmd = Command::new("rustc");
    if let Some(rustc) = rustup_rustc() {
        rustc_cmd = Command::new(rustc);
    }

    let output = rustc_cmd
        .args(["--print", "target-libdir", "--target", "wasm32-unknown-unknown"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let output = match output {
        Ok(output) => output,
        Err(_) => return false,
    };
    if !output.status.success() {
        return false;
    }

    let libdir = String::from_utf8_lossy(&output.stdout);
    let libdir_path = Path::new(libdir.trim());
    if !libdir_path.exists() {
        return false;
    }

    fs::read_dir(libdir_path)
        .map(|entries| {
            entries.into_iter().any(|entry| {
                entry
                    .map(|entry| entry.file_name().to_string_lossy().starts_with("libstd"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn build_wasm_module() -> Result<Option<PathBuf>> {
    if !wasm_target_installed() {
        eprintln!("Skipping WASM wasmtime tests: rust target wasm32-unknown-unknown not installed");
        return Ok(None);
    }

    let mut command = cargo_invocation();
    command.args([
        "build",
        "-p",
        "html-to-markdown-wasm",
        "--target",
        "wasm32-unknown-unknown",
        "--release",
        "--no-default-features",
        "--features",
        "wasmtime-testing",
    ]);
    if let Some(rustc) = rustup_rustc() {
        command.env("RUSTC", rustc);
    }

    let status = command
        .current_dir(workspace_root())
        .status()
        .context("unable to spawn cargo build for wasm artefact")?;
    if !status.success() {
        anyhow::bail!("building html-to-markdown-wasm failed");
    }
    let artefact = workspace_root().join("target/wasm32-unknown-unknown/release/html_to_markdown_wasm.wasm");
    if !artefact.exists() {
        anyhow::bail!("expected wasm artefact at {}", artefact.display());
    }
    Ok(Some(artefact))
}

struct WasmHarness {
    store: Store<()>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    dealloc: TypedFunc<(u32, u32), ()>,
    convert: TypedFunc<(u32, u32), u32>,
    convert_underlined: TypedFunc<(u32, u32), u32>,
    result_ptr: TypedFunc<(), u32>,
}

impl WasmHarness {
    fn new() -> Result<Option<Self>> {
        let engine = Engine::default();
        let wasm_path = match build_wasm_module()? {
            Some(path) => path,
            None => return Ok(None),
        };
        let module = Module::from_file(&engine, &wasm_path)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .context("wasm memory export missing")?;
        let alloc = instance
            .get_typed_func::<u32, u32>(&mut store, "htmd_alloc")
            .context("htmd_alloc export missing")?;
        let dealloc = instance
            .get_typed_func::<(u32, u32), ()>(&mut store, "htmd_dealloc")
            .context("htmd_dealloc export missing")?;
        let convert = instance
            .get_typed_func::<(u32, u32), u32>(&mut store, "htmd_convert")
            .context("htmd_convert export missing")?;
        let convert_underlined = instance
            .get_typed_func::<(u32, u32), u32>(&mut store, "htmd_convert_underlined")
            .context("htmd_convert_underlined export missing")?;
        let result_ptr = instance
            .get_typed_func::<(), u32>(&mut store, "htmd_result_ptr")
            .context("htmd_result_ptr export missing")?;

        Ok(Some(Self {
            store,
            memory,
            alloc,
            dealloc,
            convert,
            convert_underlined,
            result_ptr,
        }))
    }

    fn write_buffer(&mut self, bytes: &[u8]) -> Result<(u32, u32)> {
        let ptr = self.alloc.call(&mut self.store, bytes.len() as u32)?;
        self.memory
            .write(&mut self.store, ptr as usize, bytes)
            .context("failed to write into wasm memory")?;
        Ok((ptr, bytes.len() as u32))
    }

    fn free_buffer(&mut self, ptr: u32, len: u32) -> Result<()> {
        self.dealloc
            .call(&mut self.store, (ptr, len))
            .context("failed to free wasm memory")
    }

    fn read_result(&mut self, len: u32) -> Result<String> {
        let ptr = self
            .result_ptr
            .call(&mut self.store, ())
            .context("failed to fetch result pointer")?;
        let mut buffer = vec![0u8; len as usize];
        self.memory
            .read(&mut self.store, ptr as usize, &mut buffer)
            .context("unable to read result bytes from wasm memory")?;
        Ok(String::from_utf8(buffer)?)
    }

    fn read_markdown(&mut self, len: u32) -> Result<String> {
        let contents = self.read_result(len)?;
        if let Some(rest) = contents.strip_prefix("ERROR:") {
            anyhow::bail!("conversion failed inside wasm: {}", rest);
        }
        Ok(contents)
    }

    fn convert_html(&mut self, html: &str) -> Result<String> {
        let (ptr, len) = self.write_buffer(html.as_bytes())?;
        let out_len = self
            .convert
            .call(&mut self.store, (ptr, len))
            .context("htmd_convert trap")?;
        self.free_buffer(ptr, len)?;
        self.read_markdown(out_len)
    }

    fn convert_underlined(&mut self, html: &str) -> Result<String> {
        let (ptr, len) = self.write_buffer(html.as_bytes())?;
        let out_len = self
            .convert_underlined
            .call(&mut self.store, (ptr, len))
            .context("htmd_convert_underlined trap")?;
        self.free_buffer(ptr, len)?;
        self.read_markdown(out_len)
    }
}

#[test]
fn converts_simple_html_via_wasmtime() -> Result<()> {
    let Some(mut harness) = WasmHarness::new()? else {
        return Ok(());
    };
    let html = "<h1>Hello</h1><p>Rust + WASM</p>";
    let output = harness.convert_html(html)?;
    let expected = html_to_markdown_rs::convert(html, None)?;
    assert_eq!(output.trim(), expected.trim());
    Ok(())
}

#[test]
fn respects_conversion_options() -> Result<()> {
    let Some(mut harness) = WasmHarness::new()? else {
        return Ok(());
    };
    let html = "<h1>Title</h1><p>content here</p>";
    let options = ConversionOptions {
        heading_style: HeadingStyle::Underlined,
        wrap: true,
        wrap_width: 12,
        ..Default::default()
    };
    let expected = html_to_markdown_rs::convert(html, Some(options.clone()))?;

    let output = harness.convert_underlined(html)?;
    assert_eq!(output.trim(), expected.trim());
    Ok(())
}
