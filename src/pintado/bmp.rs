//! Escritor de archivos BMP (decisión 6.2): el lienzo guardado como imagen, sin librerías.
//!
//! BMP es el formato de imagen más simple que Windows y cualquier visor abren: una cabecera de
//! 54 bytes y después los pixeles crudos, sin comprimir. Por dentro:
//!
//! ```text
//! byte  0  "BM"                       la firma: "soy un BMP"
//! byte  2  tamaño del archivo (u32)
//! byte 10  dónde empiezan los pixeles (54)
//! byte 14  40 = tamaño de la segunda cabecera (BITMAPINFOHEADER)
//! byte 18  ancho (i32), byte 22 alto (i32, positivo = filas de abajo hacia arriba)
//! byte 26  1 plano, byte 28 24 bits por pixel, byte 30 0 = sin compresión
//! byte 54  pixeles: B, G, R por pixel; cada fila se rellena con ceros hasta múltiplo de 4
//! ```
//!
//! Todos los números van en *little endian* (el byte menos significativo primero), como los
//! guarda el procesador. Las filas van al revés (la de abajo primero), herencia de la época en
//! que el origen de las coordenadas estaba abajo a la izquierda, como en matemáticas.

use super::Lienzo;

const CABECERAS: usize = 14 + 40;

/// Los bytes de un BMP de 24 bits con el contenido del lienzo.
pub fn codificar(lienzo: &Lienzo) -> Vec<u8> {
    let (ancho, alto) = (lienzo.ancho(), lienzo.alto());
    // 3 bytes por pixel, redondeado hacia arriba al múltiplo de 4.
    let bytes_fila = (ancho * 3).div_ceil(4) * 4;
    let tam_pixeles = bytes_fila * alto;
    let mut bmp = Vec::with_capacity(CABECERAS + tam_pixeles);

    // BITMAPFILEHEADER (14 bytes).
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&((CABECERAS + tam_pixeles) as u32).to_le_bytes());
    bmp.extend_from_slice(&[0; 4]); // dos campos reservados
    bmp.extend_from_slice(&(CABECERAS as u32).to_le_bytes());

    // BITMAPINFOHEADER (40 bytes).
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(ancho as i32).to_le_bytes());
    bmp.extend_from_slice(&(alto as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planos
    bmp.extend_from_slice(&24u16.to_le_bytes()); // bits por pixel
    bmp.extend_from_slice(&0u32.to_le_bytes()); // sin compresión
    bmp.extend_from_slice(&(tam_pixeles as u32).to_le_bytes());
    bmp.extend_from_slice(&2835i32.to_le_bytes()); // 72 puntos por pulgada, en puntos por metro
    bmp.extend_from_slice(&2835i32.to_le_bytes());
    bmp.extend_from_slice(&[0; 8]); // colores de paleta: ninguno

    let relleno = bytes_fila - ancho * 3;
    for fila in lienzo.pixeles().chunks(ancho.max(1)).rev() {
        for &pixel in fila {
            let [b, g, r, _] = pixel.to_le_bytes();
            bmp.extend_from_slice(&[b, g, r]);
        }
        bmp.extend(std::iter::repeat_n(0, relleno));
    }
    bmp
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn u32_en(bytes: &[u8], i: usize) -> u32 {
        u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap())
    }

    #[test]
    fn cabecera_y_pixeles_de_un_lienzo_de_3_por_2() {
        let mut lienzo = Lienzo::nuevo(3, 2, 0xFFFFFF);
        lienzo.rectangulo(0, 0, 1, 1, 0x112233); // arriba a la izquierda
        lienzo.rectangulo(2, 1, 1, 1, 0xAABBCC); // abajo a la derecha
        let bmp = codificar(&lienzo);

        // Cada fila: 3 pixeles × 3 bytes = 9, rellenada a 12.
        assert_eq!(bmp.len(), 54 + 12 * 2);
        assert_eq!(&bmp[0..2], b"BM");
        assert_eq!(u32_en(&bmp, 2), 78);
        assert_eq!(u32_en(&bmp, 10), 54);
        assert_eq!(u32_en(&bmp, 14), 40);
        assert_eq!(u32_en(&bmp, 18), 3);
        assert_eq!(u32_en(&bmp, 22), 2);
        assert_eq!(u16::from_le_bytes([bmp[26], bmp[27]]), 1);
        assert_eq!(u16::from_le_bytes([bmp[28], bmp[29]]), 24);
        assert_eq!(u32_en(&bmp, 34), 24);

        // Primero va la fila de abajo: blanco, blanco, AABBCC (en orden B, G, R), relleno.
        let abajo = &bmp[54..66];
        assert_eq!(abajo, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xCC, 0xBB, 0xAA, 0, 0, 0]);
        // Después la de arriba: 112233 primero.
        assert_eq!(&bmp[66..69], [0x33, 0x22, 0x11]);
    }

    #[test]
    fn sin_relleno_cuando_la_fila_ya_es_multiplo_de_4() {
        let bmp = codificar(&Lienzo::nuevo(4, 1, 0));
        assert_eq!(bmp.len(), 54 + 12);
    }
}
