//! Pintado: recibe las cajas y entrega pixeles en el framebuffer. Fases 6 y 7.
//!
//! Dos pasos, como un arquitecto que primero escribe el plano y después un obrero lo ejecuta:
//!
//! 1. `lista.rs`: el árbol de cajas → la **display list** (`Vec<Orden>`), qué pintar y en qué
//!    orden, todavía sin pixeles.
//! 2. `rasterizar.rs`: `Lienzo::ejecutar` convierte cada orden en pixeles (redondeo y mezcla alfa).
//!
//! El `Lienzo` es la lista de pixeles sobre la que se dibuja. `fuente` le enseña a escribir y
//! `bmp` lo guarda como imagen (`farol --render`).

pub mod bmp;
pub mod fuente;
mod lista;
mod rasterizar;

pub use lista::{Orden, lista_de_pintado};

/// Un color en 32 bits con la forma `0x00RRGGBB`: dos dígitos hexadecimales por canal.
pub type Color = u32;

/// El framebuffer: una lista de `ancho * alto` colores, fila por fila, de arriba hacia abajo.
pub struct Lienzo {
    ancho: usize,
    alto: usize,
    pixeles: Vec<Color>,
}

impl Lienzo {
    /// Crea un lienzo relleno de un solo color.
    pub fn nuevo(ancho: usize, alto: usize, fondo: Color) -> Self {
        Self { ancho, alto, pixeles: vec![fondo; ancho * alto] }
    }

    pub fn ancho(&self) -> usize {
        self.ancho
    }

    pub fn alto(&self) -> usize {
        self.alto
    }

    /// Los pixeles tal como los espera la ventana.
    pub fn pixeles(&self) -> &[Color] {
        &self.pixeles
    }

    /// Rellena un rectángulo. La parte que quede fuera del lienzo se recorta.
    pub fn rectangulo(&mut self, x: i32, y: i32, ancho: i32, alto: i32, color: Color) {
        let x0 = x.clamp(0, self.ancho as i32) as usize;
        let y0 = y.clamp(0, self.alto as i32) as usize;
        let x1 = x.saturating_add(ancho).clamp(0, self.ancho as i32) as usize;
        let y1 = y.saturating_add(alto).clamp(0, self.alto as i32) as usize;
        for fila in y0..y1 {
            // El pixel (x, fila) vive en la posición fila * ancho + x de la lista.
            let inicio = fila * self.ancho;
            self.pixeles[inicio + x0..inicio + x1].fill(color);
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn un_lienzo_nuevo_tiene_el_color_de_fondo() {
        let lienzo = Lienzo::nuevo(4, 3, 0xFFFFFF);
        assert_eq!(lienzo.pixeles().len(), 12);
        assert!(lienzo.pixeles().iter().all(|&p| p == 0xFFFFFF));
    }

    #[test]
    fn el_rectangulo_pinta_solo_su_area() {
        let mut lienzo = Lienzo::nuevo(4, 3, 0);
        lienzo.rectangulo(1, 1, 2, 1, 0xFF0000);
        let esperado = [0, 0, 0, 0, 0, 0xFF0000, 0xFF0000, 0, 0, 0, 0, 0];
        assert_eq!(lienzo.pixeles(), esperado);
    }

    #[test]
    fn el_rectangulo_se_recorta_en_los_bordes() {
        let mut lienzo = Lienzo::nuevo(3, 2, 0);
        lienzo.rectangulo(-5, -5, 7, 100, 0x00FF00);
        assert_eq!(lienzo.pixeles(), [0x00FF00, 0x00FF00, 0, 0x00FF00, 0x00FF00, 0]);
    }

    #[test]
    fn un_rectangulo_totalmente_afuera_no_pinta_nada() {
        let mut lienzo = Lienzo::nuevo(3, 2, 0);
        lienzo.rectangulo(10, 10, 5, 5, 0xFFFFFF);
        lienzo.rectangulo(-10, 0, 5, 5, 0xFFFFFF);
        assert!(lienzo.pixeles().iter().all(|&p| p == 0));
    }
}
