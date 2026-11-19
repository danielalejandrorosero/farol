//! Los comandos de la línea de comandos: cada uno arma una parte de la tubería y muestra el
//! resultado (en la terminal, en un archivo o en la ventana).

use std::process::exit;

use crate::pintado::{self, Lienzo};
use crate::{css, estilo, html, layout, ui};

/// Alto mínimo de la imagen de `--render` cuando la página mide menos (o nada).
const ALTO_MINIMO_RENDER: usize = 100;

/// Lee un archivo como texto. Si trae bytes que no son UTF-8 válido, se reemplazan por � en vez
/// de fallar. Si no se puede leer, explica por qué y termina.
fn leer(ruta: &str) -> String {
    match std::fs::read(ruta) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(error) => {
            eprintln!("No se pudo leer {ruta}: {error}");
            exit(1);
        }
    }
}

/// Un número de pixeles escrito en la línea de comandos, o termina con un mensaje.
fn pixeles(texto: &str, que: &str) -> usize {
    match texto.parse::<usize>() {
        Ok(n) if n > 0 => n,
        _ => {
            eprintln!("El {que} debe ser un número entero de pixeles mayor que 0, no `{texto}`.");
            exit(2);
        }
    }
}

pub fn imprimir_dom(ruta: &str) {
    print!("{}", html::parsear(&leer(ruta)).imprimir());
}

pub fn imprimir_css(ruta: &str) {
    let (hoja, avisos) = css::parsear_con_avisos(&leer(ruta));
    print!("{}", hoja.imprimir());
    let declaraciones: usize = hoja.reglas.iter().map(|r| r.declaraciones.len()).sum();
    println!("\n{} reglas, {declaraciones} declaraciones.", hoja.reglas.len());
    if !avisos.is_empty() {
        println!("\nDescartado ({}):", avisos.len());
        for aviso in avisos {
            println!("  - {aviso}");
        }
    }
}

pub fn imprimir_estilo(ruta: &str, selector: Option<&&str>) {
    let doc = html::parsear(&leer(ruta));
    let hojas = estilo::hojas_del_documento(&doc);
    let Some(selector) = selector else {
        print!("{}", estilo::estilar(&doc, &hojas).imprimir(&doc));
        return;
    };
    // El selector se lee con el mismo parser de CSS, como si fuera el comienzo de una regla.
    match css::parsear(&format!("{selector} {{}}")).reglas.first().map(|r| &r.selectores[..]) {
        Some([selector]) => print!("{}", estilo::explicar(&doc, &hojas, selector)),
        _ => {
            eprintln!("No se entiende el selector `{selector}` (se acepta uno solo, sin comas).");
            exit(2);
        }
    }
}

pub fn imprimir_layout(ruta: &str, ancho: &str) {
    let ancho = pixeles(ancho, "ancho");
    let doc = html::parsear(&leer(ruta));
    let estilado = estilo::estilar(&doc, &estilo::hojas_del_documento(&doc));
    print!("{}", layout::calcular(&estilado, ancho as f32).imprimir(&doc));
}

/// `farol <archivo.html>`: la página en la ventana.
///
/// El DOM y los estilos se calculan una sola vez, aquí. El layout y el pintado se repiten en
/// cada repintado, porque dependen del ancho de la ventana: al redimensionar, el texto y las
/// cajas `auto` o en `%` se reacomodan.
pub fn abrir_pagina(ruta: &str, titulo_programa: &str) {
    let doc = html::parsear(&leer(ruta));
    let estilado = estilo::estilar(&doc, &estilo::hojas_del_documento(&doc));
    let nombre = std::path::Path::new(ruta).file_name().map(|n| n.to_string_lossy().into_owned());
    let titulo = doc.titulo().or(nombre).unwrap_or_else(|| ruta.to_string());

    // `move` mete `estilado` dentro del cierre: desde ahora vive allí, junto a la ventana.
    let dibujar = move |lienzo: &mut Lienzo| {
        let cajas = layout::calcular(&estilado, lienzo.ancho() as f32);
        lienzo.ejecutar(&pintado::lista_de_pintado(&cajas));
    };
    if let Err(error) = ui::abrir(&format!("{titulo} — {titulo_programa}"), dibujar) {
        eprintln!("Farol no pudo abrir la ventana: {error}");
        exit(1);
    }
}

/// `farol --render <archivo.html> <salida.bmp> [ancho [alto]]`: la página pintada en un BMP,
/// sin abrir ventana (decisión 6.2). Sin `alto`, la imagen mide lo que mide la página.
pub fn renderizar(ruta: &str, salida: &str, ancho: Option<&str>, alto: Option<&str>) {
    let ancho = ancho.map_or(800, |a| pixeles(a, "ancho"));
    let doc = html::parsear(&leer(ruta));
    let estilado = estilo::estilar(&doc, &estilo::hojas_del_documento(&doc));
    let cajas = layout::calcular(&estilado, ancho as f32);
    let alto_pagina = cajas.dimensiones.contenido.alto.ceil() as usize;
    let alto = alto.map_or(alto_pagina.max(ALTO_MINIMO_RENDER), |a| pixeles(a, "alto"));

    let mut lienzo = Lienzo::nuevo(ancho, alto, 0xFFFFFF);
    lienzo.ejecutar(&pintado::lista_de_pintado(&cajas));
    if let Err(error) = std::fs::write(salida, pintado::bmp::codificar(&lienzo)) {
        eprintln!("No se pudo escribir {salida}: {error}");
        exit(1);
    }
    println!("{salida}: {ancho} × {alto} pixeles (la página mide {alto_pagina} de alto).");
}
