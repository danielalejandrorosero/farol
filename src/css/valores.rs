//! Valores: convierte los tokens que van después de `propiedad:` en un `Valor`.
//!
//! Primero los tokens se agrupan en "piezas" (una función con sus argumentos es una sola pieza:
//! `rgb(1, 2, 3)`), y después cada pieza se interpreta: `10px` es una longitud, `red` un color,
//! `auto` una palabra.
//!
//! Hay una validación ligera según el tipo de propiedad: a `color` solo le sirve un color, y a
//! `margin-top` solo una longitud o una palabra. Así `color: 12px` se descarta y sigue valiendo
//! el `color` anterior, como en un navegador.

use super::tipos::{Color, Unidad, Valor};
use super::tokenizador::{Token, a_texto};

#[derive(Debug, Clone, PartialEq)]
pub enum Pieza {
    Simple(Token),
    Funcion { nombre: String, argumentos: Vec<Token> },
    Coma,
}

impl Pieza {
    fn a_texto(&self) -> String {
        match self {
            Pieza::Simple(t) => t.to_string(),
            Pieza::Funcion { nombre, argumentos } => format!("{nombre}({})", a_texto(argumentos)),
            Pieza::Coma => ",".to_string(),
        }
    }
}

/// Agrupa los tokens de un valor en piezas. Los espacios separan piezas y desaparecen.
pub fn piezas(tokens: &[Token]) -> Vec<Pieza> {
    let mut piezas = Vec::new();
    let mut i = 0;
    while let Some(token) = tokens.get(i) {
        i += 1;
        match token {
            Token::Espacio => {}
            Token::Simbolo(',') => piezas.push(Pieza::Coma),
            Token::Funcion(nombre) => {
                // Los argumentos llegan hasta el `)` que cierra esta función, contando las
                // funciones y paréntesis anidados. Si nunca cierra, llegan hasta el final.
                let mut profundidad = 1;
                let inicio = i;
                while let Some(t) = tokens.get(i) {
                    match t {
                        Token::Funcion(_) | Token::Simbolo('(') => profundidad += 1,
                        Token::Simbolo(')') => profundidad -= 1,
                        _ => {}
                    }
                    if profundidad == 0 {
                        break;
                    }
                    i += 1;
                }
                let argumentos = tokens[inicio..i.min(tokens.len())].to_vec();
                i += 1;
                piezas.push(Pieza::Funcion { nombre: nombre.clone(), argumentos });
            }
            otro => piezas.push(Pieza::Simple(otro.clone())),
        }
    }
    piezas
}

/// Une piezas en texto, tal como se escribirían: `Arial, sans-serif`.
pub fn piezas_a_texto(piezas: &[Pieza]) -> String {
    let mut texto = String::new();
    for (i, pieza) in piezas.iter().enumerate() {
        if i > 0 && *pieza != Pieza::Coma {
            texto.push(' ');
        }
        texto += &pieza.a_texto();
    }
    texto
}

/// Palabras que valen en cualquier propiedad de CSS.
pub const GLOBALES: [&str; 3] = ["inherit", "initial", "unset"];

pub const ESTILOS_DE_BORDE: [&str; 10] =
    ["none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset"];

pub fn es_global(valor: &Valor) -> bool {
    matches!(valor, Valor::Palabra(p) if GLOBALES.contains(&p.as_str()))
}

/// Qué acepta cada propiedad. Las que no están en ninguna lista aceptan cualquier valor.
#[derive(PartialEq)]
enum Tipo {
    Color,
    /// Longitud o palabra clave (`auto`, `none`, `medium`…).
    Longitud { negativa: bool },
    AnchoDeBorde,
    EstiloDeBorde,
    Libre,
}

fn tipo_de(propiedad: &str) -> Tipo {
    const LADOS: [&str; 4] = ["top", "right", "bottom", "left"];
    let de_borde = |sufijo: &str| {
        LADOS.iter().any(|lado| propiedad == format!("border-{lado}-{sufijo}"))
    };
    if matches!(propiedad, "color" | "background-color" | "outline-color" | "text-decoration-color")
        || de_borde("color")
    {
        return Tipo::Color;
    }
    if de_borde("width") {
        return Tipo::AnchoDeBorde;
    }
    if de_borde("style") {
        return Tipo::EstiloDeBorde;
    }
    let sin_negativos = [
        "width", "height", "min-width", "max-width", "min-height", "max-height", "font-size",
        "padding-top", "padding-right", "padding-bottom", "padding-left",
    ];
    if sin_negativos.contains(&propiedad) {
        return Tipo::Longitud { negativa: false };
    }
    let con_negativos = [
        "margin-top", "margin-right", "margin-bottom", "margin-left", "top", "right", "bottom",
        "left", "text-indent", "letter-spacing", "word-spacing",
    ];
    if con_negativos.contains(&propiedad) {
        return Tipo::Longitud { negativa: true };
    }
    Tipo::Libre
}

