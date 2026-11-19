//! Pruebas de punta a punta del motor de estilos: HTML con `<style>` → árbol estilado.

use super::*;
use crate::css::Color;
use crate::html;
use tipos::{NEGRO, TRANSPARENTE};

/// Estila una página con sus propios `<style>`.
fn estilar_pagina(pagina: &str) -> (Documento, NodoEstilado) {
    let doc = html::parsear(pagina);
    let hojas = hojas_del_documento(&doc);
    let arbol = estilar(&doc, &hojas);
    (doc, arbol)
}

/// El primer nodo estilado (en orden del documento) que cumple la condición.
fn buscar_por<'a>(
    doc: &Documento,
    nodo: &'a NodoEstilado,
    cumple: &dyn Fn(&Documento, NodoId) -> bool,
) -> Option<&'a NodoEstilado> {
    if nodo.texto.is_none() && cumple(doc, nodo.nodo) {
        return Some(nodo);
    }
    nodo.hijos.iter().find_map(|h| buscar_por(doc, h, cumple))
}

/// El estilo del primer elemento con esa etiqueta, o con ese id si empieza con `#`.
fn estilo<'a>(doc: &Documento, arbol: &'a NodoEstilado, quien: &str) -> &'a Estilo {
    let encontrado = match quien.strip_prefix('#') {
        Some(id) => buscar_por(doc, arbol, &|d, n| emparejar::atributo(d, n, "id") == Some(id)),
        None => buscar_por(doc, arbol, &|d, n| d.nombre(n) == Some(quien)),
    };
    &encontrado.unwrap_or_else(|| panic!("no está {quien}")).estilo
}

fn existe(doc: &Documento, arbol: &NodoEstilado, etiqueta: &str) -> bool {
    buscar_por(doc, arbol, &|d, n| d.nombre(n) == Some(etiqueta)).is_some()
}

const ROJO: Color = Color::rgb(255, 0, 0);
const AZUL: Color = Color::rgb(0, 0, 255);
const VERDE: Color = Color::rgb(0, 128, 0);

// --- Hoja por defecto ---

#[test]
fn la_hoja_por_defecto_se_parsea_sin_avisos() {
    let (hoja, avisos) = css::parsear_con_avisos(HOJA_POR_DEFECTO);
    assert!(avisos.is_empty(), "{avisos:?}");
    assert!(hoja.reglas.len() > 20);
}

#[test]
fn por_defecto_h1_grande_y_negrita() {
    let (doc, arbol) = estilar_pagina("<html><body><h1>T</h1><h2>S</h2></body></html>");
    let h1 = estilo(&doc, &arbol, "h1");
    assert_eq!(h1.display, Display::Bloque);
    assert_eq!(h1.tam_fuente, 32.0);
    assert!(h1.negrita);
    // 0.67em con la letra del propio h1: 0.67 × 32.
    assert_eq!(h1.margen.arriba, Longitud::Px(0.67 * 32.0));
    assert_eq!(h1.margen.izquierda, Longitud::Px(0.0));
    assert_eq!(estilo(&doc, &arbol, "h2").tam_fuente, 24.0);
}

