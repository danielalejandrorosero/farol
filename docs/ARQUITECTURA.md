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
```
- Las abreviadas se expanden al parsear: `margin`, `padding` → `-top/-right/-bottom/-left`;
  `border`, `border-width/-color/-style`, `border-top`… → `border-{lado}-{width,color,style}`;
  `background: <color>` → `background-color`.
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
- Hoja por defecto del navegador en `src/estilo/por_defecto.css`, cargada con `include_str!`.
- `em` se resuelve a px aquí. Los `%` se quedan como `Porcentaje` para que los resuelva el layout.
- Un borde sin `border-style` (o `none`) tiene ancho 0, como en CSS.
- Los nodos con `display: none` y sus hijos no aparecen en el árbol.

### `layout` (fases 5 y 8)
```rust
pub struct Rect { pub x: f32, pub y: f32, pub ancho: f32, pub alto: f32 }
pub struct Dimensiones { pub contenido: Rect, pub relleno: Lados<f32>, pub borde: Lados<f32>, pub margen: Lados<f32> }
pub struct Caja<'a> { pub tipo: TipoCaja, pub estilo: &'a Estilo, pub dimensiones: Dimensiones, pub hijos: Vec<Caja<'a>> }
pub fn calcular<'a>(raiz: &'a NodoEstilado, ancho_ventana: f32) -> Caja<'a>
```
- Fase 5: el contenido en línea se envuelve en cajas anónimas con un alto provisional.
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
- `farol --dom <html>` (fase 2), `farol --css <css>` (3), `farol --estilo <html>` (4),
  `farol --layout <html> [ancho]` (5), `farol --render <html> <salida.bmp> [ancho alto]` (6),
  `farol <html>` abre la página en la ventana (6), `farol --fuente` muestra la fuente (7).

## Por dentro de `pintado`

- `mod.rs`: el `Lienzo` (la lista de pixeles) y los rectángulos.
- `fuente/glifos.txt`: los dibujos de cada letra, a mano, con `#` y `.` (decisión 7.1).
- `fuente/mod.rs`: lee esos dibujos una sola vez, arma las letras con tilde, mide texto y lo
  escribe en el lienzo a cualquier tamaño.
- `fuente/demostracion.rs`: la escena de `farol --fuente`.

Este documento se actualiza cuando una decisión cambia la forma de un módulo.
