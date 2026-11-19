//! Del árbol estilado al árbol de cajas (todavía sin medidas).
//!
//! Un bloque solo puede tener dos tipos de hijos a la vez en el layout: o todos bloques, o todo
//! contenido en línea. Cuando se mezclan (`<div>texto <p>párrafo</p> más texto</div>`), cada
//! corrida seguida de cosas en línea se mete en una **caja anónima**: un bloque sin etiqueta.
//!
//! ```text
//! <div>                         div
//!   hola <b>mundo</b>     →     ├── anónima [ "hola ", <b> ]
//!   <p>párrafo</p>              ├── p
//!   chao                        └── anónima [ "chao" ]
//! </div>
//! ```
//!
//! Una corrida que es solo espacios (el salto de línea entre `</h1>` y `<p>`) no genera caja:
//! esos espacios no se ven en ningún navegador.

use std::borrow::Cow;

use super::tipos::{Caja, Dimensiones, TipoCaja};
use crate::estilo::{Display, Estilo, NodoEstilado};

/// La caja de un nodo de bloque, con sus hijos ya agrupados.
pub fn construir(nodo: &NodoEstilado) -> Caja<'_> {
    let mut hijos = Vec::new();
    let mut corrida: Vec<&NodoEstilado> = Vec::new();
    for hijo in &nodo.hijos {
        if es_bloque(hijo) {
            cerrar_corrida(&mut corrida, &nodo.estilo, &mut hijos);
            hijos.push(construir(hijo));
        } else {
            corrida.push(hijo);
        }
    }
    cerrar_corrida(&mut corrida, &nodo.estilo, &mut hijos);
    Caja {
        tipo: TipoCaja::Bloque(nodo.nodo),
        estilo: Cow::Borrowed(&nodo.estilo),
        dimensiones: Dimensiones::default(),
        hijos,
        lineas: Vec::new(),
    }
}

/// ¿El nodo se comporta como bloque? Además de `display: block`, un elemento en línea que
/// tiene un bloque adentro (`<a><div>…</div></a>`) se trata como bloque. CSS haría algo más
/// fino (partir el `<a>` en dos alrededor del `div`); es deuda anotada.
fn es_bloque(nodo: &NodoEstilado) -> bool {
    nodo.estilo.display == Display::Bloque
        || (nodo.texto.is_none() && nodo.hijos.iter().any(es_bloque))
}

fn solo_espacios(nodo: &NodoEstilado) -> bool {
    nodo.texto.as_ref().is_some_and(|t| t.trim().is_empty())
}

/// Envuelve la corrida en una caja anónima (salvo que sea solo espacios) y la vacía.
fn cerrar_corrida<'a>(corrida: &mut Vec<&'a NodoEstilado>, padre: &Estilo, hijos: &mut Vec<Caja<'a>>) {
    if corrida.iter().all(|n| solo_espacios(n)) {
        corrida.clear();
        return;
    }
    // Lo heredable (letra, alto de línea, alineación) viene del padre; fondo, bordes y márgenes
    // son los iniciales: la caja anónima es invisible.
    let estilo = Estilo { display: Display::Bloque, ..Estilo::para_texto(padre) };
    hijos.push(Caja {
        tipo: TipoCaja::Anonima(std::mem::take(corrida)),
        estilo: Cow::Owned(estilo),
        dimensiones: Dimensiones::default(),
        hijos: Vec::new(),
        lineas: Vec::new(),
    });
}