#[test]
fn por_defecto_enlace_azul_y_subrayado() {
    let (doc, arbol) = estilar_pagina(r#"<p>ver <a href="x">aquí</a></p>"#);
    let a = estilo(&doc, &arbol, "a");
    assert_eq!(a.color, Color::rgb(0, 0, 0xee));
    assert!(a.subrayado);
    assert_eq!(a.display, Display::EnLinea);
}

#[test]
fn por_defecto_body_listas_y_cita() {
    let (doc, arbol) = estilar_pagina(
        "<html><body><ul><li>a<ul><li>b</li></ul></li></ul><blockquote>c</blockquote><span>d</span></body></html>",
    );
    assert_eq!(estilo(&doc, &arbol, "body").margen, Lados::todos(Longitud::Px(8.0)));
    let ul = estilo(&doc, &arbol, "ul");
    assert_eq!(ul.margen.arriba, Longitud::Px(16.0));
    assert_eq!(ul.relleno.izquierda, Longitud::Px(40.0));
    assert_eq!(estilo(&doc, &arbol, "li").display, Display::Bloque);
    let bq = estilo(&doc, &arbol, "blockquote");
    assert_eq!(bq.margen.izquierda, Longitud::Px(40.0));
    // Lo que no está en la hoja del navegador es en línea.
    assert_eq!(estilo(&doc, &arbol, "span").display, Display::EnLinea);
}

#[test]
fn lista_anidada_sin_margen_vertical() {
    let (doc, arbol) = estilar_pagina(r#"<ul><li><ul id="dentro"><li>x</li></ul></li></ul>"#);
    let dentro = estilo(&doc, &arbol, "#dentro");
    assert_eq!(dentro.margen.arriba, Longitud::Px(0.0));
    assert_eq!(dentro.relleno.izquierda, Longitud::Px(40.0));
}

#[test]
fn por_defecto_head_oculto() {
    let (doc, arbol) =
        estilar_pagina("<html><head><title>T</title><style>p{}</style></head><body><p>x</p></body></html>");
    assert!(!existe(&doc, &arbol, "head"));
    assert!(!existe(&doc, &arbol, "title"));
    assert!(!existe(&doc, &arbol, "style"));
    assert!(existe(&doc, &arbol, "p"));
}

#[test]
fn hr_tiene_borde_gris_de_un_pixel() {
    let (doc, arbol) = estilar_pagina("<hr>");
    let hr = estilo(&doc, &arbol, "hr");
    assert_eq!(hr.borde_ancho, Lados::todos(1.0));
    assert_eq!(hr.borde_color, Lados::todos(Color::rgb(128, 128, 128)));
    assert_eq!(hr.margen.izquierda, Longitud::Auto);
}

#[test]
fn el_autor_le_gana_a_la_hoja_por_defecto() {
    let (doc, arbol) = estilar_pagina("<style>* { margin: 0 } a { color: red }</style><h1>T</h1><a>x</a>");
    assert_eq!(estilo(&doc, &arbol, "h1").margen.arriba, Longitud::Px(0.0));
    assert_eq!(estilo(&doc, &arbol, "a").color, ROJO);
}

// --- Recolección de hojas y árbol ---

#[test]
fn junta_todos_los_style_en_orden() {
    let doc = html::parsear("<style>p{color:red}</style><p>x</p><style>p{color:blue}</style>");
    let hojas = hojas_del_documento(&doc);
    assert_eq!(hojas.len(), 2);
    let arbol = estilar(&doc, &hojas);
    assert_eq!(estilo(&doc, &arbol, "p").color, AZUL);
}

#[test]
fn la_raiz_es_el_documento_con_estilo_inicial() {
    let (doc, arbol) = estilar_pagina("<p>x</p>");
    assert_eq!(arbol.nodo, doc.raiz());
    assert_eq!(arbol.estilo.display, Display::Bloque);
    assert_eq!(arbol.estilo.tam_fuente, 16.0);
    assert_eq!(arbol.estilo.color, NEGRO);
}

#[test]
fn los_textos_heredan_y_fluyen_en_linea() {
    let (doc, arbol) =
        estilar_pagina("<style>div { color: red; padding: 5px; font-size: 20px }</style><div>hola <b>x</b></div>");
    let div = buscar_por(&doc, &arbol, &|d, n| d.nombre(n) == Some("div")).unwrap();
    let texto = &div.hijos[0];
    assert_eq!(texto.texto.as_deref(), Some("hola "));
    assert_eq!(texto.estilo.display, Display::EnLinea);
    assert_eq!(texto.estilo.color, ROJO);
    assert_eq!(texto.estilo.tam_fuente, 20.0);
    assert_eq!(texto.estilo.relleno, Lados::todos(Longitud::Px(0.0)));
    assert!(texto.hijos.is_empty());
}

#[test]
fn los_textos_de_solo_espacios_se_conservan() {
    let (doc, arbol) = estilar_pagina("<p><b>a</b> <i>b</i></p>");
    let p = buscar_por(&doc, &arbol, &|d, n| d.nombre(n) == Some("p")).unwrap();
    assert_eq!(p.hijos.len(), 3);
    assert_eq!(p.hijos[1].texto.as_deref(), Some(" "));
}

#[test]
fn display_none_elimina_el_nodo_y_sus_hijos() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>.oculto { display: none }</style><div class="oculto"><p>x</p></div><span>y</span>"#,
    );
    assert!(!existe(&doc, &arbol, "div"));
    assert!(!existe(&doc, &arbol, "p"));
    assert!(existe(&doc, &arbol, "span"));
}

#[test]
fn valor_desconocido_se_ignora_y_gana_el_anterior() {
    let (doc, arbol) = estilar_pagina("<style>div { display: none } div { display: banana }</style><div>x</div>");
    assert!(!existe(&doc, &arbol, "div"));
    let (doc, arbol) = estilar_pagina("<style>p { font-weight: gordita }</style><h1>x</h1><p>y</p>");
    assert!(!estilo(&doc, &arbol, "p").negrita);
    assert!(estilo(&doc, &arbol, "h1").negrita);
}

