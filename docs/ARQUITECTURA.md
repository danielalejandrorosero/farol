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
pub fn alto_linea_natural(tam: f32) -> f32
impl Lienzo { pub fn texto(&mut self, x: f32, y: f32, texto: &str, tam: f32, color: u32, negrita: bool, cursiva: bool) }
```
- `Lienzo` sigue trabajando con `u32` `0x00RRGGBB`; la conversión desde `css::Color` (y la
  mezcla alfa) se hace en `pintado`.

### Línea de comandos
- `farol --dom <html>` (fase 2), `farol --css <css>` (3), `farol --estilo <html>` (4),
  `farol --layout <html> [ancho]` (5), `farol --render <html> <salida.bmp> [ancho alto]` (6),
  `farol <html>` abre la página en la ventana (6).

Este documento se actualiza cuando una decisión cambia la forma de un módulo.
