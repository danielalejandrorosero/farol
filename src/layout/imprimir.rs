//! El árbol de cajas en texto, para `farol --layout`.
//!
//! Cada caja muestra su tipo, la esquina de su **contenido** `(x, y)` y su tamaño `ancho × alto`;
//! después, solo las capas que no son cero, con la notación corta de CSS (`margen 16 0` =
//! 16 arriba y abajo, 0 a los lados).
//!
//! ```text
//! └── <div class="caja"> bloque · (40, 162.4) 720 × 38.4 · margen 20 · borde 2 · relleno 10
//!     └── anónima · (40, 162.4) 720 × 19.2 · "Una caja con negrita, cursiva y…"
//! ```

use std::fmt::Write;

use super::tipos::{Caja, TipoCaja};
use crate::estilo::{Lados, NodoEstilado};
use crate::html::dom::{Documento, NodoId, TipoNodo};

impl Caja<'_> {
    pub fn imprimir(&self, doc: &Documento) -> String {
        let mut salida = format!("{}\n", rotulo(self, doc));
        imprimir_hijos(self, doc, "", &mut salida);
        salida
    }
}

fn imprimir_hijos(caja: &Caja, doc: &Documento, prefijo: &str, salida: &mut String) {
    for (i, hijo) in caja.hijos.iter().enumerate() {
        let ultimo = i + 1 == caja.hijos.len();
        let rama = if ultimo { "└── " } else { "├── " };
        let _ = writeln!(salida, "{prefijo}{rama}{}", rotulo(hijo, doc));
        let siguiente = format!("{prefijo}{}", if ultimo { "    " } else { "│   " });
        imprimir_hijos(hijo, doc, &siguiente, salida);
    }
}

fn rotulo(caja: &Caja, doc: &Documento) -> String {
    let d = &caja.dimensiones;
    let c = d.contenido;
    let nombre = match &caja.tipo {
        TipoCaja::Bloque(id) => format!("{} bloque", etiqueta(doc, *id)),
        TipoCaja::Anonima(_) => "anónima".to_string(),
    };
    let mut r = format!("{nombre} · ({}, {}) {} × {}", num(c.x), num(c.y), num(c.ancho), num(c.alto));
    for (capa, lados) in [("margen", &d.margen), ("borde", &d.borde), ("relleno", &d.relleno)] {
        if lados.en_orden() != [0.0; 4] {
            let _ = write!(r, " · {capa} {}", corto(lados));
        }
    }
    if let TipoCaja::Anonima(nodos) = &caja.tipo {
        let _ = write!(r, " · {}", texto_corto(nodos));
    }
    r
}

/// `<div id="x" class="caja">` o `#documento`: solo los atributos que ayudan a ubicarse.
fn etiqueta(doc: &Documento, id: NodoId) -> String {
    match &doc.nodo(id).tipo {
        TipoNodo::Elemento { nombre, atributos } => {
            let mut r = format!("<{nombre}");
            for (clave, valor) in atributos.iter().filter(|(c, _)| c == "id" || c == "class") {
                let _ = write!(r, " {clave}=\"{valor}\"");
            }
            r + ">"
        }
        _ => "#documento".to_string(),
    }
}

/// El texto de una corrida en línea, colapsado y recortado.
fn texto_corto(nodos: &[&NodoEstilado]) -> String {
    fn juntar(nodo: &NodoEstilado, texto: &mut String) {
        match &nodo.texto {
            Some(t) => texto.push_str(t),
            None => nodo.hijos.iter().for_each(|h| juntar(h, texto)),
        }
    }
    let mut texto = String::new();
    nodos.iter().for_each(|n| juntar(n, &mut texto));
    let limpio = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    let corto: String = limpio.chars().take(40).collect();
    let puntos = if limpio.chars().count() > 40 { "…" } else { "" };
    format!("\"{corto}{puntos}\"")
}

/// Como las abreviadas de CSS: `8` si los cuatro son iguales, `16 0` si arriba = abajo e
/// izquierda = derecha, y si no, los cuatro.
fn corto(l: &Lados<f32>) -> String {
    let [a, d, b, i] = l.en_orden().map(num);
    if a == d && d == b && b == i {
        a
    } else if a == b && d == i {
        format!("{a} {d}")
    } else {
        format!("{a} {d} {b} {i}")
    }
}

/// Un número con hasta dos decimales y sin ceros de más: `29.44`, `8`, `-0.5`.
fn num(n: f32) -> String {
    let texto = format!("{n:.2}");
    let texto = texto.trim_end_matches('0').trim_end_matches('.');
    if texto == "-0" { "0".to_string() } else { texto.to_string() }
}
