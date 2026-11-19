//! Los tipos que entrega el layout: lo que el pintado (fase 6) recibe.
//!
//! El modelo de caja de CSS son cuatro rectángulos uno dentro de otro, como cajas de cartón:
//!
//! ```text
//! ┌─ margen ──────────────────────────┐   caja_margen(): el espacio que la caja reserva
//! │ ┌─ borde ───────────────────────┐ │   caja_borde():  hasta donde llega la línea del borde
//! │ │ ┌─ relleno ─────────────────┐ │ │   caja_relleno(): hasta donde se pinta el fondo
//! │ │ │ ┌─ contenido ───────────┐ │ │ │   contenido: donde van los hijos y el texto
//! │ │ │ └───────────────────────┘ │ │ │
//! │ │ └───────────────────────────┘ │ │
//! │ └───────────────────────────────┘ │
//! └───────────────────────────────────┘
//! ```
//!
//! Solo se guarda el rectángulo de contenido y el grosor de cada capa; los otros tres se
//! calculan sumando hacia afuera.

use std::borrow::Cow;

use crate::estilo::{Estilo, Lados, NodoEstilado};
use crate::html::dom::NodoId;

/// Un rectángulo en pixeles. `x` e `y` son la esquina de arriba a la izquierda, medidas desde
/// la esquina de arriba a la izquierda de la página (y crece hacia abajo, como en pantalla).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub ancho: f32,
    pub alto: f32,
}

impl Rect {
    /// El mismo rectángulo, agrandado por fuera con el grosor de cada lado.
    pub fn expandido(&self, lados: &Lados<f32>) -> Rect {
        Rect {
            x: self.x - lados.izquierda,
            y: self.y - lados.arriba,
            ancho: self.ancho + lados.izquierda + lados.derecha,
            alto: self.alto + lados.arriba + lados.abajo,
        }
    }
}

/// Dónde quedó una caja y qué grosor tiene cada capa, todo en px ya resueltos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dimensiones {
    pub contenido: Rect,
    pub relleno: Lados<f32>,
    pub borde: Lados<f32>,
    pub margen: Lados<f32>,
}

impl Default for Dimensiones {
    fn default() -> Self {
        Dimensiones {
            contenido: Rect::default(),
            relleno: Lados::todos(0.0),
            borde: Lados::todos(0.0),
            margen: Lados::todos(0.0),
        }
    }
}

impl Dimensiones {
    /// Contenido + relleno: el área que cubre el color de fondo.
    pub fn caja_relleno(&self) -> Rect {
        self.contenido.expandido(&self.relleno)
    }

    /// Contenido + relleno + borde: lo que se ve de la caja.
    pub fn caja_borde(&self) -> Rect {
        self.caja_relleno().expandido(&self.borde)
    }

    /// Todo, márgenes incluidos: el espacio que la caja le quita a sus vecinas.
    pub fn caja_margen(&self) -> Rect {
        self.caja_borde().expandido(&self.margen)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoCaja<'a> {
    /// La caja de un nodo con `display: block` (o la raíz `#documento`).
    Bloque(NodoId),
    /// Un bloque sin etiqueta que envuelve una corrida de contenido en línea (textos, `<b>`,
    /// `<a>`…) que quedó entre bloques. Guarda esos nodos tal cual; el layout en línea los parte
    /// en líneas y deja el resultado en `Caja::lineas`.
    Anonima(Vec<&'a NodoEstilado>),
}

/// Una caja del árbol de layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Caja<'a> {
    pub tipo: TipoCaja<'a>,
    /// El estilo del nodo, prestado del árbol estilado. Las cajas anónimas no tienen nodo: llevan
    /// un estilo propio (`Cow::Owned`) con lo heredable del padre y lo demás inicial, para que
    /// el pintado no repita el fondo ni los bordes del padre.
    pub estilo: Cow<'a, Estilo>,
    pub dimensiones: Dimensiones,
    pub hijos: Vec<Caja<'a>>,
    /// Las líneas de texto de una caja anónima, de arriba hacia abajo. Vacío en los bloques.
    pub lineas: Vec<Linea<'a>>,
}

/// Una caja de línea: un renglón de texto dentro de una caja anónima.
#[derive(Debug, Clone, PartialEq)]
pub struct Linea<'a> {
    /// El renglón entero: empieza en el borde izquierdo del contenido, mide todo el ancho
    /// disponible y su alto es el que pide lo más alto que lleva.
    pub rect: Rect,
    /// La `y` de la línea base (en coordenadas de página): ahí se apoyan todas las letras de la
    /// línea, sean del tamaño que sean.
    pub base: f32,
    pub fragmentos: Vec<Fragmento<'a>>,
    /// Fondos y bordes de los elementos en línea (`span`, `a`) en este renglón, de afuera hacia
    /// adentro: se pintan antes que el texto y en este orden.
    pub cajas: Vec<CajaEnLinea<'a>>,
}

/// Un pedazo de texto con un solo estilo, ya ubicado.
#[derive(Debug, Clone, PartialEq)]
pub struct Fragmento<'a> {
    /// `x` donde empieza la primera letra, `ancho` lo que avanza la pluma; `y` y `alto` son la
    /// caja de línea *natural* de su letra (`fuente::alto_linea_natural`): exactamente lo que
    /// espera `Lienzo::texto`.
    pub rect: Rect,
    /// Con los espacios ya colapsados (y el espacio duro U+00A0 convertido en espacio común).
    pub texto: String,
    /// El estilo del nodo de texto, prestado del árbol estilado: letra, color, subrayado.
    pub estilo: &'a Estilo,
}

/// El pedazo de un elemento en línea que cae en una línea. Si el elemento se parte entre dos
/// líneas, cada línea tiene su pedazo; el borde y el relleno izquierdos van solo en el primero y
/// los derechos solo en el último (como `box-decoration-break: slice`, lo normal en CSS).
#[derive(Debug, Clone, PartialEq)]
pub struct CajaEnLinea<'a> {
    /// La caja de borde del pedazo.
    pub rect: Rect,
    /// El grosor del borde de cada lado en este pedazo (0 en los lados cortados).
    pub borde: Lados<f32>,
    pub estilo: &'a Estilo,
}
