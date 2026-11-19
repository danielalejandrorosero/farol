# Arquitectura

El motor es una tubería. Cada etapa recibe una cosa y entrega otra; ninguna conoce a las que
vienen después.

```
URL → red → bytes → parsers → estilos → layout → display list → pintado → ventana
```

## Módulos

| Módulo | Recibe | Entrega | Fase |
|---|---|---|---|
| `red` | Una URL | Bytes de HTML, CSS e imágenes | 9, 10, 12 |
| `html` | Texto HTML | Árbol DOM | 2 |
| `css` | Texto CSS | Lista de reglas | 3 |
| `estilo` | DOM y reglas | Árbol de estilo: cada nodo con sus propiedades finales | 4 |
| `layout` | Árbol de estilo y ancho de ventana | Árbol de cajas con posición y tamaño | 5, 8 |
| `pintado` | Árbol de cajas | Display list y pixeles en el framebuffer | 6, 7 |
| `ui` | Eventos de ventana, mouse y teclado | Scroll, clics, navegación | 1, 11 |

## Reglas de dependencia entre módulos

- Las flechas van en un solo sentido: `layout` usa `estilo`, nunca al revés.
- `red` no sabe nada de HTML; solo entrega bytes y cabeceras.
- `ui` es el único módulo que conoce la ventana del sistema operativo.

## Por dentro de `html`

- `tokenizador.rs`: recorre el texto carácter por carácter y lo corta en tokens (abre, cierra,
  texto, comentario). El contenido de `style`, `script`, `title` y `textarea` se toma crudo.
- `constructor.rs`: arma el árbol con una pila de elementos abiertos y aplica las reglas de
  tolerancia (decisión 2.1).
- `dom.rs`: la arena (decisión 2.2). `NodoId` es la posición de un nodo en la lista.

## Por dentro de `css`

- `tokenizador.rs`: corta el texto en tokens (identificadores, números con unidad, `#hash`,
  cadenas, funciones, `!important`, símbolos y espacios). Los comentarios desaparecen aquí.
- `parser.rs`: arma las reglas y hace la recuperación de errores (salta hasta `;` o hasta la `}`
  balanceada). Anota cada cosa descartada como aviso.
- `selectores.rs`: etiqueta, clase, id, `*` y combinador descendiente.
- `valores.rs`: interpreta valores y colores, con una validación ligera por tipo de propiedad.
- `abreviadas.rs`: expande `margin`, `padding`, `border*` y `background`.
- `tipos.rs`: los tipos del contrato.

## Por dentro de `estilo`

- `por_defecto.css`: la hoja del navegador (lo que hace grande a un `h1` sin CSS del autor).
- `emparejar.rs`: ¿este selector le aplica a este elemento? Ingenuo (decisión 4.1), de derecha a
  izquierda, subiendo por `Nodo::padre` para el combinador descendiente.
- `cascada.rs`: junta las declaraciones que le aplican a un elemento y las ordena por
  (importante, origen, especificidad, orden). Guarda todas las candidatas, no solo la ganadora.
- `computar.rs`: convierte lo que ganó en valores finales, con herencia, `em`, `currentcolor`.
- `tipos.rs`: los tipos del contrato. `imprimir.rs`: el texto de `farol --estilo`.

## Contratos entre módulos (fases 3 a 8)

Escritos antes de construir, para que cada fase encaje con las demás. Los nombres pueden
ajustarse si hay una buena razón, pero la forma (quién recibe qué) se respeta. Si un contrato
cambia, se actualiza aquí.