// --- Cascada de punta a punta ---

#[test]
fn especificidad_orden_inline_e_important() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>
            #a { color: red }  .c { color: blue }  p { color: green }
            .x { color: red } .y { color: blue }
            p.imp { color: green !important } #imp { color: red }
        </style>
        <p id="a" class="c">1</p>
        <p id="b" class="x y">2</p>
        <p id="c" class="c" style="color: green">3</p>
        <p id="imp" class="imp" style="color: blue">4</p>"#,
    );
    assert_eq!(estilo(&doc, &arbol, "#a").color, ROJO);
    assert_eq!(estilo(&doc, &arbol, "#b").color, AZUL);
    assert_eq!(estilo(&doc, &arbol, "#c").color, VERDE);
    assert_eq!(estilo(&doc, &arbol, "#imp").color, VERDE);
}

#[test]
fn descendiente_de_punta_a_punta() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>nav a { color: red } a { color: blue }</style><nav><ul><li><a id="n">x</a></li></ul></nav><a id="f">y</a>"#,
    );
    assert_eq!(estilo(&doc, &arbol, "#n").color, ROJO);
    assert_eq!(estilo(&doc, &arbol, "#f").color, AZUL);
}

// --- Herencia ---

#[test]
fn las_propiedades_de_texto_se_heredan() {
    let (doc, arbol) = estilar_pagina(
        "<style>div { color: red; font-size: 20px; font-weight: bold; font-style: italic; text-align: center }</style>\
         <div><p><span>x</span></p></div>",
    );
    let span = estilo(&doc, &arbol, "span");
    assert_eq!(span.color, ROJO);
    assert_eq!(span.tam_fuente, 20.0);
    assert!(span.negrita && span.cursiva);
    assert_eq!(span.alineacion, Alineacion::Centro);
}

#[test]
fn las_propiedades_de_caja_no_se_heredan() {
    let (doc, arbol) = estilar_pagina(
        "<style>div { margin: 10px; padding: 3px; background-color: red; width: 50%; border: 2px solid }</style>\
         <div><p>x</p></div>",
    );
    let p = estilo(&doc, &arbol, "p");
    assert_eq!(p.margen.izquierda, Longitud::Px(0.0));
    assert_eq!(p.relleno, Lados::todos(Longitud::Px(0.0)));
    assert_eq!(p.fondo, TRANSPARENTE);
    assert_eq!(p.ancho, Longitud::Auto);
    assert_eq!(p.borde_ancho, Lados::todos(0.0));
    assert_eq!(estilo(&doc, &arbol, "div").ancho, Longitud::Porcentaje(50.0));
}

#[test]
fn inherit_initial_y_unset() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>
            div { color: red; padding: 7px; font-size: 20px }
            #i { padding: inherit }
            #n { color: initial; font-size: initial }
            #u { color: blue; color: unset; padding: 3px; padding: unset }
        </style>
        <div><p id="i">a</p><p id="n">b</p><p id="u">c</p></div>"#,
    );
    assert_eq!(estilo(&doc, &arbol, "#i").relleno, Lados::todos(Longitud::Px(7.0)));
    let n = estilo(&doc, &arbol, "#n");
    assert_eq!((n.color, n.tam_fuente), (NEGRO, 16.0));
    let u = estilo(&doc, &arbol, "#u");
    // unset: `color` se hereda (rojo del div); `padding` no se hereda (0).
    assert_eq!(u.color, ROJO);
    assert_eq!(u.relleno.arriba, Longitud::Px(0.0));
}

#[test]
fn el_subrayado_se_propaga_y_no_se_puede_quitar() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>b { text-decoration: none }</style><a href="x">ver <b>esto</b></a><p>y</p>"#,
    );
    assert!(estilo(&doc, &arbol, "b").subrayado);
    assert!(!estilo(&doc, &arbol, "p").subrayado);
    let (doc, arbol) = estilar_pagina(r#"<p style="text-decoration: underline red">x</p>"#);
    assert!(estilo(&doc, &arbol, "p").subrayado);
}

// --- Tamaños y em ---

#[test]
fn em_anidados_se_multiplican() {
    let (doc, arbol) = estilar_pagina(
        "<style>div { font-size: 1.5em } p { font-size: 50% }</style>\
         <div id=\"a\"><div id=\"b\"><p>x</p></div></div>",
    );
    assert_eq!(estilo(&doc, &arbol, "#a").tam_fuente, 24.0);
    assert_eq!(estilo(&doc, &arbol, "#b").tam_fuente, 36.0);
    assert_eq!(estilo(&doc, &arbol, "p").tam_fuente, 18.0);
}

