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
    /// `<a>`…) que quedó entre bloques. Guarda esos nodos tal cual: la fase 8 los parte en líneas.
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
}
