//! Farol: un navegador web desde cero.

mod css;
mod estilo;
mod html;
mod layout;
mod pintado;
mod red;
mod ui;

use pintado::Lienzo;

/// Nombre y versión, leídos de `Cargo.toml` al compilar.
const NOMBRE: &str = "Farol";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("{NOMBRE} {VERSION}");
    if let Err(error) = ui::abrir(&format!("{NOMBRE} {VERSION}"), escena_de_prueba) {
        eprintln!("Farol no pudo abrir la ventana: {error}");
        std::process::exit(1);
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
