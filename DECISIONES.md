# Registro de decisiones

Cada decisión de arquitectura queda aquí. Los códigos (0.1, 9.1…) son los de la guía en
`docs/Guia-Navegador-Desde-Cero.pdf`.

Estados: **Decidida** · **Provisional** (la tomó Claude para no bloquear; falta que el arquitecto
la confirme) · **Pendiente**.

## Decisiones

| Código | Pregunta | Elección | Estado | Fecha |
|---|---|---|---|---|
| G.1 | Nombre del navegador | Farol | Decidida | 2026-11-18 |
| 0.1 | Lenguaje del motor | Rust | Decidida | 2026-11-18 |
| 0.2 | Política de dependencias | Mínimas y declaradas | Decidida | 2026-11-18 |
| 1.1 | Cómo se obtiene la ventana | `winit` + `softbuffer` | Decidida | 2026-11-18 |
| 1.2 | Con qué se pintan los pixeles | CPU | Decidida | 2026-11-18 |
| 0.3 | Toolchain de Rust en Windows | GNU (`x86_64-pc-windows-gnu`) | Provisional | 2026-11-18 |
| 0.4 | Organización del código | Un solo crate con un módulo por etapa | Provisional | 2026-11-18 |
| 2.1 | Tolerancia del parser de HTML | Tolerante simple | Decidida (delegada a Claude) | 2026-11-18 |
| 2.2 | Cómo se guarda el DOM en memoria | Arena con índices | Decidida (delegada a Claude) | 2026-11-18 |
| 9.1 | Resolución DNS | Cliente DNS propio sobre UDP | Decidida | 2026-11-18 |
| 9.4 | Inspección de tráfico | Wireshark, con captura guardada como evidencia del hito | Decidida | 2026-11-18 |
| 12.1 | HTTPS | Ambos: librería TLS por defecto y TLS 1.3 propio experimental | Decidida | 2026-11-18 |

## Detalle

### G.1 — Nombre: Farol
- **Motivo:** el arma de Jax, el campeón del arquitecto, es un farol; y un farol alumbra el camino.

### 0.1 — Lenguaje: Rust
- **Opciones:** Rust, C++, Zig o C.
- **Motivo:** rápido, sin runtime, y el compilador atrapa errores de memoria; encaja con un
  arquitecto que revisa resultados y no teclea.

### 0.2 — Dependencias mínimas y declaradas
- **Opciones:** cero absoluto, mínimas y declaradas, o libres.
- **Elección:** mínimas y declaradas. Entra una librería solo cuando el arquitecto la aprueba, y
  queda en "Dependencias aprobadas" con su motivo.
- **Historia:** el 2026-11-18 se registró primero "cero absoluto"; el mismo día el arquitecto
  aclaró que no está en contra de las librerías y eligió `winit` + `softbuffer` para la ventana.
- **Criterio:** lo que es el corazón del navegador (parsers, estilos, layout, pintado, red) se
  escribe en el proyecto. Las librerías son para plomería del sistema operativo.

### 1.1 — Ventana: winit + softbuffer
- **Opciones:** Win32 directo, `winit` + `softbuffer`, o `minifb`.
- **Motivo:** estándar en Rust y portable a Linux y macOS. Solo entregan la ventana y un búfer de
  pixeles; todo lo que se dibuja lo pinta Farol.

### 1.2 — Pintado por CPU
- **Opciones:** CPU o GPU.
- **Motivo:** con CPU se ve cómo cada caja se convierte en pixeles; con GPU eso pasa dentro de la
  tarjeta. La velocidad del CPU sobra para cajas y texto. La GPU queda como extra: la display list
  permite cambiar de pintor sin tocar el resto.
- **Historia:** Claude la aplicó primero como consecuencia de 1.1 sin confirmación; el arquitecto
  la confirmó después de ver las dos opciones.

### 0.3 — Toolchain GNU (provisional)
- **Opciones:** MSVC (requiere instalar Visual Studio Build Tools, varios GB y permisos de
  administrador) o GNU (trae su propio enlazador, sin instalar nada más).
