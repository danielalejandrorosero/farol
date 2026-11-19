//! Farol: un navegador web desde cero.

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
  farol                      abre la ventana
  farol --dom <archivo>      imprime el árbol DOM de una página HTML
  farol --css <archivo>      imprime las reglas de una hoja de estilos CSS";

fn main() {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    match argumentos.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => abrir_ventana(),
        ["--dom", ruta] => imprimir_dom(ruta),
        ["--css", ruta] => imprimir_css(ruta),
        _ => {
            eprintln!("{USO}");
            exit(2);
        }
    }
}

fn imprimir_dom(ruta: &str) {
    let bytes = match std::fs::read(ruta) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("No se pudo leer {ruta}: {error}");
            exit(1);
        }
    };
    // Si el archivo trae bytes que no son UTF-8 válido, se reemplazan por � en vez de fallar.
    let texto = String::from_utf8_lossy(&bytes);
    print!("{}", html::parsear(&texto).imprimir());
}

fn imprimir_css(ruta: &str) {
    let bytes = match std::fs::read(ruta) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("No se pudo leer {ruta}: {error}");
            exit(1);
        }
    };
    let (hoja, avisos) = css::parsear_con_avisos(&String::from_utf8_lossy(&bytes));
    print!("{}", hoja.imprimir());
    let declaraciones: usize = hoja.reglas.iter().map(|r| r.declaraciones.len()).sum();
    println!("
{} reglas, {declaraciones} declaraciones.", hoja.reglas.len());
    if !avisos.is_empty() {
        println!("
Descartado ({}):", avisos.len());
        for aviso in avisos {
            println!("  - {aviso}");
        }
    }
}

fn abrir_ventana() {
    println!("{NOMBRE} {VERSION}");
    if let Err(error) = ui::abrir(&format!("{NOMBRE} {VERSION}"), escena_de_prueba) {
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
