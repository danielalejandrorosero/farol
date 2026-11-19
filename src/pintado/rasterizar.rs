//! Rasterizar: ejecutar la display list sobre el lienzo, convirtiendo cada orden en pixeles.
//!
//! Las cajas viven en coordenadas con decimales (`x = 28.5`), pero la pantalla es una cuadrícula
//! de pixeles enteros. Aquí se decide qué pixeles cubre cada rectángulo y de qué color quedan.
//!
//! **Redondeo:** cada *borde* del rectángulo se redondea por separado al entero más cercano
//! (`f32::round`), y se pintan los pixeles desde el borde izquierdo hasta antes del derecho. No
//! se redondea el ancho. Así, si el fondo termina en `x = 30.5` y el borde empieza en `30.5`, los
//! dos redondean al mismo 31 y no queda ni un hueco ni un pixel pintado dos veces.
//!
//! **Transparencia (decisión 6.1):** el lienzo es opaco. Un color con alfa < 255 se mezcla con lo
//! que ya hay debajo: `resultado = fondo + (color − fondo) × a / 255`, canal por canal.

use super::{Lienzo, Orden};
use crate::css;
use crate::layout::Rect;

const BLANCO: u32 = 0xFFFFFF;

/// Un `css::Color` opaco como pixel `0x00RRGGBB` (el alfa se ignora).
pub fn a_pixel(color: css::Color) -> u32 {
    (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
}

/// `color` pintado encima de `fondo`, con su transparencia.
pub fn mezclar(fondo: u32, color: css::Color) -> u32 {
    match color.a {
        255 => a_pixel(color),
        0 => fondo,
        a => {
            let a = a as u32;
            let canal = |desplazamiento: u32, nuevo: u8| {
                let viejo = (fondo >> desplazamiento) & 0xFF;
                // fondo × (255 − a) + color × a, sobre 255, redondeado (+127).
                (viejo * (255 - a) + nuevo as u32 * a + 127) / 255
            };
            canal(16, color.r) << 16 | canal(8, color.g) << 8 | canal(0, color.b)
        }
    }
}

/// Los pixeles que cubre `rect`: `(x0, y0, x1, y1)`, con `x1` e `y1` excluidos.
/// `desplazamiento` es cuánto se bajó la página (scroll): se resta a `y`.
pub fn pixeles_de(rect: Rect, desplazamiento: f32) -> (i32, i32, i32, i32) {
    let y = rect.y - desplazamiento;
    // `as i32` satura: infinitos o números enormes quedan en i32::MAX/MIN y el lienzo los recorta.
    (
        rect.x.round() as i32,
        y.round() as i32,
        (rect.x + rect.ancho).round() as i32,
        (y + rect.alto).round() as i32,
    )
}

impl Lienzo {
    /// Pinta la display list con la página sin desplazar.
    pub fn ejecutar(&mut self, ordenes: &[Orden]) {
        self.ejecutar_desplazado(ordenes, 0.0);
    }

    /// Pinta la display list con la página bajada `desplazamiento` pixeles (el scroll de la
    /// fase 11): lo que está en `y = desplazamiento` queda en la fila 0 del lienzo.
    pub fn ejecutar_desplazado(&mut self, ordenes: &[Orden], desplazamiento: f32) {
        for orden in ordenes {
            match orden {
                Orden::FondoDelLienzo(color) => self.pixeles.fill(mezclar(BLANCO, *color)),
                Orden::Rectangulo { rect, color } => {
                    let (x0, y0, x1, y1) = pixeles_de(*rect, desplazamiento);
                    self.rellenar(x0, y0, x1, y1, *color);
                }
            }
        }
    }

    /// Rellena los pixeles de `x0..x1` × `y0..y1` (recortados al lienzo), mezclando si hace falta.
    fn rellenar(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: css::Color) {
        let recortar = |v: i32, max: usize| v.clamp(0, max as i32) as usize;
        let (x0, x1) = (recortar(x0, self.ancho), recortar(x1, self.ancho));
        let (y0, y1) = (recortar(y0, self.alto), recortar(y1, self.alto));
        if x0 >= x1 {
            return;
        }
        for fila in y0..y1 {
            let inicio = fila * self.ancho;
            let tramo = &mut self.pixeles[inicio + x0..inicio + x1];
            if color.a == 255 {
                tramo.fill(a_pixel(color)); // opaco: tapa lo de abajo sin mirarlo
            } else {
                for pixel in tramo {
                    *pixel = mezclar(*pixel, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn rgba(r: u8, g: u8, b: u8, a: u8) -> css::Color {
        css::Color { r, g, b, a }
    }

    fn rect(x: f32, y: f32, ancho: f32, alto: f32) -> Rect {
        Rect { x, y, ancho, alto }
    }

    fn cerca(a: u32, b: u32) -> bool {
        [16, 8, 0].iter().all(|&d| (((a >> d) & 0xFF) as i32 - ((b >> d) & 0xFF) as i32).abs() <= 1)
    }

    #[test]
    fn convierte_css_a_pixel() {
        assert_eq!(a_pixel(css::Color::rgb(0x0F, 0x6E, 0x6E)), 0x0F6E6E);
    }

    #[test]
    fn rojo_al_50_por_ciento_sobre_blanco_es_rosado() {
        let resultado = mezclar(0xFFFFFF, rgba(255, 0, 0, 128));
        assert!(cerca(resultado, 0xFF8080), "{resultado:06X}");
        assert_eq!(mezclar(0x123456, rgba(1, 2, 3, 0)), 0x123456);
        assert_eq!(mezclar(0x123456, rgba(1, 2, 3, 255)), 0x010203);
    }

    #[test]
    fn la_mezcla_se_aplica_sobre_lo_que_ya_estaba_pintado() {
        let mut lienzo = Lienzo::nuevo(2, 1, 0x000000);
        lienzo.ejecutar(&[
            Orden::Rectangulo { rect: rect(0.0, 0.0, 1.0, 1.0), color: css::Color::rgb(0, 0, 255) },
            Orden::Rectangulo { rect: rect(0.0, 0.0, 2.0, 1.0), color: rgba(255, 255, 255, 128) },
        ]);
        assert!(cerca(lienzo.pixeles()[0], 0x8080FF), "{:06X}", lienzo.pixeles()[0]);
        assert!(cerca(lienzo.pixeles()[1], 0x808080), "{:06X}", lienzo.pixeles()[1]);
    }

    #[test]
    fn el_fondo_del_lienzo_cubre_todo() {
        let mut lienzo = Lienzo::nuevo(3, 3, 0);
        lienzo.ejecutar(&[Orden::FondoDelLienzo(css::Color::rgb(1, 2, 3))]);
        assert!(lienzo.pixeles().iter().all(|&p| p == 0x010203));
    }

    #[test]
    fn redondeo_sin_huecos_entre_fondo_y_borde() {
        // Un fondo de 10.4 a 20.6 y un borde izquierdo de 8.9 a 10.4: se tocan en 10.4.
        let mut lienzo = Lienzo::nuevo(30, 1, 0);
        lienzo.ejecutar(&[
            Orden::Rectangulo { rect: rect(8.9, 0.0, 1.5, 1.0), color: css::Color::rgb(0, 0, 255) },
            Orden::Rectangulo {
                rect: rect(10.4, 0.0, 10.2, 1.0),
                color: css::Color::rgb(255, 0, 0),
            },
        ]);
        let p = lienzo.pixeles();
        assert_eq!(p[8], 0);
        assert_eq!(&p[9..10], [0x0000FF]);
        assert!(p[10..21].iter().all(|&c| c == 0xFF0000));
        assert_eq!(p[21], 0);
    }

    #[test]
    fn el_desplazamiento_sube_la_pagina() {
        let mut lienzo = Lienzo::nuevo(1, 3, 0);
        let ordenes = [Orden::Rectangulo {
            rect: rect(0.0, 5.0, 1.0, 1.0),
            color: css::Color::rgb(255, 0, 0),
        }];
        lienzo.ejecutar_desplazado(&ordenes, 4.0);
        assert_eq!(lienzo.pixeles(), [0, 0xFF0000, 0]);
    }

    #[test]
    fn coordenadas_enormes_no_fallan() {
        for color in [rgba(255, 0, 0, 100), css::Color::rgb(0, 255, 0)] {
            let mut lienzo = Lienzo::nuevo(2, 2, 0);
            lienzo.ejecutar(&[Orden::Rectangulo {
                rect: rect(-1e30, -5.0, f32::INFINITY, 1e30),
                color,
            }]);
            assert!(lienzo.pixeles().iter().all(|&p| p != 0));
        }
    }
}
