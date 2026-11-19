//! Pruebas del árbol de cajas: HTML con `<style>` → estilo → layout → números.
//!
//! Decisión 5.2: el layout se verifica comparando coordenadas, no mirando pixeles. Donde se dice
//! "Chrome", la cuenta se hizo a mano con las reglas de CSS 2.1 y coincide con lo que muestra
//! la pestaña "Computed" / el inspector de cajas de Chrome.

use super::*;
use crate::estilo::{self, Estilo};
use crate::html::{self, Documento};
use crate::pintado::fuente;

fn estilar(pagina: &str) -> (Documento, NodoEstilado) {
    let doc = html::parsear(pagina);
    let arbol = estilo::estilar(&doc, &estilo::hojas_del_documento(&doc));
    (doc, arbol)
}

/// La primera caja de bloque (en orden del documento) con esa etiqueta, o con ese id si empieza
/// con `#`.
fn buscar<'c, 'a>(doc: &Documento, caja: &'c Caja<'a>, quien: &str) -> &'c Caja<'a> {
    fn recorrer<'c, 'a>(doc: &Documento, caja: &'c Caja<'a>, quien: &str) -> Option<&'c Caja<'a>> {
        if let TipoCaja::Bloque(id) = caja.tipo {
            let es = match (quien.strip_prefix('#'), &doc.nodo(id).tipo) {
                (Some(buscado), html::dom::TipoNodo::Elemento { atributos, .. }) => {
                    atributos.iter().any(|(c, v)| c == "id" && v == buscado)
                }
                _ => doc.nombre(id) == Some(quien),
            };
            if es {
                return Some(caja);
            }
        }
        caja.hijos.iter().find_map(|h| recorrer(doc, h, quien))
    }
    recorrer(doc, caja, quien).unwrap_or_else(|| panic!("no está {quien}"))
}

fn contenido(caja: &Caja) -> (f32, f32, f32, f32) {
    let c = caja.dimensiones.contenido;
    (c.x, c.y, c.ancho, c.alto)
}

