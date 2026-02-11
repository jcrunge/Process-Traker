# Process Tracker

Minimal process tracker with strict allowlist, optional enforcement, stealth anomaly reporting, and optional CSV/JSONL export.

## Philosophy
- Minimalism and explicit behavior (suckless/bspwm-inspired)
- One job: observe, classify, and (optionally) terminate processes
- CLI-first, scriptable, no hidden state
- We build with faith in high-quality code and functionality: correctness and clarity are non-negotiable, and we keep our humor for the bugs

## Quick Start (macOS)
1. Copy `allowlist.example` to `allowlist.txt` and edit.
2. Run:
   - Report unknown: `cargo run -- --config allowlist.txt`
   - Enforce: `cargo run -- --config allowlist.txt --enforce`
   - Stealth: `cargo run -- --stealth`

## Allowlist Format
```
name:ProcessName
path:/full/path/to/executable
hash:sha256hex
uid:501
ppid:1
arg:--flag-or-substring
```

## Stealth Mode
- Detects CPU/RAM spikes and sustained anomalies.
- Use `--sustain-seconds` for time-based sustained detection.

## Export
```
--export-csv [FILE]     (default: export.csv)
--export-jsonl [FILE]   (default: export.jsonl)
--export-all-samples    (stealth mode only)
```

## Safety
- `--enforce` uses `SIGKILL` and may require elevated privileges.

## License
`Process Tracker` is released under the **Process Tracker AI-Restricted License 1.0 (PT-ARL-1.0)**, a custom "Licencia de Código Fuente Abierto con Restricción de Uso para IA".  
En resumen: puedes usar, modificar y redistribuir el código, pero **debes atribuir/citar el proyecto y NO puedes usarlo para entrenar, ajustar o evaluar modelos de IA ni para crear datasets de entrenamiento para IA**. Consulta `LICENSE` para ver los términos completos.
