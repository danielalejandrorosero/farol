//! Estilo: recibe el DOM y las reglas, y entrega cada nodo con sus propiedades finales. Fase 4.
//!
//! Por cada elemento del DOM, de arriba hacia abajo:
//!
//! 1. **Emparejar** (`emparejar.rs`): qué reglas le aplican. Decisión 4.1: ingenuo, cada regla
//!    de cada hoja contra cada elemento.
//! 2. **Cascada** (`cascada.rs`): de lo que le aplica, qué gana en cada propiedad.
//! 3. **Computar** (`computar.rs`): convertir lo que ganó en números, heredando del padre lo
//!    que se hereda.
//!
//! Las hojas que entran son tres capas: la del navegador (`por_defecto.css`), las del autor (los
//! `<style>` de la página) y el atributo `style=""` de cada elemento.
//!
//! - `tipos.rs`: lo que se entrega al layout.
//! - `imprimir.rs`: el árbol en texto para `farol --estilo`.

mod cascada;
mod computar;
mod emparejar;
mod imprimir;
pub mod tipos;

use crate::css::{self, HojaDeEstilos, Selector};
use crate::html::dom::{Documento, NodoId, TipoNodo};
use cascada::{Candidata, Origen};
use computar::Calculado;
// `Alineacion` la usará el layout en línea (fase 8).
#[allow(unused_imports)]
pub use tipos::{Alineacion, Display, Estilo, Lados, Longitud, NodoEstilado};

/// El CSS del navegador, metido dentro del ejecutable al compilar.
const HOJA_POR_DEFECTO: &str = include_str!("por_defecto.css");

/// El contenido de cada `<style>` del documento, ya parseado, en el orden en que aparecen.
pub fn hojas_del_documento(doc: &Documento) -> Vec<HojaDeEstilos> {
    let mut hojas = Vec::new();
    juntar_estilos(doc, doc.raiz(), &mut hojas);
    hojas
}

fn juntar_estilos(doc: &Documento, id: NodoId, hojas: &mut Vec<HojaDeEstilos>) {
    if doc.nombre(id) == Some("style") {
        let mut texto = String::new();
        for &hijo in &doc.nodo(id).hijos {
            if let TipoNodo::Texto(t) = &doc.nodo(hijo).tipo {
                texto += t;
            }
        }
        hojas.push(css::parsear(&texto));
        return;
    }
    for &hijo in &doc.nodo(id).hijos {
        juntar_estilos(doc, hijo, hojas);
    }
}

/// El árbol estilado del documento. La raíz es el nodo `Documento`, con el estilo inicial y
/// `display: block`. Los nodos con `display: none` no aparecen, ni nada de lo que tengan adentro.
pub fn estilar(doc: &Documento, hojas_autor: &[HojaDeEstilos]) -> NodoEstilado {
    let navegador = css::parsear(HOJA_POR_DEFECTO);
    let hojas = con_origen(&navegador, hojas_autor);
    let raiz = Calculado::raiz();
    let hijos = estilar_hijos(doc, doc.raiz(), &raiz, &hojas);
    NodoEstilado { nodo: doc.raiz(), texto: None, estilo: raiz.estilo, hijos }
}

fn con_origen<'h>(
    navegador: &'h HojaDeEstilos,
    autor: &'h [HojaDeEstilos],
) -> Vec<(Origen, &'h HojaDeEstilos)> {
    let mut hojas = vec![(Origen::Navegador, navegador)];
    hojas.extend(autor.iter().map(|h| (Origen::Autor, h)));
    hojas
}

fn estilar_hijos(
    doc: &Documento,
    id: NodoId,
    padre: &Calculado,
    hojas: &[(Origen, &HojaDeEstilos)],
) -> Vec<NodoEstilado> {
    let mut hijos = Vec::new();
    for &hijo in &doc.nodo(id).hijos {
        match &doc.nodo(hijo).tipo {
            // Un texto no tiene reglas propias: hereda de su elemento y fluye en línea.
            TipoNodo::Texto(texto) => hijos.push(NodoEstilado {
                nodo: hijo,
                texto: Some(texto.clone()),
                estilo: Estilo::para_texto(&padre.estilo),
                hijos: Vec::new(),
            }),
            TipoNodo::Elemento { .. } => {
                let especificados = cascada::especificados(doc, hijo, hojas);
                let calculado = computar::computar(&especificados, padre);
                if calculado.estilo.display == Display::Ninguno {
                    continue;
                }
                hijos.push(NodoEstilado {
                    nodo: hijo,
                    texto: None,
                    hijos: estilar_hijos(doc, hijo, &calculado, hojas),
                    estilo: calculado.estilo,
                })
            }
            TipoNodo::Documento => {}
        }
    }
    hijos
}

/// Para depurar la cascada: de cada elemento que coincide con `selector`, todas las
/// declaraciones que le aplican agrupadas por propiedad, la ganadora primero.
/// Es lo que muestra `farol --estilo <archivo> <selector>`.
pub fn explicar(doc: &Documento, hojas_autor: &[HojaDeEstilos], selector: &Selector) -> String {
    let navegador = css::parsear(HOJA_POR_DEFECTO);
    let hojas = con_origen(&navegador, hojas_autor);
    let mut salida = String::new();
    let mut pendientes = vec![doc.raiz()];
    while let Some(id) = pendientes.pop() {
        pendientes.extend(doc.nodo(id).hijos.iter().rev());
        if doc.nombre(id).is_none() || !emparejar::coincide(doc, id, selector) {
            continue;
        }
        let especificados = cascada::especificados(doc, id, &hojas);
        let mut propiedades: Vec<(&String, &Vec<Candidata>)> =
            especificados.por_propiedad.iter().collect();
        propiedades.sort_by_key(|(nombre, _)| nombre.as_str());
        salida += &imprimir::explicacion(doc, id, &propiedades);
    }
    if salida.is_empty() {
        salida = format!("Ningún elemento coincide con `{selector}`.\n");
    }
    salida
}

#[cfg(test)]
mod pruebas;
