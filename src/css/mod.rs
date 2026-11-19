//! CSS: recibe texto CSS y entrega una lista de reglas. Fase 3.
//!
//! Dos pasos, como en `html`: el tokenizador corta el texto en piezas y el parser arma las
//! reglas con ellas. Por el camino, los valores se interpretan (`10px`, `#fff`, `rgb(…)`) y las
//! propiedades abreviadas se expanden (`margin` → cuatro `margin-*`).
//!
//! El parser nunca falla (decisión 3.1, subconjunto "medio"): lo que no entiende lo salta, como
//! un navegador de verdad, y sigue con el resto.
//!
//! - `tokenizador.rs`: texto → tokens.
//! - `parser.rs`: tokens → reglas, con la recuperación de errores.
//! - `selectores.rs`: lo que va antes de `{`.
//! - `valores.rs`: lo que va después de `propiedad:`, incluidos los colores.
//! - `abreviadas.rs`: `margin`, `padding`, `border`, `background`.
//! - `tipos.rs`: lo que se entrega al motor de estilos.

mod abreviadas;
mod parser;
mod selectores;
pub mod tipos;
mod tokenizador;
mod valores;

// El motor de estilos (fase 4) usará estos tipos directamente.
#[allow(unused_imports)]
pub use tipos::{Color, Declaracion, HojaDeEstilos, Regla, Selector, SelectorSimple, Unidad, Valor};

/// Convierte el texto de una hoja de estilos en sus reglas.
pub fn parsear(css: &str) -> HojaDeEstilos {
    parsear_con_avisos(css).0
}

/// Igual que `parsear`, pero además devuelve una línea por cada cosa que se descartó y por qué.
/// Es lo que muestra `farol --css`.
pub fn parsear_con_avisos(css: &str) -> (HojaDeEstilos, Vec<String>) {
    parser::parsear_hoja(tokenizador::tokenizar(css))
}

/// Las declaraciones del atributo `style=""`: lo mismo que va entre llaves, sin selector.
pub fn parsear_declaraciones(texto: &str) -> Vec<Declaracion> {
    parser::parsear_lista_declaraciones(&tokenizador::tokenizar(texto), &mut Vec::new())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn atributo_style() {
        let d = parsear_declaraciones("color: #333; padding: 4px 8px; nada; font-weight: bold");
        let nombres: Vec<&str> = d.iter().map(|d| d.propiedad.as_str()).collect();
        assert_eq!(
            nombres,
            ["color", "padding-top", "padding-right", "padding-bottom", "padding-left", "font-weight"]
        );
        assert_eq!(d[0].valor, Valor::Color(Color::rgb(0x33, 0x33, 0x33)));
        assert_eq!(d[2].valor, Valor::Longitud(8.0, Unidad::Px));
    }

    #[test]
    fn atributo_style_con_llaves_sueltas() {
        // En un style="" una llave suelta no cierra nada: solo invalida su declaración.
        let d = parsear_declaraciones("color: red; } width: 5px; display: block");
        let nombres: Vec<&str> = d.iter().map(|d| d.propiedad.as_str()).collect();
        assert_eq!(nombres, ["color", "display"]);
    }

    #[test]
    fn hoja_de_ejemplo_de_la_carpeta_pruebas() {
        let (hoja, _) = parsear_con_avisos(include_str!("../../pruebas/03-estilos.css"));
        assert!(hoja.reglas.len() >= 10);
        let (hoja, avisos) = parsear_con_avisos(include_str!("../../pruebas/04-roto.css"));
        assert!(!hoja.reglas.is_empty());
        assert!(!avisos.is_empty());
    }

    /// Miles de textos al azar armados con pedazos de CSS: ninguno puede tumbar el parser.
    /// El azar es un generador fijo (siempre la misma secuencia) para que la prueba sea repetible.
    #[test]
    fn nunca_entra_en_panico_con_texto_al_azar() {
        const PEDAZOS: [&str; 32] = [
            "{", "}", ";", ":", ",", ".", "#", "*", ">", "(", ")", "[", "]", "@media", "!important",
            "/*", "*/", "\"", "'", "\\", "\n", " ", "p", "color", "margin", "border", "10px", "-",
            "rgb(", "url(", "#fff", "ñ",
        ];
        let mut semilla: u64 = 12345;
        for _ in 0..3000 {
            let mut css = String::new();
            for _ in 0..(semilla % 40) {
                semilla = semilla.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                css += PEDAZOS[(semilla >> 33) as usize % PEDAZOS.len()];
            }
            let _ = parsear_con_avisos(&css);
            let _ = parsear_declaraciones(&css);
        }
    }

    #[test]
    fn imprimir() {
        let hoja = parsear("div.caja#x, p { margin: 0 auto; color: rgba(0,0,0,.5) !important }");
        let texto = hoja.imprimir();
        assert!(texto.starts_with("Regla 1: div#x.caja (1,1,1), p (0,0,1)\n"));
        assert!(texto.contains("  margin-right: auto\n"));
        assert!(texto.contains("  color: rgba(0, 0, 0, 0.5) !important\n"));
    }
}