/// El valor de una propiedad larga a partir de sus piezas, o `None` si no es válido para ella.
pub fn valor_de(propiedad: &str, piezas: &[Pieza]) -> Option<Valor> {
    let tipo = tipo_de(propiedad);
    let valor = match piezas {
        [pieza] => valor_simple(pieza),
        _ => None,
    };
    let valor = match valor {
        Some(valor) => valor,
        // Lo que Farol no sabe interpretar se guarda como texto en las propiedades sin tipo;
        // en las que tienen tipo es un error.
        None if tipo == Tipo::Libre && !piezas.is_empty() => {
            return Some(Valor::Palabra(piezas_a_texto(piezas)));
        }
        None => return None,
    };
    if es_global(&valor) {
        return Some(valor);
    }
    match tipo {
        Tipo::Libre => Some(valor),
        Tipo::Color => match valor {
            Valor::Color(_) => Some(valor),
            Valor::Palabra(ref p) if p == "currentcolor" => Some(valor),
            _ => None,
        },
        Tipo::EstiloDeBorde => match valor {
            Valor::Palabra(ref p) if ESTILOS_DE_BORDE.contains(&p.as_str()) => Some(valor),
            _ => None,
        },
        Tipo::AnchoDeBorde => match valor {
            Valor::Palabra(ref p) if ["thin", "medium", "thick"].contains(&p.as_str()) => Some(valor),
            Valor::Longitud(n, Unidad::Px | Unidad::Em) if n >= 0.0 => Some(valor),
            Valor::Numero(n) if n == 0.0 => Some(Valor::Longitud(0.0, Unidad::Px)),
            _ => None,
        },
        Tipo::Longitud { negativa } => match valor {
            Valor::Palabra(_) => Some(valor),
            Valor::Longitud(n, _) if negativa || n >= 0.0 => Some(valor),
            // `0` es la única longitud que se puede escribir sin unidad.
            Valor::Numero(n) if n == 0.0 => Some(Valor::Longitud(0.0, Unidad::Px)),
            _ => None,
        },
    }
}

/// Interpreta una sola pieza, sin saber de qué propiedad es.
pub fn valor_simple(pieza: &Pieza) -> Option<Valor> {
    match pieza {
        Pieza::Simple(Token::Identificador(nombre)) => {
            let nombre = nombre.to_ascii_lowercase();
            Some(match color_por_nombre(&nombre) {
                Some(color) => Valor::Color(color),
                None => Valor::Palabra(nombre),
            })
        }
        Pieza::Simple(Token::Numero(n)) if n.is_finite() => Some(Valor::Numero(*n)),
        Pieza::Simple(Token::Porcentaje(n)) if n.is_finite() => {
            Some(Valor::Longitud(*n, Unidad::Porcentaje))
        }
        Pieza::Simple(Token::Dimension(n, unidad)) if n.is_finite() => longitud(*n, unidad),
        Pieza::Simple(Token::Hash(hex)) => color_hex(hex).map(Valor::Color),
        Pieza::Funcion { nombre, argumentos } if nombre == "rgb" || nombre == "rgba" => {
            color_rgb(argumentos).map(Valor::Color)
        }
        _ => None,
    }
}

/// `px` y `em` pasan tal cual. Las unidades absolutas (`pt`, `cm`…) se convierten a px aquí
/// mismo, porque su equivalencia es fija: 1in = 96px. Las relativas que Farol aún no conoce
/// (`rem`, `vh`, `vw`…) no son válidas.
fn longitud(n: f32, unidad: &str) -> Option<Valor> {
    let px = |factor: f32| Some(Valor::Longitud(n * factor, Unidad::Px));
    match unidad {
        "px" => px(1.0),
        "em" => Some(Valor::Longitud(n, Unidad::Em)),
        "pt" => px(96.0 / 72.0),
        "pc" => px(16.0),
        "in" => px(96.0),
        "cm" => px(96.0 / 2.54),
        "mm" => px(96.0 / 25.4),
        _ => None,
    }
}