/// Para comparar números con decimales sin pelear con el redondeo de `f32`.
fn casi(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn es_anonima(caja: &Caja) -> bool {
    matches!(caja.tipo, TipoCaja::Anonima(_))
}

// --- Anchos ---

#[test]
fn ancho_auto_llena_el_contenedor() {
    let (doc, est) = estilar("<html><body><div>x</div></body></html>");
    let raiz = calcular(&est, 800.0);
    assert_eq!(contenido(&raiz).2, 800.0);
    // body tiene margin: 8px (hoja del navegador): el div empieza en 8 y mide 800 − 16.
    let div = buscar(&doc, &raiz, "div");
    assert_eq!((contenido(div).0, contenido(div).2), (8.0, 784.0));
}

#[test]
fn hola_coincide_con_chrome() {
    // pruebas/01-hola.html a 800 px. Cuenta a mano (la misma que hace Chrome):
    //   body: margen 8              → contenido x = 8,  ancho = 800 − 2·8 = 784
    //   .caja: margen 20, borde 2, relleno 10
    //          contenido x = 8 + 20 + 2 + 10 = 40, ancho = 800 − 2·(8 + 20 + 2 + 10) = 720
    //          caja de borde x = 8 + 20 = 28, ancho = 720 + 2·(10 + 2) = 744
    //   h1: margen 0.67em × 32 = 21.44 arriba y abajo, contenido y = 8 + 21.44 = 29.44.
    //       En Chrome el y es 21.44: el margen del body (8) colapsa con el del h1 (21.44) y queda
    //       el mayor. Farol los suma (decisión 5.1, sin colapso): diferencia de 8 px conocida.
    let (doc, est) = estilar(include_str!("../../pruebas/01-hola.html"));
    let raiz = calcular(&est, 800.0);
    let caja = buscar(&doc, &raiz, "div");
    assert_eq!((contenido(caja).0, contenido(caja).2), (40.0, 720.0));
    let borde = caja.dimensiones.caja_borde();
    assert_eq!((borde.x, borde.ancho), (28.0, 744.0));
    let h1 = buscar(&doc, &raiz, "h1");
    assert_eq!((contenido(h1).0, contenido(h1).2), (8.0, 784.0));
    assert!(casi(contenido(h1).1, 29.44));
    // El p de adentro de la caja hereda el ancho del contenido de la caja.
    let p_adentro = &caja.hijos[0];
    assert_eq!((contenido(p_adentro).0, contenido(p_adentro).2), (40.0, 720.0));
}

#[test]
fn ancho_fijo_y_el_margen_derecho_absorbe_lo_que_sobra() {
    let (doc, est) = estilar(
        "<style>body{margin:0} div{width:300px; margin-left:10px; margin-right:10px}</style>\
         <body><div>x</div></body>",
    );
    let raiz = calcular(&est, 800.0);
    let div = buscar(&doc, &raiz, "div");
    assert_eq!((contenido(div).0, contenido(div).2), (10.0, 300.0));
    // Sobre-especificado: 10 + 300 + 10 ≠ 800, el margen derecho pasa a 490.
    assert_eq!(div.dimensiones.margen.derecha, 490.0);
}

#[test]
fn porcentajes_contra_el_ancho_del_contenedor() {
    let (doc, est) = estilar(
        "<style>body{margin:0} #a{width:50%; padding:10%; margin-top:5%}</style>\
         <body><div id=a>x</div></body>",
    );
    let raiz = calcular(&est, 800.0);
    let a = buscar(&doc, &raiz, "#a");
    assert_eq!(contenido(a).2, 400.0);
    // Relleno y margen verticales en % también se miden contra el ANCHO (800), no el alto.
    assert_eq!(a.dimensiones.relleno.arriba, 80.0);
    assert_eq!(a.dimensiones.relleno.izquierda, 80.0);
    assert_eq!(a.dimensiones.margen.arriba, 40.0);
    assert_eq!(contenido(a).1, 40.0 + 80.0);
}

#[test]
fn margen_auto_a_los_dos_lados_centra() {
    let (doc, est) = estilar(
        "<style>body{margin:0} div{width:200px; margin:0 auto}</style><body><div>x</div></body>",
    );
    let raiz = calcular(&est, 800.0);
    let div = buscar(&doc, &raiz, "div");
    // (800 − 200) / 2 = 300 a cada lado.
    assert_eq!(contenido(div).0, 300.0);
    assert_eq!(div.dimensiones.margen.izquierda, 300.0);
    assert_eq!(div.dimensiones.margen.derecha, 300.0);
}

#[test]
fn un_solo_margen_auto_empuja_al_otro_lado() {
    let (doc, est) = estilar(
        "<style>body{margin:0} div{width:200px; margin-left:auto}</style><body><div>x</div></body>",
    );
    let raiz = calcular(&est, 800.0);
    assert_eq!(contenido(buscar(&doc, &raiz, "div")).0, 600.0);
}

#[test]
fn max_width_limita_y_se_puede_centrar() {
    let (doc, est) = estilar(
        "<style>body{margin:0} #a{max-width:400px; margin:0 auto} #b{max-width:50%; width:700px}\
         #c{max-width:900px}</style><body><div id=a>x</div><div id=b>y</div><div id=c>z</div></body>",
    );
    let raiz = calcular(&est, 800.0);
    let a = buscar(&doc, &raiz, "#a");
    assert_eq!((contenido(a).0, contenido(a).2), (200.0, 400.0));
    assert_eq!(contenido(buscar(&doc, &raiz, "#b")).2, 400.0);
    // Un tope más grande que el espacio no cambia nada.
    assert_eq!(contenido(buscar(&doc, &raiz, "#c")).2, 800.0);
}

#[test]
fn relleno_y_borde_suman_a_la_caja_de_borde() {
    let (doc, est) = estilar(
        "<style>body{margin:0} div{width:100px; padding:10px; border:5px solid; margin:7px}</style>\
         <body><div>x</div></body>",
    );
    let raiz = calcular(&est, 800.0);
    let d = buscar(&doc, &raiz, "div").dimensiones;
    assert_eq!(d.contenido.ancho, 100.0);
    assert_eq!(d.caja_relleno().ancho, 120.0);
    assert_eq!(d.caja_borde().ancho, 130.0);
    // El margen derecho absorbe lo que sobra: la caja de margen llena el contenedor.
    assert_eq!(d.caja_margen().ancho, 800.0);
    assert_eq!((d.caja_borde().x, d.contenido.x), (7.0, 22.0));
}

#[test]
fn width_auto_con_relleno_resta_del_contenido() {
    let (doc, est) = estilar(
        "<style>body{margin:0} div{padding:0 50px; border:10px solid}</style><body><div>x</div></body>",
    );
    let raiz = calcular(&est, 800.0);
    // 800 − 2·50 − 2·10 = 680.
    assert_eq!(contenido(buscar(&doc, &raiz, "div")).2, 680.0);
}

// --- Altos y apilado ---

#[test]
fn bloques_apilados_con_margenes_sumados() {
    let (doc, est) = estilar(
        "<style>body{margin:0} div{height:50px} #a{margin:10px 0} #b{margin:20px 0}</style>\
         <body><div id=a></div><div id=b></div></body>",
    );
    let raiz = calcular(&est, 800.0);
    assert_eq!(contenido(buscar(&doc, &raiz, "#a")).1, 10.0);
    // Sin colapso (decisión 5.1): 10 + 50 + 10 + 20 = 90. Chrome daría 80 (se queda el mayor).
    assert_eq!(contenido(buscar(&doc, &raiz, "#b")).1, 90.0);
    let body = buscar(&doc, &raiz, "body");
    assert_eq!(contenido(body).3, 10.0 + 50.0 + 10.0 + 20.0 + 50.0 + 20.0);
}

#[test]
fn alto_auto_es_la_suma_de_los_hijos() {
    let (doc, est) = estilar(
        "<style>body{margin:0} #p{padding:4px} .h{height:30px; margin:5px; border:1px solid}</style>\
         <body><div id=p><div class=h></div><div class=h></div></div></body>",
    );
    let raiz = calcular(&est, 800.0);
    let p = buscar(&doc, &raiz, "#p");
    // Cada hijo ocupa 5 + 1 + 30 + 1 + 5 = 42; el relleno del padre no cuenta en su contenido.
    assert_eq!(contenido(p).3, 84.0);
    assert_eq!(p.dimensiones.caja_borde().alto, 92.0);
    // El segundo hijo empieza donde termina la caja de margen del primero.
    assert_eq!(p.hijos[1].dimensiones.caja_margen().y, 4.0 + 42.0);
}

#[test]
fn height_fijo_manda_aunque_los_hijos_midan_mas() {
    let (doc, est) = estilar(
        "<style>body{margin:0} #p{height:20px} #h{height:100px}</style>\
         <body><div id=p><div id=h></div></div><div id=sig></div></body>",
    );
    let raiz = calcular(&est, 800.0);
    assert_eq!(contenido(buscar(&doc, &raiz, "#p")).3, 20.0);
    // El siguiente hermano se apoya en el alto fijo, no en el desborde.
    assert_eq!(contenido(buscar(&doc, &raiz, "#sig")).1, 20.0);
}

#[test]
fn la_raiz_mide_toda_la_pagina() {
    let (_, est) = estilar("<style>div{height:300px}</style><body><div></div></body>");
    let raiz = calcular(&est, 640.0);
    assert_eq!(contenido(&raiz), (0.0, 0.0, 640.0, 316.0));
}

// --- Cajas anónimas ---

#[test]
fn texto_suelto_entre_bloques_va_en_cajas_anonimas() {
    let (doc, est) = estilar("<body><div>hola <b>mundo</b><p>párrafo</p> chao </div></body>");
    let raiz = calcular(&est, 800.0);
    let div = buscar(&doc, &raiz, "div");
    let tipos: Vec<bool> = div.hijos.iter().map(es_anonima).collect();
    assert_eq!(tipos, [true, false, true]);
    // La primera anónima guarda el texto y el <b>, tal cual, para la fase 8.
    let TipoCaja::Anonima(nodos) = &div.hijos[0].tipo else { unreachable!() };
    assert_eq!(nodos.len(), 2);
    // Es invisible: hereda la letra pero no el fondo, los bordes ni los márgenes.
    assert_eq!(div.hijos[0].estilo.fondo, Estilo::inicial().fondo);
    assert_eq!(div.hijos[0].dimensiones.caja_margen(), div.hijos[0].dimensiones.contenido);
}

#[test]
fn los_espacios_entre_bloques_no_generan_cajas() {
    let (doc, est) = estilar("<body>\n  <h1>T</h1>\n  <p>a</p>\n  <div>\n    <p>b</p>\n  </div>\n</body>");
    let raiz = calcular(&est, 800.0);
    let body = buscar(&doc, &raiz, "body");
    assert_eq!(body.hijos.len(), 3);
    assert!(body.hijos.iter().all(|h| !es_anonima(h)));
    assert_eq!(buscar(&doc, &raiz, "div").hijos.len(), 1);
    // Un párrafo con texto tiene exactamente una caja anónima.
    let p = buscar(&doc, &raiz, "p");
    assert!(p.hijos.len() == 1 && es_anonima(&p.hijos[0]));
}

#[test]
fn un_bloque_dentro_de_un_en_linea_lo_vuelve_bloque() {
    let (doc, est) = estilar("<body><a href=x>antes<div>adentro</div></a></body>");
    let raiz = calcular(&est, 800.0);
    let a = buscar(&doc, &raiz, "a");
    assert_eq!(contenido(a).2, 784.0);
    let tipos: Vec<bool> = a.hijos.iter().map(es_anonima).collect();
    assert_eq!(tipos, [true, false]);
}

// --- Alto provisional del texto ---

#[test]
fn una_linea_de_texto_mide_un_alto_de_linea() {
    let (doc, est) = estilar("<body><p>Hola</p></body>");
    let raiz = calcular(&est, 800.0);
    // 16 px × 1.2 (line-height: normal) = 19.2.
    assert!(casi(contenido(buscar(&doc, &raiz, "p")).3, 19.2));
}

#[test]
fn texto_largo_ocupa_varias_lineas() {
    let frase = "palabra ".repeat(50);
    let (doc, est) = estilar(&format!("<style>body{{margin:0}}</style><body><p>{frase}</p></body>"));
    let raiz = calcular(&est, 200.0);
    // 50 palabras y 49 espacios (el del final no cuenta) en una sola línea, partidas cada 200 px.
    let total = 50.0 * fuente::ancho_texto("palabra", 16.0, false) + 49.0 * fuente::ancho_texto(" ", 16.0, false);
    let lineas = (total / 200.0).ceil();
    assert!(lineas > 5.0, "la cuenta debería dar varias líneas: {lineas}");
    assert!(casi(contenido(buscar(&doc, &raiz, "p")).3, lineas * 19.2));
    // A más ancho, menos líneas.
    let ancha = calcular(&est, 2000.0);
    assert!(contenido(buscar(&doc, &ancha, "p")).3 < contenido(buscar(&doc, &raiz, "p")).3);
}

#[test]
fn los_espacios_se_colapsan_al_medir() {
    let (doc, est) = estilar("<body><p>  a \n\n   b  </p><p>a b</p></body>");
    let raiz = calcular(&est, 800.0);
    let TipoCaja::Anonima(nodos) = &buscar(&doc, &raiz, "p").hijos[0].tipo else { unreachable!() };
    let esperado = fuente::ancho_texto("a b", 16.0, false);
    assert!(casi(en_linea::ancho_en_una_linea(nodos, 800.0), esperado));
}

#[test]
fn el_relleno_de_un_en_linea_ocupa_lugar_en_la_linea() {
    let (doc, est) = estilar("<style>span{padding:0 8px; border:1px solid}</style><body><p><span>x</span></p></body>");
    let raiz = calcular(&est, 800.0);
    let TipoCaja::Anonima(nodos) = &buscar(&doc, &raiz, "p").hijos[0].tipo else { unreachable!() };
    let esperado = fuente::ancho_texto("x", 16.0, false) + 2.0 * (8.0 + 1.0);
    assert!(casi(en_linea::ancho_en_una_linea(nodos, 800.0), esperado));
}

// --- Impresión ---

#[test]
fn imprimir_muestra_posicion_tamano_y_capas() {
    let (doc, est) = estilar(include_str!("../../pruebas/01-hola.html"));
    let texto = calcular(&est, 800.0).imprimir(&doc);
    assert!(texto.starts_with("#documento bloque · (0, 0) 800 × "));
    assert!(texto.contains("<body> bloque · (8, 8) 784 × "));
    assert!(texto.contains("<div class=\"caja\"> bloque · (40, "));
    assert!(texto.contains("· margen 20 · borde 2 · relleno 10"));
    assert!(texto.contains("anónima · (8, 29.44) 784 × 38.4 · \"Hola, mundo\""));
}
