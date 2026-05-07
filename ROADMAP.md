# ZEN Roadmap 🚀

## 📍 ¿Dónde estamos? (v0.3.0 - Identity First)
Hemos transformado un rastreador básico en una herramienta de **monitoreo forense de identidad**.
- **Forense Criptográfico:** Verificación nativa de `TeamID` y `Authority` (macOS).
- **UX Forense:** Reportes agrupados por desarrollador y personalidad de "La Inspectora de Chismes".
- **OSINT Directo:** Hipervínculos OSC 8 para investigación instantánea en Google.
- **Automonitoreo:** IPC para consulta de estado (`--status`) y memoria en caliente.

## 🏃 ¿A dónde vamos? (v0.4.0 - Deep Visibility)
El siguiente salto es pasar del "quién" al **"qué"**. 
- **Transparencia de Intérpretes:** Identificar qué scripts exactos de Python, Node o Ruby se están ejecutando.
- **Argumentos Forenses:** Mostrar y permitir reglas basadas en los flags de ejecución.
- **Detección de Anomalías de Comportamiento:** Reglas dinámicas basadas en consumo de recursos (CPU/RAM).

## 🛠️ ¿Qué nos falta? (Backlog)
- [ ] **Visibilidad de Scripts:** Extraer y mostrar el entry point de lenguajes interpretados.
- [ ] **Reglas de Recursos:** `allowlist` con límites (ej. `cpu_max: 80%`).
- [ ] **Cuarentena:** Capacidad de congelar procesos (`SIGSTOP`) sin matarlos.
- [ ] **Persistencia de Caché:** Guardar firmas verificadas en disco para optimizar arranques.
- [ ] **Dashboard Forense:** Exportación estructurada para análisis de incidentes.
