//! Farol: un navegador web desde cero.

mod comandos;
mod css;
mod estilo;
mod html;
mod layout;
mod pintado;
mod red;
mod ui;

use pintado::Lienzo;
use std::process::exit;

/// Nombre y versión, leídos de `Cargo.toml` al compilar.
const NOMBRE: &str = "Farol";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const USO: &str = "Uso:
  farol                      abre la ventana con la escena de prueba
  farol <archivo>            abre una página HTML en la ventana
  farol --dom <archivo>      imprime el árbol DOM de una página HTML
  farol --css <archivo>      imprime las reglas de una hoja de estilos CSS
  farol --estilo <archivo> [selector]
                             imprime el árbol estilado de una página HTML; con un selector,
                             qué declaraciones pelean en cada elemento y cuál gana
  farol --layout <archivo> [ancho]
                             imprime el árbol de cajas con posición y tamaño (ancho 800)
  farol --render <archivo> <salida.bmp> [ancho [alto]]
                             pinta la página en una imagen BMP, sin abrir ventana (ancho 800;
                             alto: el de la página)
  farol --fuente             abre la ventana con la demostración de la fuente";

fn main() {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    match argumentos.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => abrir_ventana(escena_de_prueba),
        ["--fuente"] => abrir_ventana(pintado::fuente::demostracion::escena),
        ["--dom", ruta] => comandos::imprimir_dom(ruta),
        ["--css", ruta] => comandos::imprimir_css(ruta),
        ["--estilo", ruta, resto @ ..] if resto.len() <= 1 => {
            comandos::imprimir_estilo(ruta, resto.first())
        }
        ["--layout", ruta] => comandos::imprimir_layout(ruta, "800"),
        ["--layout", ruta, ancho] => comandos::imprimir_layout(ruta, ancho),
        ["--render", ruta, salida, medidas @ ..] if medidas.len() <= 2 => {
            comandos::renderizar(ruta, salida, medidas.first().copied(), medidas.get(1).copied())
        }
        [ruta] if !ruta.starts_with("--") => {
            println!("{NOMBRE} {VERSION}");
            comandos::abrir_pagina(ruta, NOMBRE)
        }
        _ => {
            eprintln!("{USO}");
            exit(2);
        }
    }
}

/// Abre la ventana con una escena fija (sin página).
fn abrir_ventana(escena: fn(&mut Lienzo)) {
    println!("{NOMBRE} {VERSION}");
    if let Err(error) = ui::abrir(&format!("{NOMBRE} {VERSION}"), escena) {
        eprintln!("Farol no pudo abrir la ventana: {error}");
        exit(1);
    }
}

/// La escena de la fase 1: rectángulos cuyo tamaño depende del tamaño de la ventana,
/// para comprobar que todo se redibuja al redimensionar.
fn escena_de_prueba(lienzo: &mut Lienzo) {
    let ancho = lienzo.ancho() as i32;
    let alto = lienzo.alto() as i32;

    const FONDO: u32 = 0xF3F5F7;
    const BARRA: u32 = 0x0F6E6E;
    const ROJO: u32 = 0xD64545;
    const VERDE: u32 = 0x3FA34D;
    const AZUL: u32 = 0x3B6FD4;
    const AMBAR: u32 = 0xF2B134;

    lienzo.rectangulo(0, 0, ancho, alto, FONDO);
    // Una barra superior de alto fijo, como la futura barra de direcciones.
    lienzo.rectangulo(0, 0, ancho, 48, BARRA);

    // Tres columnas que se reparten el ancho disponible.
    let margen = 24;
    let columna = (ancho - margen * 4) / 3;
    let alto_columna = alto - 48 - margen * 2 - 60;
    for (i, color) in [ROJO, VERDE, AZUL].into_iter().enumerate() {
        let x = margen + (columna + margen) * i as i32;
        lienzo.rectangulo(x, 48 + margen, columna, alto_columna, color);
    }

    // Una franja inferior pegada al borde de abajo.
    lienzo.rectangulo(margen, alto - margen - 36, ancho - margen * 2, 36, AMBAR);
}
