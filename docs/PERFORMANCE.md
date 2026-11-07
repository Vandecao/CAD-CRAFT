# Performance and resilience checks

Normal workspace tests use small synthetic fixtures. The large workload is generated at runtime: 200 pages, 5,000 devices and 20,000 logical connections with drawn routes. No third-party project or generated binary belongs in Git.

Run the core workload in Release, optionally saving its generated project for the GUI benchmark:

```powershell
New-Item -ItemType Directory -Force .local/performance | Out-Null
$env:CADCRAFT_PERFORMANCE_PROJECT = "$PWD/.local/performance/synthetic.cadcraft"
cargo test -p cad-craft --release --locked --test large_project_performance -- --nocapture
cargo test -p cadcraft-gui --release --locked --test large_project_frames -- --nocapture
Remove-Item Env:CADCRAFT_PERFORMANCE_PROJECT
```

The facade test measures real disk save/open, page display-list generation, device and connection numbering, checks, PDF/DXF and report generation. The GUI test loads that same synthetic project and measures settled egui frames, wheel zoom and middle-button pan, asserting that those inputs really change the camera. It measures CPU frame construction, not GPU presentation or physical monitor latency. Run benchmarks without other builds or heavy workloads. Disk measurements may use the operating system's cache; they are not cold-storage guarantees.

The core large test runs automatically in Release. In Debug it returns before generating the large workload unless `CADCRAFT_PERFORMANCE` is set. The GUI large test runs only when `CADCRAFT_PERFORMANCE_PROJECT` identifies the generated fixture. Keep that variable unset for normal validation. Both suites still exercise small synthetic behavior in normal workspace tests.

Local acceptance goals are save/open under 3 seconds, page construction under 16 milliseconds, combined numbering under 2 seconds, and a 200-page PDF under 30 seconds. CI limits are intentionally looser (15 seconds for save/open, 160 milliseconds per frame, 10 seconds for numbering, 90 seconds for large exports/checks/reports) to detect regressions without assuming a specific runner. Printed target results and CI assertions are distinct: passing CI does not prove every local goal was met.

Idle native CPU must be measured separately in the Release executable after the UI settles and while the pointer, text caret and background tasks are inactive. Report process CPU time divided by wall time as a percentage of one core. Window-title commands are sent only when the title changes; a dirty project with recovery enabled schedules its next checkpoint, and active background workers schedule their own short wakeups.

Resilience tests run without opt-in:

```powershell
cargo test -p cad-craft --locked --test deterministic_fuzz
cargo test -p cadcraft-gui --locked --test recovery_process --test idle_frames --test screen_scales
```

Fuzz cases use a fixed seed and bounded mutations of valid archives/content plus random bytes. This is reproducible lightweight coverage, not exhaustive fuzzing. The recovery test starts and kills its own child process during an atomic write, then checks the exact last completed autosave. It never kills a user's CAD-CRAFT instance. Resolution tests drive the real egui application and accessibility tree; final visual acceptance still requires the manual checklist.
