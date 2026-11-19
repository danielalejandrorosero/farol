//! El árbol estilado en texto, para `farol --estilo`.
//!
//! Cada elemento muestra un resumen corto: su `display` y solo lo que cambia. Las propiedades
//! que se heredan (color, letra, alineación, alto de línea) se muestran donde cambian respecto
//! al padre; las que no se heredan (márgenes, bordes, fondo, ancho), cuando no valen lo inicial.
//!
//! ```text
//! └── <h1> bloque · color #0f6e6e · 32px negrita · margen 21.44px 0
//!     └── "Hola, mundo"
//! ```

use std::fmt::Write;

use super::cascada::Candidata;
use super::emparejar::atributo;
use super::tipos::{Alineacion, Display, Estilo, Lados, Longitud, NodoEstilado};
use crate::html::dom::{Documento, NodoId};

impl NodoEstilado {
    pub fn imprimir(&self, doc: &Documento) -> String {
        let mut salida = String::from("#documento\n");
        imprimir_hijos(self, doc, "", &mut salida);
        salida
    }
}

fn imprimir_hijos(nodo: &NodoEstilado, doc: &Documento, prefijo: &str, salida: &mut String) {
    // Los textos que son solo espacios existen en el árbol, pero no se muestran (como en --dom).
    let visibles: Vec<&NodoEstilado> = nodo
        .hijos
        .iter()
        .filter(|h| h.texto.as_ref().is_none_or(|t| !t.trim().is_empty()))
        .collect();
    for (i, hijo) in visibles.iter().enumerate() {
        let ultimo = i + 1 == visibles.len();
        let rama = if ultimo { "└── " } else { "├── " };
        let rotulo = match &hijo.texto {
            Some(texto) => texto_corto(texto),
            None => format!("{} {}", etiqueta(doc, hijo.nodo), resumen(&hijo.estilo, &nodo.estilo)),
        };
        let _ = writeln!(salida, "{prefijo}{rama}{rotulo}");
        let siguiente = format!("{prefijo}{}", if ultimo { "    " } else { "│   " });
        imprimir_hijos(hijo, doc, &siguiente, salida);
    }
}

fn texto_corto(texto: &str) -> String {
    let limpio = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    let corto: String = limpio.chars().take(50).collect();
    let puntos = if limpio.chars().count() > 50 { "…" } else { "" };
    format!("\"{corto}{puntos}\"")
}

/// `<div id="x" class="caja">`: solo los atributos que usan los selectores.
fn etiqueta(doc: &Documento, id: NodoId) -> String {
    let mut r = format!("<{}", doc.nombre(id).unwrap_or("?"));
    for clave in ["id", "class"] {
        if let Some(valor) = atributo(doc, id, clave) {
            let _ = write!(r, " {clave}=\"{valor}\"");
        }
    }
    r + ">"
}

