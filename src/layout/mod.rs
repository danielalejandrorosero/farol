//! Layout: recibe el árbol de estilo y entrega cajas con posición y tamaño. Fases 5 y 8.
//!
//! Hasta aquí cada nodo sabe *cómo* quiere verse (`margin: 20px`, `width: 50%`), pero nadie sabe
//! *dónde* queda. El layout convierte eso en coordenadas de pixeles, en dos pasos:
//!
//! 1. **Armar el árbol de cajas** (`arbol.rs`): una caja por cada bloque, y cajas anónimas que
//!    envuelven el texto suelto entre bloques.
//! 2. **Medir** (`bloque.rs`): anchos de arriba hacia abajo, altos de abajo hacia arriba.
//!
//! - `tipos.rs`: lo que se entrega al pintado (`Caja`, `Dimensiones`, `Rect`).
//! - `en_linea.rs`: alto provisional del texto; la fase 8 pone el layout en línea de verdad.
//! - `imprimir.rs`: el árbol en texto para `farol --layout`.

mod arbol;
mod bloque;
mod en_linea;
mod imprimir;
pub mod tipos;

use crate::estilo::NodoEstilado;
// El pintado usa `Rect` y `TipoCaja`; `Dimensiones` se nombra solo dentro de `layout` por ahora.
#[allow(unused_imports)]
pub use tipos::{Caja, Dimensiones, Rect, TipoCaja};

/// El árbol de cajas de una página puesta en una ventana de `ancho_ventana` pixeles.
/// La raíz es la caja del nodo `#documento`, en (0, 0) y con todo el ancho de la ventana.
pub fn calcular(raiz: &NodoEstilado, ancho_ventana: f32) -> Caja<'_> {
    let mut caja = arbol::construir(raiz);
    bloque::colocar(&mut caja, 0.0, 0.0, ancho_ventana);
    caja
}

#[cfg(test)]
mod pruebas;
