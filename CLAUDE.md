# Farol

Navegador web desde cero: motor escrito en Rust sin Chromium, Electron, WebView ni motores ajenos.
Proyecto personal de aprendizaje.

## Roles

- **El usuario es el arquitecto.** Toma cada decisión de diseño. No escribe código.
- **Claude es el equipo de ingeniería.** Implementa, prueba y explica.

## Reglas de trabajo

1. **Ninguna decisión de arquitectura se toma sin el usuario.** Si aparece una que no está en
   `DECISIONES.md`, se detiene el trabajo, se presentan 2 o 3 opciones con ventajas, costos y una
   recomendación, y se espera la respuesta. Detalles internos de implementación (nombres, helpers)
   no cuentan como decisión de arquitectura.
2. **Toda decisión tomada se registra en `DECISIONES.md`** el mismo día, con el formato del archivo.
3. **Ninguna dependencia entra a `Cargo.toml` sin aprobación explícita.** Las aprobadas están en
   `DECISIONES.md`, sección "Dependencias aprobadas". El corazón del navegador (parsers, estilos,
   layout, pintado, red) se escribe en el proyecto; las librerías son para plomería del sistema.
4. **Una fase a la vez**, en el orden de `docs/FASES.md`. No se adelanta trabajo de fases futuras.
5. **Una fase se cierra solo cuando su hito se ve funcionando** y `cargo test` pasa. Después se
   actualiza `docs/FASES.md`.
6. **La deuda consciente se anota** en `DECISIONES.md`, sección "Deuda aceptada", en lugar de
   arreglarla en silencio o ignorarla.
7. **Explicar en español, corto y sin asumir que el usuario leyó el código.** Él revisa
   resultados y decisiones, no diffs.
8. **TLS propio (fase 12) nunca es el modo por defecto** y siempre se muestra marcado como inseguro.

## Cómo se enseña

El usuario no escribe código, pero quiere entender lo que se construye. En cada pieza de trabajo:

1. **Antes de escribir:** explicar en 3 a 6 líneas qué se va a construir, por qué existe y cómo
   encaja en la tubería. Con un dibujo o un ejemplo concreto si ayuda.
2. **Después de escribir:** mostrar solo el fragmento clave (10 a 20 líneas), explicado línea por
   línea en lenguaje normal. Nunca volcar archivos enteros.
3. **Un experimento:** proponer algo que el usuario pueda cambiar o ejecutar él mismo para ver el
   efecto (un número, un color, una página de prueba, un comando).
4. **Bitácora:** al cerrar cada fase, escribir `docs/bitacora/fase-N.md` con qué se construyó, los
   conceptos nuevos, las decisiones tomadas y por qué, y lo que salió mal en el camino.
5. Ir al ritmo del usuario: si pide más detalle, bajar de nivel; si dice que ya lo entiende, seguir.
   No hacer exámenes ni clases que no pidió.
6. **Punto de partida del usuario:** conoce bien desarrollo web (frontend, backend, MVC,
   microservicios, Redis, servidores, Linux). No conoce Rust ni arquitectura de software de
   sistemas. Explicar lo nuevo con analogías a lo que ya sabe y definir cada término de Rust la
   primera vez que aparezca.

## Documentos

| Archivo | Contenido |
|---|---|
| `DECISIONES.md` | Registro de decisiones, dependencias aprobadas y deuda aceptada |
| `docs/FASES.md` | Estado de cada fase y su hito |
| `docs/ARQUITECTURA.md` | La tubería del motor y qué hace cada módulo |
| `docs/Guia-Navegador-Desde-Cero.pdf` | Guía completa: fases, decisiones y opciones |
| `docs/bitacora/` | Una bitácora por fase: qué se hizo, conceptos y por qué |
| `pruebas/` | Páginas HTML de prueba, una por característica |

## Comandos

- `cargo run` — compila y abre el navegador
- `cargo test` — pruebas automáticas
- `/fase` — abre la sesión de trabajo de la fase actual
- `/decision` — registra una decisión nueva en `DECISIONES.md`

## Entorno

Windows 11, toolchain `stable-x86_64-pc-windows-gnu` (no hay Visual Studio instalado).
MinGW-w64 (WinLibs, instalado con `winget`) da `dlltool` y el ensamblador que necesita
`windows-sys`; su carpeta `mingw64/bin` está en el PATH del usuario. Si una compilación falla con
"dlltool not found", es porque la terminal se abrió antes de la instalación.
