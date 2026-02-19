# 🛡️ Daemon Mode — Process Tracker

> *"Take the power back into your hands. Know what's running on your machine, even when you're not looking."*

## ¿Qué es el Modo Daemon?

El modo daemon convierte Process Tracker en un **centinela permanente**.
En lugar de escanear una vez y salir, el daemon vigila tu sistema en un loop continuo:

```
Iniciar → Escanear procesos → ¿Conocido? → Sí: dejarlo
                                          → No: registrar alerta
                              → ¿Sistema sobrecargado? → alertar
         → Dormir → Repetir ∞
```

Cada proceso pasa por tres filtros:

| Filtro | ¿Qué evalúa? | Acción si no pasa |
|---|---|---|
| **Self-check** | ¿Es el propio tracker? | Ignorar (nunca alertar sobre sí mismo) |
| **Sistema macOS** | ¿Vive en `/System/`, `/usr/`, `/sbin/`? | Monitorear recursos, **nunca matar** |
| **Allowlist/Perfil** | ¿Está en la lista blanca activa? | Registrar como desconocido |

---

## Inicio Rápido

### 1. Observar (modo seguro)

```bash
# Escanea cada 2 segundos, registra desconocidos en audit.log
process_tracker --daemon --config allowlist.txt --audit-log audit.log --interval 2000
```

Revisa el log:
```bash
cat audit.log
```

Agrega procesos legítimos a tu allowlist:
```bash
echo "name:Brave Browser" >> allowlist.txt
echo "path:/Applications/Docker.app/Contents/MacOS/Docker Desktop" >> allowlist.txt
```

### 2. Confiar

Repite hasta que `audit.log` muestre **cero falsos positivos**.
Tu allowlist ahora cubre todos los procesos normales de tu Mac.

### 3. Enforce (con precaución)

```bash
# Ahora sí: mata procesos realmente desconocidos
process_tracker --daemon --config allowlist.txt --audit-log audit.log --enforce
```

> ⚠️ **Solo usar enforce cuando confías en tu allowlist.**
> macOS tiene ~488 procesos de sistema. Con una allowlist inmadura,
> `--enforce` podría matar procesos del SO y dejar tu Mac inestable.

---

## Detección de Procesos macOS

El daemon **reconoce automáticamente** los procesos del sistema operativo
por su ruta de ejecutable:

```
/System/Library/...     → Sistema
/usr/libexec/...        → Sistema
/usr/sbin/...           → Sistema
/usr/bin/...            → Sistema
/sbin/...               → Sistema
/Library/Apple/...      → Sistema
Todo lo demás           → Usuario (requiere allowlist)
```

Esto cubre **~87% de los procesos** sin necesidad de agregarlos manualmente.

### Alertas de sobrecarga

Los procesos de sistema **no se matan**, pero si uno supera los umbrales
de CPU o RAM, el daemon te alerta:

```
system-overload pid=371 name=mds cpu=95.20 ram=12.30
```

Puedes ajustar los umbrales:

```bash
process_tracker --daemon --cpu-threshold 90 --ram-threshold 25 ...
```

Para monitorear procesos de sistema como cualquier otro (sin auto-detección):

```bash
process_tracker --daemon --no-ignore-system ...
```

---

## Perfiles

Un perfil es simplemente un archivo de allowlist con un nombre descriptivo.
Viven en el directorio `profiles/`:

```
profiles/
├── reposo.txt               # Solo SO — nada de usuario
├── sin-distracciones.txt    # Terminal + editor, nada más
├── escritura.txt            # Terminal + editor + browser (docs)
└── compilando.txt           # Todo lo de desarrollo
```

### Usar un perfil

```bash
# Equivale a: --config profiles/sin-distracciones.txt
process_tracker --daemon --profile sin-distracciones
```

### Crear un perfil

Usa el mismo formato de `allowlist.txt`:

