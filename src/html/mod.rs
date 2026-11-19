//! HTML: recibe texto HTML y entrega el árbol DOM. Fase 2.
//!
//! Dos pasos: el tokenizador corta el texto en piezas y el constructor arma el árbol con ellas.
//! El parser es tolerante (decisión 2.1): nunca falla, cierra solo lo que la gente olvida cerrar.

mod constructor;
pub mod dom;
mod tokenizador;

pub use dom::Documento;

/// Convierte el texto de una página en su árbol DOM.
pub fn parsear(html: &str) -> Documento {
    constructor::construir(tokenizador::tokenizar(html))
}