#[test]
fn em_en_otras_propiedades_usa_la_letra_propia() {
    let (doc, arbol) =
        estilar_pagina("<style>div { font-size: 10px } p { font-size: 20px; margin-left: 2em; width: 3em }</style><div><p>x</p></div>");
    let p = estilo(&doc, &arbol, "p");
    assert_eq!(p.margen.izquierda, Longitud::Px(40.0));
    assert_eq!(p.ancho, Longitud::Px(60.0));
}

#[test]
fn tamanos_con_palabra_clave() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>#a { font-size: small } #b { font-size: x-large } #c { font-size: larger }</style>
        <p id="a">a</p><p id="b">b</p><div style="font-size: 10px"><p id="c">c</p></div>"#,
    );
    assert_eq!(estilo(&doc, &arbol, "#a").tam_fuente, 13.0);
    assert_eq!(estilo(&doc, &arbol, "#b").tam_fuente, 24.0);
    assert_eq!(estilo(&doc, &arbol, "#c").tam_fuente, 12.0);
}

#[test]
fn alto_de_linea() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>
            #normal { font-size: 20px }
            #factor { line-height: 1.5; font-size: 10px }
            #px { line-height: 30px; font-size: 10px }
            #em { line-height: 2em; font-size: 10px }
            span { font-size: 20px }
        </style>
        <p id="normal">a</p>
        <div id="factor"><span id="f">b</span></div>
        <div id="px"><span id="p">c</span></div>
        <div id="em"><span id="e">d</span></div>"#,
    );
    assert_eq!(estilo(&doc, &arbol, "#normal").alto_linea, 24.0);
    assert_eq!(estilo(&doc, &arbol, "#factor").alto_linea, 15.0);
    // Un número se hereda como factor: 1.5 × 20px de la letra del hijo.
    assert_eq!(estilo(&doc, &arbol, "#f").alto_linea, 30.0);
    // Una longitud se hereda ya convertida a px.
    assert_eq!(estilo(&doc, &arbol, "#p").alto_linea, 30.0);
    assert_eq!(estilo(&doc, &arbol, "#e").alto_linea, 20.0);
}

#[test]
fn peso_de_la_letra() {
    let (doc, arbol) = estilar_pagina(
        r#"<p id="a" style="font-weight: 700">a</p><p id="b" style="font-weight: 500">b</p>
        <p id="c" style="font-weight: bolder">c</p><h1 id="d" style="font-weight: normal">d</h1>"#,
    );
    assert!(estilo(&doc, &arbol, "#a").negrita);
    assert!(!estilo(&doc, &arbol, "#b").negrita);
    assert!(estilo(&doc, &arbol, "#c").negrita);
    assert!(!estilo(&doc, &arbol, "#d").negrita);
}

