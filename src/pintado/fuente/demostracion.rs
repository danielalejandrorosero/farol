//! La escena del hito de la fase 7 (`farol --fuente`): la fuente en varios tamaños, colores y
//! estilos, más todos sus caracteres.

use super::{alto_linea_natural, linea_base};
use crate::pintado::Lienzo;

const FONDO: u32 = 0xFBFAF7;
const TINTA: u32 = 0x1F2328;
const GRIS: u32 = 0x6A737D;
const BARRA: u32 = 0x0F6E6E;

pub fn escena(lienzo: &mut Lienzo) {
    let ancho = lienzo.ancho() as i32;
    lienzo.rectangulo(0, 0, ancho, lienzo.alto() as i32, FONDO);
    lienzo.rectangulo(0, 0, ancho, 40, BARRA);
    lienzo.texto(
        16.0,
        10.0,
        "Farol · fase 7: texto y fuentes",
        16.0,
        0xFFFFFF,
        true,
        false,
    );

    let x = 16.0;
    let mut y = 52.0;
    let colores = [0x1F2328, 0xD64545, 0x3FA34D, 0x3B6FD4, 0x8A4FBF];
    for (tam, color) in [12.0, 16.0, 24.0, 32.0, 48.0].into_iter().zip(colores) {
        // El tamaño se anota en gris a la izquierda, sobre la misma línea base que la frase.
        let y_etiqueta = y + linea_base(tam) - linea_base(12.0);
        lienzo.texto(
            x,
            y_etiqueta,
            &format!("{tam} px"),
            12.0,
            GRIS,
            false,
            false,
        );
        lienzo.texto(
            x + 44.0,
            y,
            "Hola, mundo. ¿Qué tal, señor Ñandú? ¡Farol!",
            tam,
            color,
            false,
            false,
        );
        y += alto_linea_natural(tam);
    }

    // Escribe una línea y baja la pluma lo que mide esa línea.
    let mut linea = |lienzo: &mut Lienzo, texto: &str, tam: f32, color, negrita, cursiva| {
        lienzo.texto(x, y, texto, tam, color, negrita, cursiva);
        y += alto_linea_natural(tam);
    };

    linea(
        lienzo,
        "Negrita: ¿Qué tal, señor Ñandú? ¡Farol!",
        24.0,
        TINTA,
        true,
        false,
    );
    linea(
        lienzo,
        "Cursiva: ¿Qué tal, señor Ñandú? ¡Farol!",
        24.0,
        TINTA,
        false,
        true,
    );
    linea(
        lienzo,
        "Negrita y cursiva a 16 px: «Pingüino» — 25 °C",
        16.0,
        GRIS,
        true,
        true,
    );

    for tam in [16.0, 32.0] {
        linea(
            lienzo,
            "ABCDEFGHIJKLMNÑOPQRSTUVWXYZ ÁÉÍÓÚÜ",
            tam,
            TINTA,
            false,
            false,
        );
        linea(
            lienzo,
            "abcdefghijklmnñopqrstuvwxyz áéíóúü",
            tam,
            TINTA,
            false,
            false,
        );
        linea(
            lienzo,
            "0123456789 !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~",
            tam,
            TINTA,
            false,
            false,
        );
        linea(
            lienzo,
            "¿¡ © ° « » – — “ ” ‘ ’ … · l1I O0 rn m 漢",
            tam,
            GRIS,
            false,
            false,
        );
    }
}