```bash
# profiles/sin-distracciones.txt
name:iTerm2
name:nvim
name:tmux
path:/Applications/Cursor.app/Contents/MacOS/Cursor
```

```bash
# profiles/compilando.txt
name:iTerm2
name:nvim
name:cargo
name:rustc
name:node
path:/Applications/Docker.app/Contents/MacOS/Docker Desktop
path:/Applications/Brave Browser.app/Contents/MacOS/Brave Browser
```

```bash
# profiles/reposo.txt
# Vacío o solo con procesos críticos de usuario
# Todos los procesos de sistema ya se detectan automáticamente
name:Finder
```

### Flujo recomendado

```
                    ┌───────────────────┐
                    │  Descubre tu Mac  │
                    │  (modo observar)  │
                    └────────┬──────────┘
                             │
                    ┌────────▼──────────┐
                    │  Clasifica apps   │
                    │  por actividad    │
                    └────────┬──────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
      ┌──────────────┐ ┌──────────┐ ┌────────────┐
      │   reposo     │ │ escribir │ │  compilar  │
      │  (mínimo)    │ │ (medio)  │ │  (amplio)  │
      └──────────────┘ └──────────┘ └────────────┘
```

1. Corre el daemon en modo observar y revisa qué procesos usa tu Mac
2. Clasifica cada app según en qué perfil debe estar
3. Crea los perfiles y úsalos según lo que estés haciendo

---

## Audit Log

El audit log registra dos tipos de eventos:

### Proceso desconocido
```json
{"ts":1708300000,"kind":"audit","pid":1234,"uid":501,"ppid":1,"name":"SuspiciousApp","path":"/tmp/suspicious","action":"logged"}
```

### Sobrecarga de sistema
```json
{"ts":1708300000,"kind":"system-overload","pid":371,"name":"mds","path":"/System/.../mds","cpu":95.20,"ram":12.30}
```

El archivo también se exporta en **CSV** si usas `--export-csv`.

---

## Deduplicación

El daemon **no te spamea**. Cada proceso desconocido se reporta **una sola vez**.
Si el proceso desaparece y vuelve a aparecer, se reporta de nuevo.

```
# Primer scan: alerta
unknown pid=1234 name=SuspiciousApp path=/tmp/sus

# Segundo scan: silencio (ya reportado)

# ... proceso 1234 muere ...

# Proceso reaparece con nuevo PID:
unknown pid=5678 name=SuspiciousApp path=/tmp/sus
```

---

## Referencia CLI Completa

```
process_tracker --daemon [OPTIONS]

Opciones del daemon:
  --daemon              Activar modo daemon (loop continuo)
  --profile NAME        Cargar profiles/NAME.txt como allowlist
  --audit-log [FILE]    Log de eventos (default: audit.log)
  --no-ignore-system    No auto-detectar procesos de macOS
  --interval MS         Intervalo de escaneo en ms (default: 1000)
  --enforce             Matar procesos desconocidos

Opciones de umbrales (para alertas de sistema):
  --cpu-threshold PCT   Umbral de CPU para alerta (default: 80)
  --ram-threshold PCT   Umbral de RAM para alerta (default: 20)

Opciones existentes:
  --config FILE         Archivo de allowlist (default: allowlist.txt)
  --stealth             Modo stealth (anomalías, independiente del daemon)
  --export-csv [FILE]   Exportar CSV
  --export-jsonl [FILE] Exportar JSONL
```

---

## Filosofía

Este daemon sigue las reglas del proyecto al pie de la letra:

- **Un solo trabajo**: observar, clasificar, y (opcionalmente) terminar procesos
- **Minimalismo**: cero dependencias nuevas, cero frameworks
- **Sin comportamiento oculto**: todo se activa con flags explícitos
- **CLI-first, scriptable**: `process_tracker --daemon --profile reposo &`
- **Rápido y liviano**: `starts_with()` sobre strings, `sleep()` entre scans
- **Fail fast**: errores precisos, sin estados ambiguos