- **Motivo:** el PC no tiene Visual Studio. GNU deja compilar de inmediato.
- **Costo:** algunas librerías de Windows se prueban primero con MSVC. Si en la fase 1 la ventana
  da problemas, se reconsidera.
- **Lo que pasó en la fase 1:** `winit` no compiló porque al toolchain GNU le faltaban `dlltool`
  y un ensamblador en el PATH. Se resolvió instalando MinGW-w64 (WinLibs, vía `winget`, sin
  permisos de administrador). Con eso GNU funciona y no hace falta cambiar a MSVC.

### 0.4 — Un solo crate (provisional)
- **Opciones:** un solo crate con módulos, o un workspace con un crate por etapa.
- **Motivo:** más simple; las fronteras entre módulos ya quedan limpias, así que separar después
  es fácil.
- **Estado:** recomendada por Claude y aplicada para no bloquear; falta confirmación.

### 2.1 — Parser de HTML tolerante simple
- **Opciones:** estricto, tolerante simple, o el algoritmo completo de WHATWG.
- **Motivo:** funciona con casi todas las páginas reales sin las semanas que cuesta el algoritmo
  oficial. Se puede endurecer después sin tocar el resto del motor.
- **Reglas que implementa:** los elementos vacíos (`br`, `img`…) no tienen hijos; un bloque nuevo
  cierra un `<p>` abierto; un `<li>` cierra el anterior (igual `dt`, `dd`, `tr`, `td`, `option`);
  un cierre cierra todo lo que quedó abierto adentro; un cierre sin apertura se ignora; el final
  del archivo cierra todo.
- **Delegada:** el arquitecto le pidió a Claude elegir.

### 2.2 — DOM en una arena con índices
- **Opciones:** arena con índices, punteros compartidos (`Rc<RefCell>`), o árbol solo hacia abajo.
- **Motivo:** todos los nodos en una lista y cada uno apunta a su padre e hijos por posición, como
  `padre_id` en una tabla SQL. Rápido y sin peleas con el compilador. Es lo que usa Servo.
- **Delegada:** el arquitecto le pidió a Claude elegir.

### 9.1 — Cliente DNS propio
- **Opciones:** resolvedor del sistema operativo o cliente propio sobre UDP.
- **Motivo:** el arquitecto quiere ver las redes por dentro; el paquete se arma byte a byte.

### 12.1 — HTTPS: ambos
- **Opciones:** solo librería, solo TLS propio, o ambos.
- **Motivo:** la librería permite navegar la web real con seguridad; el TLS propio es el reto de
  aprendizaje.
- **Regla fija:** con el TLS propio no se escriben contraseñas ni se entra a bancos o correo.

## Dependencias aprobadas

| Crate | Para qué | Decisión |
|---|---|---|
| `winit` | Crear la ventana y recibir eventos del sistema operativo | 1.1 |
| `softbuffer` | Entregar un búfer de pixeles y mostrarlo en la ventana | 1.1 |

## Deuda aceptada

| Fase | Qué falta | Efecto visible | Cuándo retomarlo |
|---|---|---|---|
| 2 | Algoritmo de "adopción" de WHATWG para etiquetas cruzadas (`<b><i></b></i>`) | En `<b>x <i>y</b> z</i>`, la "z" no sale en cursiva; en Chrome sí | Si una página real se ve mal por esto |
| 2 | No se crean `<html>`, `<head>` y `<body>` cuando faltan | Una página sin esas etiquetas tiene un árbol con otra forma que en Chrome | Fase 4, si los estilos por defecto lo necesitan |
| 2 | Solo se traducen unas pocas entidades con nombre (`&amp;`, `&lt;`, `&ntilde;`…) más todas las numéricas | `&eacute;` se muestra tal cual | Cuando aparezca en una página de prueba |

## Plantilla

```
### <código> — <título>
- **Opciones:** …
- **Elección:** …
- **Motivo:** …
- **Costo:** …
```
