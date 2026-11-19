//! Layout: recibe el árbol de estilo y entrega cajas con posición y tamaño. Fases 5 y 8.
//!
//! Hasta aquí cada nodo sabe *cómo* quiere verse (`margin: 20px`, `width: 50%`), pero nadie sabe
//! *dónde* queda. El layout convierte eso en coordenadas de pixeles, en tres pasos:
//!
//! 1. **Armar el árbol de cajas** (`arbol.rs`): una caja por cada bloque, y cajas anónimas que
//!    envuelven el texto suelto entre bloques.
//! 2. **Medir** (`bloque.rs`): anchos de arriba hacia abajo, altos de abajo hacia arriba.
//! 3. **Partir el texto en renglones** (`en_linea/`): dentro de cada caja anónima, qué palabra
//!    va en qué renglón y en qué `(x, y)`. Lo llama `bloque.rs` al medir la caja anónima.
//!
//! - `tipos.rs`: lo que se entrega al pintado (`Caja`, `Dimensiones`, `Rect`, `Linea`, `Fragmento`).
//! - `imprimir.rs`: el árbol en texto para `farol --layout`.

mod arbol;
mod bloque;
mod en_linea;
mod imprimir;
pub mod tipos;

use crate::estilo::NodoEstilado;
// El pintado usa `Rect`, `TipoCaja` y las líneas; `Dimensiones` se nombra solo dentro de `layout`.
#[allow(unused_imports)]
pub use tipos::{Caja, CajaEnLinea, Dimensiones, Fragmento, Linea, Rect, TipoCaja};

/// El árbol de cajas de una página puesta en una ventana de `ancho_ventana` pixeles.
/// La raíz es la caja del nodo `#documento`, en (0, 0) y con todo el ancho de la ventana.
pub fn calcular(raiz: &NodoEstilado, ancho_ventana: f32) -> Caja<'_> {
    let mut caja = arbol::construir(raiz);
    bloque::colocar(&mut caja, 0.0, 0.0, ancho_ventana);
    caja
}

#[cfg(test)]
mod pruebas;