### `css` (fase 3)
```rust
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }      // a = 255 es opaco
pub enum Unidad { Px, Em, Porcentaje }
pub enum Valor { Palabra(String), Longitud(f32, Unidad), Numero(f32), Color(Color) }
pub struct Declaracion { pub propiedad: String, pub valor: Valor, pub importante: bool }
pub struct SelectorSimple { pub etiqueta: Option<String>, pub id: Option<String>, pub clases: Vec<String> }
pub struct Selector { pub cadena: Vec<SelectorSimple> }   // combinador descendiente, izq. a der.
impl Selector { pub fn especificidad(&self) -> (u32, u32, u32) }  // (ids, clases, etiquetas)
pub struct Regla { pub selectores: Vec<Selector>, pub declaraciones: Vec<Declaracion> }
pub struct HojaDeEstilos { pub reglas: Vec<Regla> }
pub fn parsear(css: &str) -> HojaDeEstilos
pub fn parsear_declaraciones(texto: &str) -> Vec<Declaracion>   // para el atributo style=""
pub fn parsear_con_avisos(css: &str) -> (HojaDeEstilos, Vec<String>)  // lo que usa `--css`
```
- Implementado así en la fase 3. Agregados al contrato: `parsear_con_avisos` y `Color::rgb`;
  `HojaDeEstilos::imprimir()` y `Display` para todos los tipos.
- Las propiedades y palabras clave llegan en minúsculas; clases e ids, tal como se escribieron.
- `Valor::Palabra` guarda las palabras clave y, en propiedades que Farol no interpreta, el valor
  completo como texto (`font-family` → `"Helvetica Neue", Arial, sans-serif`).
- `0` sin unidad en una propiedad de longitud llega como `Longitud(0, Px)`; en las demás
  (`line-height: 0`) como `Numero(0)`. `pt`, `pc`, `in`, `cm`, `mm` llegan ya convertidas a px.
  `rem`, `vh`, `vw` y similares no son válidas todavía: la declaración se descarta.
- Validación ligera: las propiedades de color solo aceptan colores, `currentcolor` y las globales
  (`inherit`, `initial`, `unset`); las de longitud, longitudes o palabras; `width`, `height`,
  `padding-*`, `font-size` no aceptan negativos. Un valor inválido descarta la declaración.
- Las abreviadas se expanden al parsear: `margin`, `padding` → `-top/-right/-bottom/-left`;
  `border`, `border-width/-color/-style`, `border-top`… → `border-{lado}-{width,color,style}`;
  `background: <color>` → `background-color`.
- Como en CSS, lo que una abreviada no menciona vuelve a su valor inicial: `border: 1px solid`
  emite también `border-*-color: currentcolor`; sin ancho emite `medium`, sin estilo `none`.
  `background` sin color emite `background-color: transparent`. La fase 4 debe entender
  `medium` (3px), `thin` (1px), `thick` (5px) y `currentcolor` (el `color` del elemento).
- Selectores no soportados (`>`, `+`, `~`, `:hover`, `[attr]`, `::before`) descartan la regla
  entera. Los bloques `@media`, `@font-face`, etc. se saltan completos.

### `estilo` (fase 4)
```rust
pub enum Display { Bloque, EnLinea, Ninguno }
pub enum Longitud { Auto, Px(f32), Porcentaje(f32) }
pub enum Alineacion { Izquierda, Centro, Derecha }
pub struct Lados<T> { pub arriba: T, pub derecha: T, pub abajo: T, pub izquierda: T }
pub struct Estilo {
    pub display: Display,
    pub color: Color, pub fondo: Color,                    // css::Color
    pub ancho: Longitud, pub alto: Longitud, pub ancho_maximo: Longitud,
    pub margen: Lados<Longitud>, pub relleno: Lados<Longitud>,
    pub borde_ancho: Lados<f32>, pub borde_color: Lados<Color>,
    pub tam_fuente: f32, pub negrita: bool, pub cursiva: bool, pub subrayado: bool,
    pub alineacion: Alineacion, pub alto_linea: f32,       // en px ya resueltos
}
pub struct NodoEstilado { pub nodo: NodoId, pub texto: Option<String>, pub estilo: Estilo, pub hijos: Vec<NodoEstilado> }
pub fn hojas_del_documento(doc: &Documento) -> Vec<HojaDeEstilos>   // contenido de los <style>
pub fn estilar(doc: &Documento, hojas_autor: &[HojaDeEstilos]) -> NodoEstilado
```
- Implementado así en la fase 4. Agregados al contrato: `Lados::todos/en_orden/desde_orden`,
  `Estilo::inicial()`, `Estilo::para_texto(&padre)`, `NodoEstilado::imprimir(&doc)` y
  `estilo::explicar(doc, hojas, selector)` (lo que usa `--estilo <html> <selector>`).
