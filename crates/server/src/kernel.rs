use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use uuid::Uuid;

/// A long-lived Python interpreter that keeps ONE shared namespace across every
/// cell, so variables/imports/functions defined in one cell are visible in the
/// next (proper notebook semantics). Code is fed to the driver over stdin as a
/// JSON-encoded string (one line per cell); the driver replies with a single
/// `__PRISM_RESULT__{...}` line carrying captured stdout/stderr, the repr of the
/// last expression, and any traceback.
const DRIVER: &str = r#"
import sys, json, io, ast, traceback, contextlib

_ns = {"__name__": "__main__"}

# Pretty output by default: rich's pretty-printer makes reprs of dicts/lists/
# objects readable, and pandas gets sane display widths. All optional — a missing
# library must never stop the kernel from starting.
def _bootstrap():
    try:
        import rich
        from rich import pretty
        pretty.install()                      # auto-pretty repr in the REPL
        _ns["rich"] = rich
        from rich.pretty import pprint as _pp
        _ns["pprint"] = _pp                   # `pprint(obj)` available everywhere
    except Exception:
        from pprint import pprint as _pp
        _ns["pprint"] = _pp
    try:
        import pandas as pd
        pd.set_option("display.max_columns", 50)
        pd.set_option("display.width", 120)
        _ns["pd"] = pd
    except Exception:
        pass

_bootstrap()

def _run(src):
    out, err = io.StringIO(), io.StringIO()
    res = {"stdout": "", "stderr": "", "result": None, "error": None}
    try:
        tree = ast.parse(src, mode="exec")
        last = None
        if tree.body and isinstance(tree.body[-1], ast.Expr):
            last = ast.Expression(tree.body.pop().value)
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            if tree.body:
                exec(compile(tree, "<cell>", "exec"), _ns)
            if last is not None:
                val = eval(compile(last, "<cell>", "eval"), _ns)
                if val is not None:
                    res["result"] = repr(val)
    except SystemExit:
        pass
    except BaseException:
        res["error"] = traceback.format_exc()
    res["stdout"], res["stderr"] = out.getvalue(), err.getvalue()
    return res

def _main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            src = json.loads(line)
        except Exception:
            continue
        res = _run(src)
        sys.stdout.write("__PRISM_RESULT__" + json.dumps(res) + "\n")
        sys.stdout.flush()

_main()
"#;

const RESULT_PREFIX: &str = "__PRISM_RESULT__";

pub struct KernelManager {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
    kernel_id: String,
    execution_count: usize,
    timeout: Duration,
}

impl KernelManager {
    pub fn new() -> Result<Self> {
        // Verify python is available before we commit to a long-lived process.
        let check = std::process::Command::new("python")
            .arg("-c")
            .arg("print('ok')")
            .output();
        if check.is_err() || !String::from_utf8_lossy(&check?.stdout).contains("ok") {
            return Err(anyhow!("python not found on PATH"));
        }

        let (child, stdin, stdout) = Self::spawn_process()?;
        Ok(KernelManager {
            child: Some(child),
            stdin: Some(stdin),
            stdout: Some(stdout),
            kernel_id: Uuid::new_v4().to_string(),
            execution_count: 0,
            timeout: Duration::from_secs(60),
        })
    }

    fn spawn_process() -> Result<(Child, ChildStdin, BufReader<ChildStdout>)> {
        let mut child = Command::new("python")
            .arg("-u") // unbuffered, so we see the result line immediately
            .arg("-c")
            .arg(DRIVER)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("failed to start python kernel: {}", e))?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("no kernel stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("no kernel stdout"))?;
        Ok((child, stdin, BufReader::new(stdout)))
    }

    /// Restart the interpreter, wiping the shared namespace. Used after a timeout
    /// (the old process is left in an unknown state) and for explicit restarts.
    pub fn restart(&mut self) -> Result<()> {
        if let Some(mut c) = self.child.take() {
            let _ = c.start_kill();
        }
        let (child, stdin, stdout) = Self::spawn_process()?;
        self.child = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(stdout);
        self.execution_count = 0;
        Ok(())
    }

    pub async fn execute(&mut self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        self.execution_count += 1;

        // pip installs run as a one-off so we don't block the shared interpreter.
        if code.trim().starts_with("pip install") || code.trim().starts_with("!pip") {
            return self.handle_package_install(code).await;
        }

        match tokio::time::timeout(self.timeout, self.execute_internal(code)).await {
            Ok(Ok(outputs)) => Ok((vec![], outputs)),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // The interpreter is mid-execution and the pipe is desynced; the
                // only safe recovery is a restart.
                let _ = self.restart();
                Err(anyhow!("Execution timed out after {:?}", self.timeout))
            }
        }
    }

    async fn execute_internal(&mut self, code: &str) -> Result<Vec<Value>> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("kernel not running"))?;

        // One JSON-encoded line per cell (newlines in code become \n inside the
        // JSON string, so the framing stays line-oriented).
        let msg = serde_json::to_string(code)?;
        stdin.write_all(msg.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        let reader = self
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow!("kernel not running"))?;

        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("kernel exited unexpectedly"));
            }
            if let Some(rest) = line.strip_prefix(RESULT_PREFIX) {
                let res: Value = serde_json::from_str(rest.trim_end())?;
                return Self::build_outputs(res);
            }
            // Anything else is stray driver chatter — ignore it.
        }
    }

    /// Turn the driver's JSON reply into Jupyter-style output objects. A Python
    /// exception is surfaced as an `Err` carrying the traceback, which the API
    /// layer renders as an `error` output the "Fix with AI" button can act on.
    fn build_outputs(res: Value) -> Result<Vec<Value>> {
        if let Some(tb) = res.get("error").and_then(|v| v.as_str()) {
            if !tb.is_empty() {
                return Err(anyhow!("{}", tb));
            }
        }

        let mut outputs = vec![];
        if let Some(s) = res.get("stdout").and_then(|v| v.as_str()) {
            if !s.is_empty() {
                outputs.push(json!({"output_type": "stream", "name": "stdout", "text": s}));
            }
        }
        if let Some(s) = res.get("stderr").and_then(|v| v.as_str()) {
            if !s.is_empty() {
                outputs.push(json!({"output_type": "stream", "name": "stderr", "text": s}));
            }
        }
        if let Some(r) = res.get("result").and_then(|v| v.as_str()) {
            outputs.push(json!({"output_type": "execute_result", "text": r}));
        }
        Ok(outputs)
    }

    async fn handle_package_install(&self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let clean_code = code.replace("!pip install", "pip install").replace("!pip", "pip");

        let output = Command::new("python")
            .arg("-m")
            .arg("pip")
            .arg("install")
            .args(clean_code.split_whitespace().skip(2))
            .output()
            .await
            .map_err(|e| anyhow!("Package installation failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut outputs = vec![];
        if !stdout.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stdout",
                "text": format!("✅ Package installed\n{}", stdout)
            }));
        }
        if !stderr.is_empty() {
            outputs.push(json!({"output_type": "stream", "name": "stderr", "text": stderr}));
        }
        Ok((vec![stdout], outputs))
    }

    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout = duration;
    }

    pub fn execution_count(&self) -> usize {
        self.execution_count
    }

    pub fn kernel_id(&self) -> &str {
        &self.kernel_id
    }
}

impl Drop for KernelManager {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
    }
}
