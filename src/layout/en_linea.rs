//! Alto PROVISIONAL de una caja anónima. La fase 8 reemplaza todo este archivo.
//!
//! Para saber cuánto mide de alto un párrafo hay que partirlo en líneas, y para eso hay que
//! medir palabra por palabra con la fuente: eso es el layout en línea de la fase 8. Mientras
//! tanto, una estimación honesta y simple:
//!
//! ```text
//! líneas = ⌈ ancho de todo el texto en una sola línea / ancho disponible ⌉
//! alto   = líneas × alto de línea del bloque
//! ```
//!
//! Se queda corta cuando una palabra larga no cabe al final de una línea (el corte real deja ese
//! hueco vacío y gasta una línea más), y no mira tamaños de letra mezclados dentro de una línea.

use crate::estilo::{Lados, Longitud, NodoEstilado};
use crate::pintado::fuente;

/// El alto estimado de una corrida de contenido en línea puesta en `ancho` pixeles.
pub fn alto_estimado(nodos: &[&NodoEstilado], ancho: f32, alto_linea: f32) -> f32 {
    let total = ancho_en_una_linea(nodos, ancho);
    let lineas = if total <= 0.0 {
        0.0
    } else if ancho <= 0.0 {
        1.0
    } else {
        (total / ancho).ceil()
    };
    lineas * alto_linea
}

/// Cuánto mediría la corrida escrita toda en una sola línea, con los espacios colapsados como
/// en HTML (varios espacios o saltos de línea seguidos valen un espacio; al principio y al final,
/// nada).
pub fn ancho_en_una_linea(nodos: &[&NodoEstilado], contenedor: f32) -> f32 {
    let mut medida = Medida { total: 0.0, hay_texto: false, espacio_pendiente: false };
    for nodo in nodos {
        medida.sumar(nodo, contenedor);
    }
    medida.total
}

struct Medida {
    total: f32,
    /// Ya se escribió alguna palabra (un espacio al principio no cuenta).
    hay_texto: bool,
    /// Hubo espacio después de la última palabra; se cobra solo si viene otra.
    espacio_pendiente: bool,
}

impl Medida {
    fn sumar(&mut self, nodo: &NodoEstilado, contenedor: f32) {
        let e = &nodo.estilo;
        let Some(texto) = &nodo.texto else {
            // Un elemento en línea (`<b>`, `<span class="boton">`): su relleno, borde y margen
            // a los lados ocupan lugar en la línea. Los de arriba y abajo no la agrandan.
            let lados = |l: &Lados<Longitud>| {
                [l.izquierda, l.derecha].map(|x| match x {
                    Longitud::Px(px) => px,
                    Longitud::Porcentaje(p) => p / 100.0 * contenedor,
                    Longitud::Auto => 0.0,
                })
            };
            let extra: f32 = lados(&e.relleno).iter().chain(&lados(&e.margen)).sum::<f32>()
                + e.borde_ancho.izquierda
                + e.borde_ancho.derecha;
            self.total += extra;
            for hijo in &nodo.hijos {
                self.sumar(hijo, contenedor);
            }
            return;
        };
        if texto.starts_with(char::is_whitespace) {
            self.espacio_pendiente = true;
        }
        for (i, palabra) in texto.split_whitespace().enumerate() {
            if self.hay_texto && (i > 0 || self.espacio_pendiente) {
                self.total += fuente::ancho_texto(" ", e.tam_fuente, e.negrita);
            }
            self.total += fuente::ancho_texto(palabra, e.tam_fuente, e.negrita);
            self.hay_texto = true;
            self.espacio_pendiente = false;
        }
        if texto.ends_with(char::is_whitespace) {
            self.espacio_pendiente = true;
        }
    }
}
