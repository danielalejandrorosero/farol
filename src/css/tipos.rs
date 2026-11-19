//! Los tipos que entrega el parser de CSS: lo que el motor de estilos (fase 4) recibe.
//!
//! Una hoja es una lista de reglas; una regla son selectores más declaraciones:
//!
//! ```text
//! h1, .titulo { color: red; margin: 0 }
//! └─selectores┘ └──────declaraciones──────┘
//! ```

use std::fmt;

/// Un color en sus cuatro canales de 0 a 255. `a = 255` es opaco, `a = 0` es invisible.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unidad {
    Px,
    Em,
    Porcentaje,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    /// Una palabra clave (`auto`, `bold`, `solid`), en minúsculas. También guarda, tal como se
    /// escribió, cualquier valor de varias piezas que Farol no interpreta
    /// (`font-family: Arial, sans-serif`).
    Palabra(String),
    Longitud(f32, Unidad),
    /// Un número sin unidad: `line-height: 1.5`, `font-weight: 700`.
    Numero(f32),
    Color(Color),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaracion {
    /// En minúsculas, y siempre una propiedad "larga": las abreviadas ya vienen expandidas.
    pub propiedad: String,
    pub valor: Valor,
    pub importante: bool,
}

/// Un selector sin espacios: `div.caja#x` es etiqueta `div`, id `x` y clase `caja`.
/// `*` es un selector simple sin nada.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectorSimple {
    pub etiqueta: Option<String>,
    pub id: Option<String>,
    pub clases: Vec<String>,
}

/// Selectores simples unidos por el combinador descendiente, de izquierda a derecha:
/// `nav ul a` es `[nav, ul, a]`; el último es el elemento al que se aplica la regla.
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub cadena: Vec<SelectorSimple>,
}

impl Selector {
    /// Cuánto pesa el selector cuando dos reglas pelean por la misma propiedad:
    /// (ids, clases, etiquetas). Se comparan en ese orden, como versiones de software:
    /// `(1,0,0)` le gana a `(0,9,9)`.
    pub fn especificidad(&self) -> (u32, u32, u32) {
        let mut cuenta = (0, 0, 0);
        for simple in &self.cadena {
            cuenta.0 += simple.id.is_some() as u32;
            cuenta.1 += simple.clases.len() as u32;
            cuenta.2 += simple.etiqueta.is_some() as u32;
        }
        cuenta
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Regla {
    pub selectores: Vec<Selector>,
    pub declaraciones: Vec<Declaracion>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HojaDeEstilos {
    pub reglas: Vec<Regla>,
}

impl HojaDeEstilos {
    /// La hoja en texto legible, para `farol --css`:
    ///
    /// ```text
    /// Regla 1: h1 (0,0,1), .titulo (0,1,0)
    ///   color: #ff0000
    /// ```
    pub fn imprimir(&self) -> String {
        let mut salida = String::new();
        for (i, regla) in self.reglas.iter().enumerate() {
            let selectores: Vec<String> = regla
                .selectores
                .iter()
                .map(|s| {
                    let (a, b, c) = s.especificidad();
                    format!("{s} ({a},{b},{c})")
                })
                .collect();
            salida += &format!("Regla {}: {}\n", i + 1, selectores.join(", "));
            for declaracion in &regla.declaraciones {
                salida += &format!("  {declaracion}\n");
            }
            if regla.declaraciones.is_empty() {
                salida += "  (sin declaraciones)\n";
            }
        }
        salida
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.a == 255 {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            let alfa = (self.a as f32 / 255.0 * 100.0).round() / 100.0;
            write!(f, "rgba({}, {}, {}, {alfa})", self.r, self.g, self.b)
        }
    }
}

impl fmt::Display for Unidad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Unidad::Px => "px",
            Unidad::Em => "em",
            Unidad::Porcentaje => "%",
        })
    }
}

impl fmt::Display for Valor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Valor::Palabra(p) => write!(f, "{p}"),
            Valor::Longitud(n, u) => write!(f, "{n}{u}"),
            Valor::Numero(n) => write!(f, "{n}"),
            Valor::Color(c) => write!(f, "{c}"),
        }
    }
}

impl fmt::Display for Declaracion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.propiedad, self.valor)?;
        if self.importante {
            write!(f, " !important")?;
        }
        Ok(())
    }
}

impl fmt::Display for SelectorSimple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.etiqueta {
            Some(etiqueta) => write!(f, "{etiqueta}")?,
            None if self.id.is_none() && self.clases.is_empty() => write!(f, "*")?,
            None => {}
        }
        if let Some(id) = &self.id {
            write!(f, "#{id}")?;
        }
        for clase in &self.clases {
            write!(f, ".{clase}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let partes: Vec<String> = self.cadena.iter().map(ToString::to_string).collect();
        write!(f, "{}", partes.join(" "))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn simple(etiqueta: Option<&str>, id: Option<&str>, clases: &[&str]) -> SelectorSimple {
        SelectorSimple {
            etiqueta: etiqueta.map(String::from),
            id: id.map(String::from),
            clases: clases.iter().map(|c| c.to_string()).collect(),
        }
    }

    #[test]
    fn especificidad_suma_toda_la_cadena() {
        let s = Selector {
            cadena: vec![simple(Some("div"), Some("x"), &["a", "b"]), simple(Some("p"), None, &[])],
        };
        assert_eq!(s.especificidad(), (1, 2, 2));
        assert_eq!(Selector { cadena: vec![simple(None, None, &[])] }.especificidad(), (0, 0, 0));
    }

    #[test]
    fn colores_se_muestran_en_hex_o_rgba() {
        assert_eq!(Color::rgb(255, 0, 16).to_string(), "#ff0010");
        assert_eq!(Color { r: 0, g: 0, b: 0, a: 128 }.to_string(), "rgba(0, 0, 0, 0.5)");
    }

    #[test]
    fn selectores_se_muestran_como_en_css() {
        let s = Selector { cadena: vec![simple(None, None, &[]), simple(Some("p"), Some("x"), &["a"])] };
        assert_eq!(s.to_string(), "* p#x.a");
        assert_eq!(simple(None, None, &["c"]).to_string(), ".c");
    }
}
