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
| 3.1 | Subconjunto de CSS inicial | Medio | Decidida (delegada a Claude) | 2026-11-18 |
| 4.1 | Cómo se emparejan reglas con nodos | Ingenuo: cada regla contra cada nodo | Decidida (delegada a Claude) | 2026-11-18 |
| 5.1 | Colapso de márgenes | No, por ahora (deuda) | Decidida (delegada a Claude) | 2026-11-18 |
| 5.2 | Cómo se verifica el layout | Pruebas automáticas del árbol de cajas + render a imagen | Decidida (delegada a Claude) | 2026-11-18 |
| 6.1 | Transparencia | Opaco, con mezcla alfa simple para colores `rgba` | Decidida (delegada a Claude) | 2026-11-18 |
| 6.2 | Cómo se ve el resultado sin abrir la ventana | `farol --render` escribe un BMP con un escritor propio | Decidida (delegada a Claude) | 2026-11-18 |
| 7.1 | De dónde salen las letras | Fuente de mapa de bits propia, escalable | Decidida (delegada a Claude) | 2026-11-18 |
| 8.1 | Cómo se parten las líneas | Voraz | Decidida (delegada a Claude) | 2026-11-18 |
| 11.a | Adelantar el scroll de la fase 11 | Sí, a pedido del arquitecto: rueda, flechas, AvPág/RePág, Espacio, Inicio/Fin y barra | Decidida | 2026-11-18 |
| 11.b | Selección de texto y copiar | Arrastre, doble clic (palabra), Shift+clic, Ctrl+A, Escape; Ctrl+C al portapapeles de Windows con FFI propio, sin crates | Decidida (pedida por el arquitecto; implementación elegida por Claude) | 2026-11-18 |
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

### 3.1 a 8.1 — Delegadas en bloque
El 2026-11-18 el arquitecto pidió que las fases 3 a 8 las construyeran agentes y que Claude
eligiera. Se tomaron las recomendaciones de la guía: CSS medio, emparejamiento ingenuo, sin
colapso de márgenes, colores opacos con alfa simple, fuente de mapa de bits propia y partición de
líneas voraz. Se agregó 6.2 (render a BMP) para que los agentes puedan verificar visualmente sin
abrir la ventana. Los contratos entre módulos están en `docs/ARQUITECTURA.md`.

### 11.a y 11.b — Scroll y selección, adelantados
- **Origen:** al abrir el artículo, el arquitecto notó que no había scroll y pidió además poder
  seleccionar con el resaltado azul (arrastre, doble clic, Ctrl+A) y copiar.
- **Organización:** `ui` define el contrato `Pagina` (pintar y recibir mouse/teclado en
  coordenadas de página) y no sabe qué es una página. El módulo nuevo `navegador` implementa ese
  contrato: une la tubería con la selección, como el controlador de un MVC.