#[test]
fn justify_queda_a_la_izquierda() {
    let (doc, arbol) = estilar_pagina(r#"<p style="text-align: justify">x</p>"#);
    assert_eq!(estilo(&doc, &arbol, "p").alineacion, Alineacion::Izquierda);
}

// --- Bordes y colores ---

#[test]
fn borde_sin_estilo_mide_cero() {
    let (doc, arbol) = estilar_pagina(
        r#"<div id="a" style="border-width: 5px">a</div>
        <div id="b" style="border: 4px none red">b</div>
        <div id="c" style="border: 4px solid red; border-left-style: hidden">c</div>
        <div id="d" style="border-style: solid">d</div>
        <div id="e" style="border: thin solid; border-bottom-width: thick">e</div>"#,
    );
    assert_eq!(estilo(&doc, &arbol, "#a").borde_ancho, Lados::todos(0.0));
    assert_eq!(estilo(&doc, &arbol, "#b").borde_ancho, Lados::todos(0.0));
    let c = estilo(&doc, &arbol, "#c");
    assert_eq!(c.borde_ancho.en_orden(), [4.0, 4.0, 4.0, 0.0]);
    // Con estilo y sin ancho: `medium` = 3px.
    assert_eq!(estilo(&doc, &arbol, "#d").borde_ancho, Lados::todos(3.0));
    assert_eq!(estilo(&doc, &arbol, "#e").borde_ancho.en_orden(), [1.0, 1.0, 5.0, 1.0]);
}

#[test]
fn currentcolor_es_el_color_final_del_elemento() {
    let (doc, arbol) = estilar_pagina(
        r#"<style>
            div { border: 1px solid; color: red }
            p { color: blue; background-color: currentcolor; border-top: 2px solid currentcolor }
            span { color: currentcolor }
        </style>
        <div>a</div><p>b <span>c</span></p>"#,
    );
    assert_eq!(estilo(&doc, &arbol, "div").borde_color, Lados::todos(ROJO));
    let p = estilo(&doc, &arbol, "p");
    assert_eq!(p.fondo, AZUL);
    assert_eq!(p.borde_color.arriba, AZUL);
    // En `color` mismo, currentcolor es el color heredado.
    assert_eq!(estilo(&doc, &arbol, "span").color, AZUL);
}

#[test]
fn background_sin_color_es_transparente() {
    let (doc, arbol) =
        estilar_pagina(r#"<style>p { background-color: red } p { background: url(x.png) }</style><p>x</p>"#);
    assert_eq!(estilo(&doc, &arbol, "p").fondo.a, 0);
}

#[test]
fn max_width_none_y_porcentajes() {
    let (doc, arbol) = estilar_pagina(
        r#"<div id="a" style="max-width: 600px; margin: 0 auto; padding: 5%">a</div><div id="b" style="max-width: none">b</div>"#,
    );
    let a = estilo(&doc, &arbol, "#a");
    assert_eq!(a.ancho_maximo, Longitud::Px(600.0));
    assert_eq!(a.margen.izquierda, Longitud::Auto);
    assert_eq!(a.relleno.arriba, Longitud::Porcentaje(5.0));
    assert_eq!(estilo(&doc, &arbol, "#b").ancho_maximo, Longitud::Auto);
}

#[test]
fn padding_auto_no_vale() {
    let (doc, arbol) = estilar_pagina(r#"<p style="padding-left: 4px; padding-left: auto">x</p>"#);
    assert_eq!(estilo(&doc, &arbol, "p").relleno.izquierda, Longitud::Px(4.0));
}

// --- Las páginas de la carpeta pruebas ---

#[test]
fn pagina_01_hola() {
    let (doc, arbol) = estilar_pagina(include_str!("../../pruebas/01-hola.html"));
    assert!(!existe(&doc, &arbol, "head"));
    let h1 = estilo(&doc, &arbol, "h1");
    assert_eq!(h1.color, Color::rgb(0x0f, 0x6e, 0x6e));
    assert_eq!(h1.tam_fuente, 32.0);
    let caja = buscar_por(&doc, &arbol, &|d, n| emparejar::atributo(d, n, "class") == Some("caja"))
        .unwrap();
    assert_eq!(caja.estilo.borde_ancho, Lados::todos(2.0));
    assert_eq!(caja.estilo.relleno, Lados::todos(Longitud::Px(10.0)));
}

#[test]
fn pagina_05_cascada() {
    let (doc, arbol) = estilar_pagina(include_str!("../../pruebas/05-cascada.html"));
    assert!(!existe(&doc, &arbol, "aside"));
    assert!(arbol.imprimir(&doc).contains("<p id=\"especifico\""));
}

#[test]
fn imprimir_resume_lo_que_cambia() {
    let (doc, arbol) = estilar_pagina(
        r#"<html><body><h1 style="color: #0f6e6e">Hola</h1><p class="c">x</p></body></html>"#,
    );
    let texto = arbol.imprimir(&doc);
    assert!(texto.contains("<body> bloque · margen 8px\n"), "{texto}");
    assert!(texto.contains("<h1> bloque · color #0f6e6e · 32px negrita · margen 21.44px 0\n"), "{texto}");
    assert!(texto.contains("<p class=\"c\"> bloque · margen 16px 0\n"), "{texto}");
    assert!(texto.contains("\"Hola\""));
}

#[test]
fn explicar_muestra_ganadora_y_perdedoras() {
    let doc = html::parsear(r#"<style>p { color: red } .n { color: blue }</style><p class="n">x</p>"#);
    let hojas = hojas_del_documento(&doc);
    let selector = css::parsear("p {}").reglas.remove(0).selectores.remove(0);
    let texto = explicar(&doc, &hojas, &selector);
    let gana = texto.lines().position(|l| l.contains("gana") && l.contains(".n (0,1,0)"));
    let pierde = texto.lines().position(|l| l.contains("pierde") && l.contains("p (0,0,1)"));
    assert!(gana.is_some() && pierde.is_some() && gana < pierde, "{texto}");
    assert!(texto.contains("navegador · p"), "{texto}");
}
