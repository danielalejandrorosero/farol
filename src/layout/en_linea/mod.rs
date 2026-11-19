//! Layout en línea: el texto de una caja anónima, partido en renglones. Fase 8.
//!
//! Una caja anónima tiene una corrida de nodos en línea (`"Hola "`, `<b>mundo</b>`, `<br>`…) y
//! un ancho. Hay que decidir qué palabra va en qué renglón y dónde queda cada una. Dos pasos:
//!
//! 1. `piezas.rs`: el árbol de nodos se aplana en una cinta de palabras, espacios, saltos y
//!    aperturas/cierres de elementos, con los espacios ya colapsados.
//! 2. `lineas.rs`: la cinta se reparte en renglones, palabra por palabra, con partición voraz
//!    (decisión 8.1); cada renglón calcula su alto, su línea base y su alineación.
//!
//! El resultado son `Linea`s con `Fragmento`s en coordenadas de página, listos para pintar. Todo
//! se rehace cada vez que cambia el ancho: al achicar la ventana, las mismas palabras se
//! reparten en más renglones.

mod lineas;
mod piezas;

use crate::estilo::{Estilo, NodoEstilado};
use crate::layout::tipos::Linea;

/// Los renglones de una corrida de nodos en línea, puestos desde `(x, y)` en `ancho` pixeles.
/// `estilo` es el de la caja anónima (lo heredable del bloque: letra, `line-height`,
/// `text-align`). El alto de la caja es la suma de los altos de los renglones.
pub fn colocar<'a>(
    nodos: &[&'a NodoEstilado],
    estilo: &Estilo,
    x: f32,
    y: f32,
    ancho: f32,
) -> Vec<Linea<'a>> {
    let bloque = lineas::Bloque {
        x,
        y,
        ancho,
        tam_fuente: estilo.tam_fuente,
        alto_linea: estilo.alto_linea,
        alineacion: estilo.alineacion,
    };
    lineas::partir(&piezas::aplanar(nodos), &bloque)
}

#[cfg(test)]
mod pruebas;