- **Portapapeles:** API de Windows (`OpenClipboard`, `SetClipboardData`…) declarada a mano en
  `src/ui/portapapeles.rs`. No entra ningún crate (0.2).

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
| 3 | Unidades `rem`, `vh`, `vw`, `ex`, `ch` no se soportan: la declaración se descarta | `font-size: 1.2rem` se ignora y queda el tamaño heredado | Cuando una página de prueba las use (`rem` es común) |
| 3 | La validación de valores es ligera: solo colores y longitudes. Las demás propiedades aceptan cualquier palabra (`display: banana` pasa) | Un valor inválido puede pisar a uno válido anterior, cuando en Chrome se ignoraría | Fase 4, al leer cada propiedad: tratar lo desconocido como inválido |
| 3 | Valores de varias piezas en propiedades no interpretadas se guardan como texto (`font-family`, `text-decoration: underline red`) | Ninguno por ahora; la fase 4 debe partir el texto si los necesita | Cuando una propiedad lo requiera |
| 3 | `background` solo conserva el color; imágenes, posiciones y varias capas se ignoran | Sin imágenes de fondo | Fase 10 (imágenes) |
| 3 | Sin `@media`, `@import`, `@font-face` ni selectores `>`, `+`, `~`, `:pseudo`, `[atributo]` (decisión 3.1) | Esas reglas no se aplican | Si una página real se ve mal por esto |
| 3 | Sin `var()` ni propiedades personalizadas, `calc()`, `hsl()` ni la lista completa de 148 nombres de color | Esos valores se descartan o quedan como texto | Cuando una página de prueba los use |
| 4 | `display` solo distingue bloque, en línea y ninguno: `flex`, `grid`, `list-item` y tablas se tratan como bloque; `inline-block` como en línea | Un `display: flex` apila los hijos uno debajo de otro; las tablas salen como bloques apilados; los `<li>` sin viñeta | Cuando una página de prueba use flex o tablas |
| 4 | Solo dos pesos de letra (≥ 600 es negrita); `bolder` siempre da negrita y `lighter` siempre normal | `font-weight: 500` se ve normal; `300` también | Fase 7, si la fuente tiene más pesos |
| 4 | `text-align: justify` se trata como izquierda | Texto justificado sale alineado a la izquierda | Sigue pendiente tras la fase 8: repartir lo que sobra entre los espacios de cada renglón (menos el último) |
| 4 | El estilo del borde solo dice si se ve o no; `dashed`, `dotted`, `double`, `inset`… se dibujarán sólidos | Bordes punteados salen continuos | Fase 6, si hace falta |
| 4 | `text-decoration` solo entiende `underline`; `line-through` y `overline` se ignoran | `<del>` y `<s>` salen sin tachar | Cuando una página de prueba lo use |
| 4 | Sin `:link` ni `[hidden]` en la hoja por defecto (no hay esos selectores): todo `<a>` es azul y subrayado, tenga o no `href`; `<p hidden>` se ve | Un `<a name="x">` sale azul | Si una página real se ve mal por esto |
| 4 | `line-height: normal` es siempre 1.2 × el tamaño de letra, sin mirar la fuente | Diferencias de 1 o 2 px por línea con Chrome | Fase 7 |
| 4 | Sin `font-family` ni `white-space`: `pre` y `code` no usan letra monoespaciada ni respetan saltos de línea | Un `<pre>` se ve como un párrafo | Sigue pendiente tras la fase 8: `white-space: pre` necesita no colapsar espacios y cortar en cada salto |
| 5 | Sin colapso de márgenes (decisión 5.1): los márgenes verticales de hermanos se suman, y el de un primer o último hijo no se fusiona con el del padre | Más espacio vertical que en Chrome: en `01-hola.html` el `h1` queda 8 px más abajo (body 8 + h1 21.44, Chrome se queda con 21.44) y entre párrafos hay 32 px en vez de 16 | Cuando se compare el render con Chrome (fase 6) o una página se vea muy espaciada |
| 5 | ~~El alto de las cajas anónimas es una estimación: ancho del texto en una línea / ancho disponible, hacia arriba, × `line-height`~~ | **Resuelta en la fase 8:** el texto se parte en renglones de verdad y la caja mide la suma de sus renglones | — |
| 5 | Un elemento en línea con un bloque adentro (`<a><div>…</div></a>`) se trata entero como bloque, con sus márgenes y relleno de bloque; CSS lo parte en dos pedazos en línea alrededor del bloque | El texto antes del bloque sale en su propia franja, como en Chrome, pero el fondo y el borde del `<a>` envuelven todo | Si una página real se ve mal por esto |
| 5 | Sin `box-sizing`: `width` siempre es el ancho del contenido (`content-box`); `border-box` se ignora | En páginas que usan `* { box-sizing: border-box }` las cajas con relleno o borde salen más anchas que en Chrome | Cuando una página de prueba lo use (es muy común en CSS moderno) |
| 5 | `height` en `%` se trata como `auto`; sin `min-width`, `min-height` ni `max-height` | `height: 100%` no estira la caja | Cuando una página de prueba lo use |
| 5 | Sin `position`, `float`, `overflow` ni márgenes negativos especiales: todo bloque se apila en el flujo normal; lo que se sale de un `height` fijo queda encima del siguiente | Una caja con `float: left` sale apilada a todo lo ancho | Después de la fase 8, si una página lo necesita |
| 6 | ~~Los elementos en línea (`span`, `a`, `b`) no pintan su fondo ni su borde~~ | **Resuelta en la fase 8:** cada elemento en línea pinta fondo y borde en cada renglón donde aparece (los `.boton` de `05-cascada.html` ya se ven) | — |
| 6 | Las esquinas del borde son de los lados de arriba y abajo; CSS las corta en diagonal entre los dos lados que se juntan | Con cuatro colores distintos, las esquinas toman el color de arriba o de abajo en vez de un corte en diagonal | Si una página con bordes de colores distintos se ve mal |
| 6 | Sin `border-radius`, sombras, degradados ni `opacity`: solo rectángulos de un color | Esquinas siempre rectas; botones "modernos" se ven cuadrados | Cuando una página de prueba lo use |
| 6 | El fondo del lienzo identifica `html` y `body` por posición (primer bloque y primer bloque de ese), no por etiqueta; y el parser no crea `<html>`/`<body>` si faltan | En una página sin `<html>` ni `<body>`, el fondo del primer `div` cubriría toda la ventana | Junto con la deuda de la fase 2 sobre `<html>/<body>` implícitos |
| 6 | Sin escalado por DPI: la página se mide en pixeles físicos de la pantalla | Con la escala de Windows al 150 %, la página se ve más chica que en Chrome | Fase 11 (interacción y ventana) |
| 6 | Cada repintado rehace el layout y la display list completos, aunque no haya cambiado el ancho | Ninguno a este tamaño; con páginas grandes, redimensionar podría ir lento | Si se nota lento al redimensionar o al hacer scroll (fase 11) |
| 6 | Lo que queda por debajo del alto de la ventana no se ve: no hay scroll todavía (el desplazamiento existe pero vale 0) | Las páginas largas se cortan abajo | Fase 11 |
| 7 | Sin suavizado (anti-aliasing): cada pixel está encendido o apagado | Bordes en escalera, más visibles a 24 px o más y en cursiva | Si el texto grande molesta; se puede suavizar al escalar |
| 7 | Una sola fuente: `font-family` se ignora; serif, sans y monoespaciada se ven iguales | `<code>` y `<pre>` no se ven monoespaciados | Si una página lo necesita (no entró en la fase 8) |
| 7 | Sin kerning ni ligaduras: el espacio entre letras es siempre 1 columna | Pares como "AV" o "To" quedan algo separados | Si se nota en páginas reales |
| 7 | La fuente está dibujada para 16 px; otros tamaños se agrandan o achican por vecino más cercano | A 12 px o menos algunas letras se apelmazan (la ñ, la m); a 24 px los trazos tienen 1,5 px y se ven desparejos | Si el texto chico de las páginas cuesta leerse: dibujar una segunda versión para tamaños chicos |
| 7 | Cobertura: ASCII, español y algunos signos tipográficos; el resto sale como "tofu" (un rectángulo hueco) | Letras como ç, à, ß, € o cualquier alfabeto no latino se ven como rectángulos | Cuando aparezcan en una página de prueba |
| 7 | Negrita y cursiva son sintéticas (doble trazo, inclinación) | Se ven menos finas que una negrita o cursiva diseñadas | No está previsto |
| 8 | Solo `vertical-align: baseline`; los elementos en línea no aportan su propio "puntal" al alto del renglón (solo su texto y el puntal del bloque) | `<sub>` y `<sup>` no suben ni bajan, solo se achican; un `<span>` vacío con letra grande no agranda el renglón | Cuando una página use `vertical-align`, `sub` o `sup` |
| 8 | Solo se parte en los espacios: nunca dentro de una palabra (sin guiones automáticos ni `overflow-wrap`), ni después de un `-` o una `/` | Una URL o palabra muy larga se sale por la derecha del bloque; Chrome tampoco la parte por defecto, pero sí parte en `a-b` | Si una página con URLs largas se ve rota |
| 8 | El fondo de un elemento en línea cubre la celda de la fuente (`font-size` de alto, desde el ascendente) más su relleno y borde verticales; Chrome usa el ascendente y descendente de la fuente real (≈ 1,15 × `font-size`) | Los fondos de `span` y `.boton` son 1 o 2 px más bajos que en Chrome | Si se nota al comparar |
| 8 | El subrayado es una raya de `tam/16` px un cuadrito debajo de la línea base, de punta a punta del fragmento; no salta los descendentes (`text-decoration-skip-ink`) e incluye la columna vacía después de la última letra | La raya cruza la cola de la g y la p, y se pasa 1 px a la derecha | Si molesta al leer |
| 8 | El texto ignora la transparencia de su color (`rgba` se pinta opaco), `letter-spacing`, `word-spacing`, `text-indent`, `text-transform` y la dirección derecha-a-izquierda | Esas propiedades no tienen efecto | Cuando una página de prueba las use |
| 8 | `inline-block` se trata como en línea: su `width`, `height` y márgenes verticales se ignoran | Un botón con `display: inline-block; width: 120px` mide lo que mida su texto | Cuando una página use `inline-block` (común en menús y botones) |
| 11 | Al cambiar el ancho de la ventana se borra la selección (los fragmentos se mueven) | Redimensionar quita el resaltado | Fase 11 |
| 11 | Arrastrar fuera de la ventana no hace scroll solo; la barra no se puede arrastrar con el mouse | Para seleccionar más de una pantalla hay que usar la rueda mientras se arrastra | Fase 11 |
| 11 | Sin triple clic (renglón o párrafo) | Solo palabra con doble clic | Si se extraña |
| 11 | El portapapeles solo funciona en Windows | En Linux o macOS Ctrl+C avisa que no está implementado | Si Farol se compila en otro sistema |
| 11 | Cada movimiento del mouse con el botón apretado rehace el layout completo al repintar | Ninguno visible en páginas chicas | Si se siente lento |

## Plantilla

```
### <código> — <título>
- **Opciones:** …
- **Elección:** …
- **Motivo:** …
- **Costo:** …
```
