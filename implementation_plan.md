# Plan de Exploración Forense: "El Ojo de la Inspectora"

## Objetivo
Potenciar las capacidades forenses de Zen para que la "Inspectora de Chismes" no solo sepa *quién* (ej. Python) está corriendo, sino *qué* está haciendo (ej. qué script exacto está ejecutando). Esto cierra una brecha de seguridad masiva donde intérpretes confiables pueden ejecutar código malicioso.

## Futuros Módulos (TODO List)
Antes de entrar en el plan forense, aquí están las ideas que discutimos, guardadas para el futuro:

- [ ] **Reglas Dinámicas por Rendimiento:** Poder establecer en la `allowlist` reglas como `cpu_max:80%` o `ram_max:500MB` para un proceso específico (ej. Chrome). Si las excede, la Inspectora lo mata, independientemente de si está firmado o no.
- [ ] **Cuarentena (Sandbox):** En lugar de matar el proceso, usar comandos de macOS para congelarlo (`SIGSTOP`) o revocarle permisos de red temporalmente mientras el usuario decide qué hacer.
- [ ] **Exportación Forense Estructurada:** Guardar un JSONL local con la cadena de ejecución completa (quién llamó a quién, con qué argumentos y a qué hora) para análisis post-mortem (útil en incidentes de seguridad reales).

---

## Plan de Acción Actual: Argumentos de Línea de Comandos

Actualmente, el tracker de macOS que programamos **ya extrae los argumentos** (`info.args`) usando la API `sysctl (KERN_PROCARGS2)`. ¡La data ya la tenemos! Solo que la Inspectora no la está chismeando.

### Cambios Propuestos

#### 1. Extracción de Argumentos Relevantes
Modificar `ReportItem` en `src/main.rs` para incluir un nuevo campo `args_summary`. 
- No podemos imprimir todos los argumentos de todos los procesos porque Chrome y Electron tienen decenas de flags ilegibles.
- **Solución Inteligente:** Si el proceso tiene argumentos, los unimos en una cadena. Si la cadena es muy larga (ej. > 80 caracteres), la truncamos con `...` para mantener el minimalismo. 
- **Filtrado:** Excluiremos los argumentos para procesos propios del sistema operativo que no aportan valor forense al usuario final, o simplemente los acortaremos limpiamente.

#### 2. Modificar la UI de la Inspectora
Actualizar la función `print_grouped_report` para inyectar los argumentos de forma elegante.
**Ejemplo Visual:**
```text
📦 TEAM: Team ID: EQHXZ8M8AV 
 └─ ✅ [1234] Python (/usr/bin/python3) -> logged [🔍 Investigar]
       ↳ 📜 Argumentos: python3 /Users/cairo/malicious.py --hidden
```
*Se agregaría una segunda línea indentada con el símbolo `↳` solo si hay argumentos válidos, manteniendo la estética limpia.*

#### 3. Actualizar `allowlist.example`
Documentar explícitamente cómo usar la regla `arg:` que ya existe en el código pero no estamos aprovechando. Para que puedas hacer cosas como:
`arg: /Users/cairo.g.r/scripts_seguros/`

## User Review Required

> [!IMPORTANT]
> **Pregunta de Diseño Visual:** ¿Te gusta la idea de poner los argumentos en una segunda línea con la flechita `↳ 📜 Argumentos: ...` o prefieres que se incluya todo en la misma línea principal (ej. `[PID] Python (path) [args...] -> logged`)? La segunda línea ayuda a mantenerlo compacto si los argumentos son largos.

¿Apruebas este plan para comenzar la implementación?
