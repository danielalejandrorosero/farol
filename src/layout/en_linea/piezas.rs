//! Paso 1 del layout en línea: aplanar los nodos en una fila de **piezas** y colapsar espacios.
//!
//! El HTML es un árbol (`<p>Hola <b>mundo</b></p>`), pero un renglón es una fila: lo que importa
//! es qué viene después de qué. Así que el árbol se recorre en orden y se convierte en una cinta:
//!
//! ```text
//! "Hola "  <b>  "mundo"  </b>   →   Palabra(Hola) Espacio Abre(b) Palabra(mundo) Cierra(b)
//! ```
//!
//! **Colapso de espacios** (`white-space: normal`): en el HTML, un salto de línea o diez espacios
//! seguidos valen lo mismo que un espacio. Aquí, cualquier racha de espacios, tabuladores y saltos
//! de línea da **un** `Espacio`, aunque la racha cruce de un nodo a otro (`<b>a </b> <i>b</i>`
//! tiene un solo espacio entre `a` y `b`). Los del principio no dan nada; los del final de cada
//! línea los quita el paso 2, que es el que sabe dónde termina cada línea.
//!
//! El espacio duro (`&nbsp;`, U+00A0) no colapsa ni deja partir la línea: queda *dentro* de la
//! palabra, convertido en un espacio común para que la fuente lo dibuje y lo mida igual.

use crate::estilo::{Estilo, NodoEstilado};

/// Lo que el paso 2 acomoda en líneas. `estilo` es siempre el del nodo de donde salió la pieza.
#[derive(Debug, Clone, PartialEq)]
pub enum Pieza<'a> {
    /// Letras seguidas, sin espacios que colapsen. Nunca se parte.
    Palabra { texto: String, estilo: &'a Estilo },
    /// Un espacio entre palabras: el único lugar donde se puede partir la línea.
    Espacio { estilo: &'a Estilo },
    /// Un `<br>`: aquí la línea termina sí o sí.
    Salto,
    /// Empieza un elemento en línea (`<b>`, `<span>`): aporta su margen, borde y relleno izquierdos.
    Abre(&'a Estilo),
    /// Termina ese elemento: aporta los derechos.
    Cierra(&'a Estilo),
}

/// ¿Es un espacio de los que colapsan? Ojo: `char::is_whitespace` de Rust también dice que sí al
/// espacio duro U+00A0, y ese justamente no colapsa. Por eso la lista va a mano (la de CSS).
pub fn colapsable(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{0C}')
}

/// La cinta de piezas de una corrida de nodos en línea, con los espacios ya colapsados.
pub fn aplanar<'a>(nodos: &[&'a NodoEstilado]) -> Vec<Pieza<'a>> {
    // `tras_espacio` empieza en `true`: un espacio al principio de todo no se ve.
    let mut cinta = Cinta { piezas: Vec::new(), tras_espacio: true };
    for nodo in nodos {
        cinta.nodo(nodo);
    }
    cinta.piezas
}

struct Cinta<'a> {
    piezas: Vec<Pieza<'a>>,
    /// Lo último que importa fue un espacio (o el principio, o un `<br>`): otro espacio no suma.
    /// Cruza aperturas y cierres de elementos: eso es lo que colapsa `a </b> <i>b`.
    tras_espacio: bool,
}

impl<'a> Cinta<'a> {
    fn nodo(&mut self, nodo: &'a NodoEstilado) {
        if nodo.salto_de_linea {
            self.piezas.push(Pieza::Salto);
            self.tras_espacio = true; // la línea siguiente no empieza con espacio
            return;
        }
        match &nodo.texto {
            Some(texto) => self.texto(texto, &nodo.estilo),
            None => {
                self.piezas.push(Pieza::Abre(&nodo.estilo));
                for hijo in &nodo.hijos {
                    self.nodo(hijo);
                }
                self.piezas.push(Pieza::Cierra(&nodo.estilo));
            }
        }
    }

    fn texto(&mut self, texto: &str, estilo: &'a Estilo) {
        let mut palabra = String::new();
        for c in texto.chars() {
            if colapsable(c) {
                self.cerrar_palabra(&mut palabra, estilo);
                if !self.tras_espacio {
                    self.piezas.push(Pieza::Espacio { estilo });
                    self.tras_espacio = true;
                }
            } else {
                palabra.push(if c == '\u{A0}' { ' ' } else { c });
            }
        }
        self.cerrar_palabra(&mut palabra, estilo);
    }

    fn cerrar_palabra(&mut self, palabra: &mut String, estilo: &'a Estilo) {
        if !palabra.is_empty() {
            self.piezas.push(Pieza::Palabra { texto: std::mem::take(palabra), estilo });
            self.tras_espacio = false;
        }
    }
}
