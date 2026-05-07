# ZEN: Forense de Procesos & Seguridad 💅

> **"¡Ándale! Aquí hay gente que no se ha presentado..."**  
> Zen no es solo un tracker; es un sistema forense de identidad para macOS con personalidad propia.

**ZEN** (v0.3.0) es un monitor de procesos minimalista diseñado bajo la filosofía *RedHat Pro* y *Suckless*. Su misión es proteger tu máquina identificando no solo qué corre, sino quién lo firmó criptográficamente.

## 💅 La Inspectora de Chismes
El alma de Zen es la **Inspectora de Chismes**. Ella no solo te da logs aburridos; ella clasifica la actividad de tu sistema en dos categorías:
- **✅ Chismes (Seguro):** Procesos conocidos y firmados por desarrolladores en los que confías (Google, Apple, Microsoft, Tailscale).
- **⚠️ Escándalos (Alerta):** Procesos sin firma, sospechosos o desconocidos que intentan pasar desapercibidos.

Zen utiliza `codesign` para verificar el **Team ID** y la **Authority** de cada binario, permitiéndote confiar en "Google LLC" de forma global sin tener que autorizar cada sub-proceso de Chrome uno por uno.

## 🚀 Quick Start (macOS)
1. Copia `allowlist.example` a `allowlist.txt` y edítalo.
2. Comandos principales:
   - **Daemon Mode:** `cargo run -- --daemon`  
     (Monitoreo continuo con reporte forense agrupado por Team ID).
   - **Status Check:** `target/debug/zen --status`  
     (Consulta la memoria (RSS), PID y reglas activas del daemon en tiempo real).
   - **Enforce:** `cargo run -- --enforce`  
     (Termina automáticamente cualquier proceso que no esté en la lista o no tenga una firma confiable).
   - **Stealth:** `cargo run -- --stealth`  
     (Solo reporta anomalías de CPU/RAM sin interrumpir).

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

## Export & Logging
```
--export-csv [FILE]     (default: export.csv)
--export-jsonl [FILE]   (default: export.jsonl)
--export-all-samples    (stealth mode only)
--audit-log [FILE]      (daemon mode only, default: audit.log)
```

## Safety
- `--enforce` uses `SIGKILL` and may require elevated privileges.

## License
`Process Tracker` is released under the **Process Tracker AI-Restricted License 1.0 (PT-ARL-1.0)**, a custom "Licencia de Código Fuente Abierto con Restricción de Uso para IA".  
En resumen: puedes usar, modificar y redistribuir el código, pero **debes atribuir/citar el proyecto y NO puedes usarlo para entrenar, ajustar o evaluar modelos de IA ni para crear datasets de entrenamiento para IA**. Consulta `LICENSE` para ver los términos completos.
