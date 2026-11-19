//! Las órdenes de dibujo de los renglones de una caja anónima (fase 8).
//!
//! Por cada caja anónima, en este orden (cada capa queda encima de la anterior):
//!
//! 1. El fondo y el borde de cada elemento en línea (`<span style="background:…">`, `.boton`),
//!    pedazo por pedazo, renglón por renglón. Van primero para que las letras queden encima.
//! 2. Cada fragmento de texto como una `Orden::Texto`.
//! 3. Si el fragmento está subrayado, una raya debajo de la línea base, del ancho del fragmento.

use super::fuente;
use super::lista::{Orden, agregar, pintar_anillo};
use crate::layout::{Fragmento, Linea, Rect};

/// Agrega a `ordenes` todo lo que se dibuja en estos renglones.
pub fn pintar_lineas(lineas: &[Linea], ordenes: &mut Vec<Orden>) {
    for caja in lineas.iter().flat_map(|l| &l.cajas) {
        let b = &caja.borde;
        // La caja de relleno es la de borde sin el grosor de cada lado.
        let relleno = Rect {
            x: caja.rect.x + b.izquierda,
            y: caja.rect.y + b.arriba,
            ancho: caja.rect.ancho - b.izquierda - b.derecha,
            alto: caja.rect.alto - b.arriba - b.abajo,
        };
        agregar(ordenes, relleno, caja.estilo.fondo);
        pintar_anillo(caja.rect, relleno, b, &caja.estilo.borde_color, ordenes);
    }
    for fragmento in lineas.iter().flat_map(|l| &l.fragmentos) {
        pintar_fragmento(fragmento, ordenes);
    }
}

fn pintar_fragmento(f: &Fragmento, ordenes: &mut Vec<Orden>) {
    let e = f.estilo;
    ordenes.push(Orden::Texto {
        x: f.rect.x,
        y: f.rect.y,
        texto: f.texto.clone(),
        tam: e.tam_fuente,
        color: e.color,
        negrita: e.negrita,
        cursiva: e.cursiva,
    });
    if e.subrayado {
        agregar(ordenes, subrayado(f), e.color);
    }
}

/// La raya del subrayado: un cuadrito de la fuente debajo de la línea base (cruza la cola de la
/// g y la p, como en los navegadores) y de un cuadrito de grueso, nunca menos de 1 pixel.
pub fn subrayado(f: &Fragmento) -> Rect {
    let tam = f.estilo.tam_fuente;
    let cuadrito = tam / 16.0;
    let base = f.rect.y + fuente::linea_base(tam);
    Rect { x: f.rect.x, y: base + cuadrito, ancho: f.rect.ancho, alto: cuadrito.max(1.0) }
}