fn resumen(e: &Estilo, padre: &Estilo) -> String {
    let inicial = Estilo::inicial();
    let mut partes = vec![match e.display {
        Display::Bloque => "bloque".to_string(),
        Display::EnLinea => "en línea".to_string(),
        Display::Ninguno => "ninguno".to_string(),
    }];
    if e.color != padre.color {
        partes.push(format!("color {}", e.color));
    }
    if e.fondo != inicial.fondo {
        partes.push(format!("fondo {}", e.fondo));
    }
    let mut letra = Vec::new();
    if e.tam_fuente != padre.tam_fuente {
        letra.push(px(e.tam_fuente));
    }
    for (propio, del_padre, nombre) in [
        (e.negrita, padre.negrita, "negrita"),
        (e.cursiva, padre.cursiva, "cursiva"),
        (e.subrayado, padre.subrayado, "subrayado"),
    ] {
        if propio != del_padre {
            letra.push(if propio { nombre.to_string() } else { format!("sin {nombre}") });
        }
    }
    if !letra.is_empty() {
        partes.push(letra.join(" "));
    }
    // El alto de línea se muestra solo si no sale de multiplicar la letra por el mismo factor.
    if (e.alto_linea / e.tam_fuente - padre.alto_linea / padre.tam_fuente).abs() > 0.001 {
        partes.push(format!("línea {}", px(e.alto_linea)));
    }
    if e.alineacion != padre.alineacion {
        partes.push(
            match e.alineacion {
                Alineacion::Izquierda => "a la izquierda",
                Alineacion::Centro => "centrado",
                Alineacion::Derecha => "a la derecha",
            }
            .to_string(),
        );
    }
    for (valor, nombre) in [(e.ancho, "ancho"), (e.alto, "alto"), (e.ancho_maximo, "ancho máx.")] {
        if valor != Longitud::Auto {
            partes.push(format!("{nombre} {}", longitud(valor)));
        }
    }
    if e.margen != inicial.margen {
        partes.push(format!("margen {}", compacto(e.margen, longitud)));
    }
    if e.relleno != inicial.relleno {
        partes.push(format!("relleno {}", compacto(e.relleno, longitud)));
    }
    if e.borde_ancho != inicial.borde_ancho {
        // El color de un lado sin borde no se ve: si los lados visibles comparten color, va uno.
        let anchos = e.borde_ancho.en_orden();
        let colores = e.borde_color.en_orden();
        let visibles: Vec<_> = (0..4).filter(|&i| anchos[i] > 0.0).map(|i| colores[i]).collect();
        let colores = if visibles.iter().all(|c| *c == visibles[0]) {
            visibles[0].to_string()
        } else {
            compacto(e.borde_color, |c| c.to_string())
        };
        partes.push(format!("borde {} {colores}", compacto(e.borde_ancho, px)));
    }
    partes.join(" · ")
}

/// Un número en px sin decimales de sobra: `8px`, `21.44px`, `0`.
fn px(n: f32) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    let texto = format!("{n:.2}");
    let texto = texto.trim_end_matches('0').trim_end_matches('.');
    format!("{texto}px")
}

fn longitud(l: Longitud) -> String {
    match l {
        Longitud::Auto => "auto".to_string(),
        Longitud::Px(n) => px(n),
        Longitud::Porcentaje(n) => format!("{n}%"),
    }
}

/// Cuatro lados con la abreviatura de CSS: `8px` si son iguales, `1em 0` si se repiten de a dos…
fn compacto<T: Copy>(lados: Lados<T>, mostrar: impl Fn(T) -> String) -> String {
    let [arriba, derecha, abajo, izquierda] = lados.en_orden().map(mostrar);
    if derecha == izquierda {
        if arriba == abajo {
            if arriba == derecha {
                return arriba;
            }
            return format!("{arriba} {derecha}");
        }
        return format!("{arriba} {derecha} {abajo}");
    }
    format!("{arriba} {derecha} {abajo} {izquierda}")
}

/// Las declaraciones que pelean por cada propiedad de un elemento, la ganadora primero.
pub fn explicacion(doc: &Documento, id: NodoId, propiedades: &[(&String, &Vec<Candidata>)]) -> String {
    let mut salida = format!("{}\n", etiqueta(doc, id));
    for (propiedad, candidatas) in propiedades {
        for (i, c) in candidatas.iter().enumerate() {
            let nombre = if i == 0 { format!("{propiedad}:") } else { String::new() };
            let importante = if c.declaracion.importante { " !important" } else { "" };
            let valor = format!("{}{importante}", c.declaracion.valor);
            let veredicto = if i == 0 { "gana  " } else { "pierde" };
            let (a, b, d) = c.prioridad.2;
            let de_donde = if c.selector.is_empty() {
                c.origen().to_string()
            } else {
                format!("{} · {} ({a},{b},{d})", c.origen(), c.selector)
            };
            let _ = writeln!(salida, "  {nombre:<22} {valor:<22} {veredicto} {de_donde}");
        }
    }
    salida + "\n"
}