/// `#rgb`, `#rgba`, `#rrggbb` o `#rrggbbaa`.
pub fn color_hex(hex: &str) -> Option<Color> {
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let digitos: Vec<u8> = hex.chars().map(|c| c.to_digit(16).unwrap_or(0) as u8).collect();
    let (r, g, b, a) = match digitos[..] {
        // En la forma corta cada dígito se repite: `#f80` es `#ff8800`.
        [r, g, b] => (r * 17, g * 17, b * 17, 255),
        [r, g, b, a] => (r * 17, g * 17, b * 17, a * 17),
        [r1, r2, g1, g2, b1, b2] => (r1 * 16 + r2, g1 * 16 + g2, b1 * 16 + b2, 255),
        [r1, r2, g1, g2, b1, b2, a1, a2] => (r1 * 16 + r2, g1 * 16 + g2, b1 * 16 + b2, a1 * 16 + a2),
        _ => return None,
    };
    Some(Color { r, g, b, a })
}

/// `rgb(255, 0, 0)`, `rgba(0, 0, 0, 0.5)`, `rgb(100% 0% 0% / 50%)`. Acepta comas o espacios,
/// números o porcentajes, y alfa opcional en las dos funciones.
fn color_rgb(argumentos: &[Token]) -> Option<Color> {
    let mut numeros = Vec::new();
    for token in argumentos {
        match token {
            Token::Espacio | Token::Simbolo(',') | Token::Simbolo('/') => {}
            Token::Numero(n) if n.is_finite() => numeros.push((*n, false)),
            Token::Porcentaje(n) if n.is_finite() => numeros.push((*n, true)),
            _ => return None,
        }
    }
    let canal = |(n, porcentaje): (f32, bool)| {
        let n = if porcentaje { n * 2.55 } else { n };
        n.round().clamp(0.0, 255.0) as u8
    };
    let alfa = |(n, porcentaje): (f32, bool)| {
        let n = if porcentaje { n / 100.0 } else { n };
        (n.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    match numeros[..] {
        [r, g, b] => Some(Color { r: canal(r), g: canal(g), b: canal(b), a: 255 }),
        [r, g, b, a] => Some(Color { r: canal(r), g: canal(g), b: canal(b), a: alfa(a) }),
        _ => None,
    }
}

/// Los 16 colores básicos de CSS, `transparent` y algunos comunes más. Recibe el nombre en
/// minúsculas.
pub fn color_por_nombre(nombre: &str) -> Option<Color> {
    let (r, g, b) = match nombre {
        "transparent" => return Some(Color { r: 0, g: 0, b: 0, a: 0 }),
        // Los 16 básicos.
        "black" => (0, 0, 0),
        "silver" => (192, 192, 192),
        "gray" | "grey" => (128, 128, 128),
        "white" => (255, 255, 255),
        "maroon" => (128, 0, 0),
        "red" => (255, 0, 0),
        "purple" => (128, 0, 128),
        "fuchsia" | "magenta" => (255, 0, 255),
        "green" => (0, 128, 0),
        "lime" => (0, 255, 0),
        "olive" => (128, 128, 0),
        "yellow" => (255, 255, 0),
        "navy" => (0, 0, 128),
        "blue" => (0, 0, 255),
        "teal" => (0, 128, 128),
        "aqua" | "cyan" => (0, 255, 255),
        // Comunes en páginas reales.
        "orange" => (255, 165, 0),
        "pink" => (255, 192, 203),
        "brown" => (165, 42, 42),
        "gold" => (255, 215, 0),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "dimgray" | "dimgrey" => (105, 105, 105),
        "whitesmoke" => (245, 245, 245),
        "gainsboro" => (220, 220, 220),
        "darkblue" => (0, 0, 139),
        "darkred" => (139, 0, 0),
        "darkgreen" => (0, 100, 0),
        "lightblue" => (173, 216, 230),
        "skyblue" => (135, 206, 235),
        "steelblue" => (70, 130, 180),
        "indigo" => (75, 0, 130),
        "violet" => (238, 130, 238),
        "crimson" => (220, 20, 60),
        "tomato" => (255, 99, 71),
        "coral" => (255, 127, 80),
        "salmon" => (250, 128, 114),
        "beige" => (245, 245, 220),
        "ivory" => (255, 255, 240),
        "khaki" => (240, 230, 140),
        "tan" => (210, 180, 140),
        "chocolate" => (210, 105, 30),
        "turquoise" => (64, 224, 208),
        "lavender" => (230, 230, 250),
        "rebeccapurple" => (102, 51, 153),
        _ => return None,
    };
    Some(Color::rgb(r, g, b))
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::css::tokenizador::tokenizar;

    fn valor(propiedad: &str, texto: &str) -> Option<Valor> {
        valor_de(propiedad, &piezas(&tokenizar(texto)))
    }

    fn color(texto: &str) -> Option<Color> {
        match valor("color", texto) {
            Some(Valor::Color(c)) => Some(c),
            _ => None,
        }
    }

    #[test]
    fn longitudes() {
        assert_eq!(valor("width", "10px"), Some(Valor::Longitud(10.0, Unidad::Px)));
        assert_eq!(valor("width", "1.5EM"), Some(Valor::Longitud(1.5, Unidad::Em)));
        assert_eq!(valor("width", "50%"), Some(Valor::Longitud(50.0, Unidad::Porcentaje)));
        assert_eq!(valor("width", "0"), Some(Valor::Longitud(0.0, Unidad::Px)));
        assert_eq!(valor("width", "12pt"), Some(Valor::Longitud(16.0, Unidad::Px)));
        assert_eq!(valor("width", "1in"), Some(Valor::Longitud(96.0, Unidad::Px)));
        assert_eq!(valor("width", "auto"), Some(Valor::Palabra("auto".into())));
        assert_eq!(valor("margin-left", "-5px"), Some(Valor::Longitud(-5.0, Unidad::Px)));
    }

    #[test]
    fn longitudes_invalidas() {
        assert_eq!(valor("width", "10"), None, "sin unidad solo vale el 0");
        assert_eq!(valor("width", "2rem"), None, "rem aún no se soporta");
        assert_eq!(valor("width", "-5px"), None, "width no acepta negativos");
        assert_eq!(valor("width", "red"), None);
        assert_eq!(valor("width", "10px 20px"), None);
        assert_eq!(valor("width", "\"hola\""), None);
    }

    #[test]
    fn numeros_sin_unidad() {
        assert_eq!(valor("line-height", "1.5"), Some(Valor::Numero(1.5)));
        assert_eq!(valor("font-weight", "700"), Some(Valor::Numero(700.0)));
        assert_eq!(valor("line-height", "0"), Some(Valor::Numero(0.0)));
    }

    #[test]
    fn colores_por_nombre() {
        assert_eq!(color("red"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(color("RED"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(color("grey"), color("gray"));
        assert_eq!(color("orange"), Some(Color::rgb(255, 165, 0)));
        assert_eq!(color("transparent"), Some(Color { r: 0, g: 0, b: 0, a: 0 }));
        assert_eq!(color("banana"), None);
    }

    #[test]
    fn colores_hex() {
        assert_eq!(color("#fff"), Some(Color::rgb(255, 255, 255)));
        assert_eq!(color("#F80"), Some(Color::rgb(255, 136, 0)));
        assert_eq!(color("#0f6e6e"), Some(Color::rgb(15, 110, 110)));
        assert_eq!(color("#00000080"), Some(Color { r: 0, g: 0, b: 0, a: 128 }));
        assert_eq!(color("#ff"), None);
        assert_eq!(color("#ggg"), None);
    }

    #[test]
    fn colores_rgb() {
        assert_eq!(color("rgb(255, 0, 10)"), Some(Color::rgb(255, 0, 10)));
        assert_eq!(color("rgb(300, -4, 0)"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(color("rgba(0, 0, 0, 0.5)"), Some(Color { r: 0, g: 0, b: 0, a: 128 }));
        assert_eq!(color("rgb(100% 0% 0% / 50%)"), Some(Color { r: 255, g: 0, b: 0, a: 128 }));
        assert_eq!(color("RGB(1,2,3)"), Some(Color::rgb(1, 2, 3)));
        assert_eq!(color("rgb(1, 2)"), None);
        assert_eq!(color("rgb(a, b, c)"), None);
        assert_eq!(color("rgb(1, 2, 3"), Some(Color::rgb(1, 2, 3)), "sin cerrar al final vale");
    }

    #[test]
    fn propiedades_de_color_solo_aceptan_colores() {
        assert_eq!(valor("color", "12px"), None);
        assert_eq!(valor("color", "currentColor"), Some(Valor::Palabra("currentcolor".into())));
        assert_eq!(valor("color", "inherit"), Some(Valor::Palabra("inherit".into())));
    }

    #[test]
    fn propiedades_libres_guardan_el_texto() {
        assert_eq!(
            valor("font-family", "\"Helvetica Neue\", Arial,sans-serif"),
            Some(Valor::Palabra("\"Helvetica Neue\", Arial, sans-serif".into()))
        );
        assert_eq!(valor("display", "BLOCK"), Some(Valor::Palabra("block".into())));
        assert_eq!(valor("transition", "2s"), Some(Valor::Palabra("2s".into())));
        assert_eq!(valor("display", ""), None);
    }
}