- La raíz (`NodoEstilado` del nodo `Documento`) tiene el estilo inicial con `display: Bloque`.
- Los nodos de texto llevan `texto: Some(..)`, `display: EnLinea`, las propiedades heredables del
  padre y el resto en su valor inicial (sin márgenes, relleno, bordes ni fondo). Los textos de solo
  espacios se conservan: los colapsa la fase 8.
- `ancho_maximo: Auto` significa `max-width: none`.
- `alto_linea` ya está en px. `line-height: 1.5` se hereda como factor (cada hijo multiplica por
  su propia letra); `line-height: 24px` o `1.5em` se heredan como px fijos. `normal` = 1.2.
- `subrayado` incluye el de los ancestros (en CSS la decoración se propaga y no se puede quitar).
- `display` se reduce a tres valores: `flex`, `grid`, `list-item` y las partes de tabla se
  tratan como `Bloque`; `inline-block` como `EnLinea`. Un valor que no se entiende se ignora y
  vale la siguiente declaración de la cascada.
- Hoja por defecto del navegador en `src/estilo/por_defecto.css`, cargada con `include_str!`.
- `em` se resuelve a px aquí. Los `%` se quedan como `Porcentaje` para que los resuelva el layout.
- Un borde sin `border-style` (o `none`) tiene ancho 0, como en CSS.
- Los nodos con `display: none` y sus hijos no aparecen en el árbol.

### `layout` (fases 5 y 8)
```rust
pub struct Rect { pub x: f32, pub y: f32, pub ancho: f32, pub alto: f32 }
pub struct Dimensiones { pub contenido: Rect, pub relleno: Lados<f32>, pub borde: Lados<f32>, pub margen: Lados<f32> }
impl Dimensiones { pub fn caja_relleno(&self) -> Rect; pub fn caja_borde(&self) -> Rect; pub fn caja_margen(&self) -> Rect }
pub enum TipoCaja<'a> { Bloque(NodoId), Anonima(Vec<&'a NodoEstilado>) }
pub struct Caja<'a> { pub tipo: TipoCaja<'a>, pub estilo: Cow<'a, Estilo>, pub dimensiones: Dimensiones, pub hijos: Vec<Caja<'a>> }
pub fn calcular<'a>(raiz: &'a NodoEstilado, ancho_ventana: f32) -> Caja<'a>
```
- Implementado así en la fase 5. Ajustes al contrato, con su motivo:
  - `TipoCaja::Bloque` lleva el `NodoId` (para saber la etiqueta al imprimir o depurar) y
    `TipoCaja::Anonima` lleva los nodos en línea que envuelve, tal cual: es la entrada de la fase 8.
  - `estilo` es `Cow<'a, Estilo>` (prestado *o* propio; se lee igual que una referencia). Las
    cajas de bloque prestan el estilo de su nodo; las anónimas no tienen nodo y llevan uno propio:
    lo heredable del padre (letra, alto de línea, alineación) y lo demás inicial (sin fondo, ni
    bordes, ni márgenes), para que el pintado no repita el fondo del padre.
  - Agregados: `Rect::expandido(&Lados<f32>)` y `Caja::imprimir(&doc)` (lo que usa `--layout`).
- `contenido` es el rectángulo de contenido en coordenadas de página (0, 0 = esquina de arriba a la
  izquierda de la página; y crece hacia abajo). La raíz es la caja de `#documento`, en (0, 0), con
  el ancho de la ventana y el alto de toda la página.
