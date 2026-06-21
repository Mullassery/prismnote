use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
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
import sys, json, io, ast, base64, traceback, contextlib, signal

_ns = {"__name__": "__main__"}

# Ensure SIGINT raises KeyboardInterrupt even if we inherited SIG_IGN from a
# backgrounded parent (nohup/&/service) — this is what makes "stop cell" work.
try:
    signal.signal(signal.SIGINT, signal.default_int_handler)
except Exception:
    pass

# Pretty output + rich rendering by default. All optional — a missing library
# must never stop the kernel from starting.
def _bootstrap():
    try:
        import matplotlib
        matplotlib.use("Agg")   # headless: figures are captured, never displayed
    except Exception:
        pass
    try:
        import rich
        from rich import pretty
        pretty.install()
        _ns["rich"] = rich
        from rich.pretty import pprint as _pp
        _ns["pprint"] = _pp
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

    # Dynamic input widgets (Databricks dbutils.widgets-style). prism.input/slider/
    # select/checkbox render a control and return its current value; the value
    # persists across runs and is set by the UI before re-running the cell.
    class _Prism:
        def __init__(self):
            self._vals = {}
            self._widgets = []
        def _set(self, name, value):
            self._vals[name] = value
        def _w(self, spec, default):
            spec = dict(spec)
            val = self._vals.get(spec["name"], default)
            spec["value"] = val
            self._widgets.append(spec)
            return val
        def input(self, name, default=""):
            return self._w({"type": "text", "name": name}, default)
        def slider(self, name, min=0, max=100, default=None):
            return self._w({"type": "slider", "name": name, "min": min, "max": max},
                           default if default is not None else min)
        def select(self, name, options, default=None):
            opts = list(options)
            return self._w({"type": "select", "name": name, "options": opts},
                           default if default is not None else (opts[0] if opts else None))
        def checkbox(self, name, default=False):
            return self._w({"type": "checkbox", "name": name}, default)
    _ns["prism"] = _Prism()

_bootstrap()

def _mime_bundle(val):
    """Build a Jupyter-style MIME bundle for a value (text + optional HTML +
    structured table data so the UI can chart it)."""
    bundle = {"text/plain": repr(val)}
    fn = getattr(val, "_repr_html_", None)   # pandas DataFrame, etc.
    if callable(fn):
        try:
            html = fn()
            if html:
                bundle["text/html"] = html
        except Exception:
            pass
    # Structured payload for the chart switcher (DataFrames only, capped rows).
    try:
        import pandas as _pd
        if isinstance(val, _pd.DataFrame):
            bundle["application/vnd.prismnote.df+json"] = json.loads(
                val.head(500).to_json(orient="split", date_format="iso")
            )
    except Exception:
        pass
    return bundle

def _capture_figures(outputs):
    """Emit any open matplotlib figures as image/png display_data, then clear."""
    try:
        import matplotlib.pyplot as plt
    except Exception:
        return
    try:
        for num in plt.get_fignums():
            fig = plt.figure(num)
            buf = io.BytesIO()
            fig.savefig(buf, format="png", bbox_inches="tight", dpi=110)
            buf.seek(0)
            outputs.append({
                "output_type": "display_data",
                "data": {"image/png": base64.b64encode(buf.read()).decode("ascii")},
                "metadata": {},
            })
        plt.close("all")
    except Exception:
        pass

def _run(src):
    outputs = []
    out, err = io.StringIO(), io.StringIO()
    pr = _ns.get("prism")
    if pr is not None:
        pr._widgets = []  # collect widgets declared during this run
    try:
        tree = ast.parse(src, mode="exec")
        last = None
        if tree.body and isinstance(tree.body[-1], ast.Expr):
            last = ast.Expression(tree.body.pop().value)
        val = None
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            if tree.body:
                exec(compile(tree, "<cell>", "exec"), _ns)
            if last is not None:
                val = eval(compile(last, "<cell>", "eval"), _ns)
        so, se = out.getvalue(), err.getvalue()
        if pr is not None and getattr(pr, "_widgets", None):
            for w in pr._widgets:
                outputs.append({"output_type": "display_data",
                                "data": {"application/vnd.prismnote.widget+json": w}, "metadata": {}})
        if so:
            outputs.append({"output_type": "stream", "name": "stdout", "text": [so]})
        if se:
            outputs.append({"output_type": "stream", "name": "stderr", "text": [se]})
        _capture_figures(outputs)
        if last is not None and val is not None:
            outputs.append({"output_type": "execute_result", "data": _mime_bundle(val), "metadata": {}})
    except SystemExit:
        so = out.getvalue()
        if so:
            outputs.append({"output_type": "stream", "name": "stdout", "text": [so]})
    except BaseException:
        so = out.getvalue()
        if so:
            outputs.append({"output_type": "stream", "name": "stdout", "text": [so]})
        tb = traceback.format_exc()
        outputs.append({
            "output_type": "error",
            "ename": type(sys.exc_info()[1]).__name__,
            "evalue": str(sys.exc_info()[1]),
            "traceback": tb.splitlines(),
            "text": [tb],
        })
    return outputs

def _main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            src = json.loads(line)
        except Exception:
            continue
        outputs = _run(src)
        sys.stdout.write("__PRISM_RESULT__" + json.dumps({"outputs": outputs}) + "\n")
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
    /// Current interpreter PID, shared so the API can SIGINT a running cell
    /// without taking the kernel lock (which a running execute() holds).
    pid: Arc<AtomicI32>,
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
        let pid = Arc::new(AtomicI32::new(child.id().map(|p| p as i32).unwrap_or(0)));
        Ok(KernelManager {
            child: Some(child),
            stdin: Some(stdin),
            stdout: Some(stdout),
            kernel_id: Uuid::new_v4().to_string(),
            execution_count: 0,
            timeout: Duration::from_secs(60),
            pid,
        })
    }

    /// Shareable handle to the live interpreter PID (0 when not running).
    pub fn pid_handle(&self) -> Arc<AtomicI32> {
        self.pid.clone()
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
        self.pid.store(child.id().map(|p| p as i32).unwrap_or(0), Ordering::SeqCst);
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
                Err(anyhow!(
                    "Execution timed out after {:?} — kernel restarted, all variables cleared.",
                    self.timeout
                ))
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

    /// The driver already builds Jupyter-style output objects (stream,
    /// execute_result with a MIME bundle, display_data for figures, and error
    /// with a traceback). Just hand them through.
    fn build_outputs(res: Value) -> Result<Vec<Value>> {
        match res.get("outputs") {
            Some(Value::Array(a)) => Ok(a.clone()),
            _ => Ok(vec![]),
        }
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