- Todos los `%` y `auto` ya están resueltos a px en `Dimensiones`.
- Sin colapso de márgenes (decisión 5.1): entre hermanos queda la suma de los dos márgenes.
- Un elemento en línea que tiene un bloque adentro se trata entero como bloque.
- Fase 5: el contenido en línea se envuelve en cajas anónimas con un alto provisional
  (líneas estimadas × `alto_linea`); las anónimas no tienen hijos.
- Fase 8: las cajas anónimas guardan líneas con fragmentos de texto posicionados (x, y, ancho,
  alto, texto, estilo), medidos con `pintado::fuente`.

### `pintado` (fases 6 y 7)
```rust
pub enum Orden { Rectangulo { rect: Rect, color: Color }, Texto { /* fase 8 */ } }
pub fn lista_de_pintado(raiz: &Caja) -> Vec<Orden>
impl Lienzo { pub fn ejecutar(&mut self, ordenes: &[Orden]) }
// fase 7, en pintado::fuente:
pub fn ancho_texto(texto: &str, tam: f32, negrita: bool) -> f32
pub fn alto_linea_natural(tam: f32) -> f32          // 1,25 × tam
pub fn linea_base(tam: f32) -> f32                  // tope de la caja de línea → línea base
pub fn ascendente(tam: f32) -> f32                  // tope de la celda → línea base (13/16 × tam)
pub fn descendente(tam: f32) -> f32                 // línea base → fondo de la celda (3/16 × tam)
pub fn tiene_glifo(c: char) -> bool
impl Lienzo { pub fn texto(&mut self, x: f32, y: f32, texto: &str, tam: f32, color: u32, negrita: bool, cursiva: bool) }
```
- `y` es el **tope de la caja de línea** (de alto `alto_linea_natural(tam)`), no la línea base.
  Con otro `line-height`, el layout corre `y` la mitad de la diferencia.
- `tam` es el `font-size` en px: la celda de 16 filas mide `tam` de alto (a 16 px, un cuadrito
  del dibujo = un pixel). Los glifos están dibujados en `src/pintado/fuente/glifos.txt`.
- `Lienzo` sigue trabajando con `u32` `0x00RRGGBB`; la conversión desde `css::Color` (y la
  mezcla alfa) se hace en `pintado`.

### Línea de comandos
- `farol --dom <html>` (fase 2), `farol --css <css>` (3), `farol --estilo <html> [selector]` (4),
  `farol --layout <html> [ancho]` (5), `farol --render <html> <salida.bmp> [ancho alto]` (6),
  `farol <html>` abre la página en la ventana (6), `farol --fuente` muestra la fuente (7).

## Por dentro de `layout`

- `arbol.rs`: del árbol estilado al árbol de cajas. Una caja por bloque; las corridas de
  contenido en línea entre bloques se envuelven en cajas anónimas (salvo que sean solo espacios).
- `bloque.rs`: el modelo de caja. El ancho baja del padre (CSS 2.1 §10.3.3: las siete medidas
  horizontales suman el ancho del contenedor; `auto`, centrado con márgenes `auto`, `max-width`,
  `%`); la posición apila cada hijo debajo del anterior; el alto sube de los hijos.
- `en_linea.rs`: alto provisional de las cajas anónimas (ancho del texto en una línea / ancho
  disponible, hacia arriba, × alto de línea). La fase 8 lo reemplaza por el layout en línea real.
- `tipos.rs`: los tipos del contrato. `imprimir.rs`: el texto de `farol --layout`.

## Por dentro de `pintado`

- `mod.rs`: el `Lienzo` (la lista de pixeles) y los rectángulos.
- `fuente/glifos.txt`: los dibujos de cada letra, a mano, con `#` y `.` (decisión 7.1).
- `fuente/mod.rs`: lee esos dibujos una sola vez, arma las letras con tilde, mide texto y lo
  escribe en el lienzo a cualquier tamaño.
- `fuente/demostracion.rs`: la escena de `farol --fuente`.

Este documento se actualiza cuando una decisión cambia la forma de un módulo.
