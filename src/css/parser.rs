//! El parser de reglas: toma los tokens y arma la hoja de estilos.
//!
//! La idea central es la recuperación de errores. CSS se diseñó para que un navegador viejo
//! pudiera leer CSS nuevo: lo que no entiende lo salta, y sigue con lo demás. Los límites que
//! se respetan al saltar son:
//!
//! - Una declaración rota se salta hasta el siguiente `;` (o el `}` que cierra la regla).
//! - Una regla con selector raro se salta completa, hasta su `}` balanceado: se cuentan las
//!   llaves que abren y cierran, como quien cuenta paréntesis en una fórmula de Excel.
//! - Una regla `@` (`@media`, `@import`) se salta hasta su `;` o hasta el final de su bloque.
//! - El final del archivo cierra todo lo que quedó abierto.
//!
//! Lo que se descarta queda anotado como aviso para `farol --css`.

use super::abreviadas;
use super::selectores::{parsear_lista, recortar_espacios};
use super::tipos::{Declaracion, HojaDeEstilos, Regla};
use super::tokenizador::{Token, a_texto};
use super::valores::{piezas, valor_de};

pub fn parsear_hoja(tokens: Vec<Token>) -> (HojaDeEstilos, Vec<String>) {
    let mut parser = Parser { tokens, pos: 0, hoja: HojaDeEstilos::default(), avisos: Vec::new() };
    parser.correr();
    (parser.hoja, parser.avisos)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    hoja: HojaDeEstilos,
    avisos: Vec<String>,
}

impl Parser {
    fn siguiente(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        token
    }

    fn correr(&mut self) {
        while let Some(token) = self.tokens.get(self.pos) {
            match token {
                Token::Espacio => self.pos += 1,
                Token::Arroba(nombre) => {
                    let nombre = nombre.clone();
                    self.pos += 1;
                    self.saltar_regla_arroba();
                    self.avisos.push(format!("Se saltó la regla @{nombre} (Farol no la soporta)"));
                }
                _ => self.regla(),
            }
        }
    }

    /// Salta una regla `@` entera: hasta un `;` (como `@import "x.css";`) o hasta el final de su
    /// bloque (como `@media print { ... }`), lo que llegue primero.
    fn saltar_regla_arroba(&mut self) {
        while let Some(token) = self.siguiente() {
            match token {
                Token::Simbolo(';') => return,
                Token::Simbolo('{') => {
                    self.bloque();
                    return;
                }
                _ => {}
            }
        }
    }

    /// Lee el contenido de un bloque `{ ... }` cuyo `{` ya se consumió, hasta su `}` balanceado.
    /// Devuelve los tokens de adentro y si el bloque llegó a cerrarse.
    fn bloque(&mut self) -> (Vec<Token>, bool) {
        let mut contenido = Vec::new();
        let mut profundidad = 1;
        while let Some(token) = self.siguiente() {
            match token {
                Token::Simbolo('{') => profundidad += 1,
                Token::Simbolo('}') => {
                    profundidad -= 1;
                    if profundidad == 0 {
                        return (contenido, true);
                    }
                }
                _ => {}
            }
            contenido.push(token);
        }
        (contenido, false)
    }

    fn regla(&mut self) {
        let mut preludio = Vec::new();
        loop {
            match self.siguiente() {
                None => {
                    let texto = a_texto(&preludio);
                    self.avisos.push(format!("Se descartó `{texto}`: el archivo terminó antes de su `{{`"));
                    return;
                }
                Some(Token::Simbolo('{')) => break,
                Some(token) => preludio.push(token),
            }
        }
        let (contenido, cerrado) = self.bloque();
        let texto = a_texto(&preludio);
        if !cerrado {
            self.avisos.push(format!("La regla `{texto}` no cerró su `}}`: el final del archivo la cerró"));
        }
        let Some(selectores) = parsear_lista(&preludio) else {
            self.avisos.push(format!("Se descartó la regla `{texto}`: selector no soportado"));
            return;
        };
        let declaraciones = parsear_lista_declaraciones(&contenido, &mut self.avisos);
        self.hoja.reglas.push(Regla { selectores, declaraciones });
    }
}

/// Parte el contenido de un bloque en declaraciones por los `;` que no estén dentro de
/// paréntesis, corchetes o llaves, y parsea cada una. Las rotas se anotan y se saltan.
pub fn parsear_lista_declaraciones(tokens: &[Token], avisos: &mut Vec<String>) -> Vec<Declaracion> {
    let mut pedazos = Vec::new();
    let mut profundidad = 0usize;
    let mut inicio = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Funcion(_) | Token::Simbolo('(' | '[' | '{') => profundidad += 1,
            Token::Simbolo(')' | ']' | '}') => profundidad = profundidad.saturating_sub(1),
            Token::Simbolo(';') if profundidad == 0 => {
                pedazos.push(&tokens[inicio..i]);
                inicio = i + 1;
            }
            _ => {}
        }
    }
    pedazos.push(&tokens[inicio..]);

    let mut declaraciones = Vec::new();
    for pedazo in pedazos {
        let pedazo = recortar_espacios(pedazo);
        if pedazo.is_empty() {
            continue;
        }
        match declaracion(pedazo) {
            Ok(lista) => declaraciones.extend(lista),
            Err(motivo) => avisos.push(format!("Se descartó `{}`: {motivo}", a_texto(pedazo))),
        }
    }
    declaraciones
}

/// Una declaración `propiedad: valor [!important]`. Si es abreviada, devuelve varias.
fn declaracion(tokens: &[Token]) -> Result<Vec<Declaracion>, &'static str> {
    let Some(Token::Identificador(nombre)) = tokens.first() else {
        return Err("no empieza con un nombre de propiedad");
    };
    let propiedad = nombre.to_ascii_lowercase();
    let resto = recortar_espacios(&tokens[1..]);
    let Some((Token::Simbolo(':'), valor)) = resto.split_first() else {
        return Err("falta el `:` después de la propiedad");
    };
    let mut valor = recortar_espacios(valor);
    let mut importante = false;
    if let Some((Token::Importante, antes)) = valor.split_last() {
        importante = true;
        valor = recortar_espacios(antes);
    }
    if valor.is_empty() {
        return Err("valor vacío");
    }
    if valor.contains(&Token::Importante) {
        return Err("`!important` solo puede ir al final");
    }
    if valor.contains(&Token::CadenaRota) {
        return Err("una cadena no se cerró antes del salto de línea");
    }
    if valor.iter().any(|t| matches!(t, Token::Simbolo('{' | '}' | ';' | '!'))) {
        return Err("el valor tiene símbolos que no van ahí");
    }

    let piezas = piezas(valor);
    let pares = match abreviadas::expandir(&propiedad, &piezas) {
        Some(Some(pares)) => pares,
        Some(None) => return Err("valor no válido para esta abreviada"),
        None => match valor_de(&propiedad, &piezas) {
            Some(v) => vec![(propiedad, v)],
            None => return Err("valor no válido para esta propiedad"),
        },
    };
    Ok(pares
        .into_iter()
        .map(|(propiedad, valor)| Declaracion { propiedad, valor, importante })
        .collect())
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::css::tipos::{Color, Unidad, Valor};
    use crate::css::tokenizador::tokenizar;

    fn hoja(css: &str) -> HojaDeEstilos {
        parsear_hoja(tokenizar(css)).0
    }

    fn declaraciones(texto: &str) -> Vec<Declaracion> {
        parsear_lista_declaraciones(&tokenizar(texto), &mut Vec::new())
    }

    fn propiedades(texto: &str) -> Vec<String> {
        declaraciones(texto).into_iter().map(|d| d.propiedad).collect()
    }

    #[test]
    fn regla_completa() {
        let h = hoja("h1, .titulo { color: red; margin-top: 10px !important }");
        assert_eq!(h.reglas.len(), 1);
        let r = &h.reglas[0];
        assert_eq!(r.selectores.len(), 2);
        assert_eq!(r.declaraciones[0].valor, Valor::Color(Color::rgb(255, 0, 0)));
        assert!(!r.declaraciones[0].importante);
        assert_eq!(r.declaraciones[1].valor, Valor::Longitud(10.0, Unidad::Px));
        assert!(r.declaraciones[1].importante);
    }

    #[test]
    fn importante_se_propaga_a_la_expansion() {
        let d = declaraciones("margin: 0 ! important");
        assert_eq!(d.len(), 4);
        assert!(d.iter().all(|d| d.importante));
    }

    #[test]
    fn declaracion_rota_no_se_lleva_a_las_demas() {
        assert_eq!(
            propiedades("color: red; nada aqui; width 10px; : 5px; height: 2px; 12: x; top: 1px"),
            ["color", "height", "top"]
        );
        assert_eq!(propiedades("color: banana; display: block"), ["display"]);
        assert_eq!(propiedades("color:; display: block;;;"), ["display"]);
        assert_eq!(propiedades("color: red !important x; display: block"), ["display"]);
        assert_eq!(propiedades("color: 'abc\n; display: block"), ["display"]);
    }

    #[test]
    fn el_punto_y_coma_dentro_de_parentesis_no_corta() {
        let d = declaraciones("background-image: url(data:x;y); color: rgb(1;2;3); display: block");
        assert_eq!(d[0].propiedad, "background-image");
        assert_eq!(d[1].propiedad, "display");
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn propiedades_en_minusculas() {
        assert_eq!(propiedades("COLOR: Red"), ["color"]);
    }

    #[test]
    fn varias_reglas_y_comentarios() {
        let h = hoja("/* hola */ a{color:red} /* x */ p { } div p{display:none}");
        assert_eq!(h.reglas.len(), 3);
        assert!(h.reglas[1].declaraciones.is_empty());
        assert_eq!(h.reglas[2].selectores[0].cadena.len(), 2);
    }

    #[test]
    fn selector_no_soportado_salta_solo_su_regla() {
        let h = hoja("a:hover { color: red; } ul > li { x: y } p { color: blue } a[href] { b: c }");
        assert_eq!(h.reglas.len(), 1);
        assert_eq!(h.reglas[0].selectores[0].to_string(), "p");
    }

    #[test]
    fn llaves_anidadas_en_regla_descartada() {
        let h = hoja("a:hover { x { y { } } z: w } p { color: blue }");
        assert_eq!(h.reglas.len(), 1);
        assert_eq!(h.reglas[0].selectores[0].to_string(), "p");
    }

    #[test]
    fn reglas_arroba_se_saltan() {
        let (h, avisos) = parsear_hoja(tokenizar(
            "@import url(x.css); @charset \"utf-8\"; \
             @media (max-width: 600px) { p { color: red } .x { y: z } } \
             @font-face { font-family: X; } h1 { color: blue }",
        ));
        assert_eq!(h.reglas.len(), 1);
        assert_eq!(h.reglas[0].selectores[0].to_string(), "h1");
        assert_eq!(avisos.len(), 4);
    }

    #[test]
    fn final_del_archivo_cierra_la_regla() {
        let (h, avisos) = parsear_hoja(tokenizar("p { color: red; margin: 0"));
        assert_eq!(h.reglas.len(), 1);
        assert_eq!(h.reglas[0].declaraciones.len(), 5);
        assert_eq!(avisos.len(), 1);
    }

    #[test]
    fn regla_sin_bloque_al_final() {
        let (h, avisos) = parsear_hoja(tokenizar("p { color: red } h1"));
        assert_eq!(h.reglas.len(), 1);
        assert_eq!(avisos.len(), 1);
    }

    #[test]
    fn llave_de_cierre_suelta() {
        // Un `}` suelto entra al preludio de la regla siguiente y la invalida; la de después
        // sobrevive.
        let h = hoja("p { color: red } } h1 { color: blue } h2 { color: green }");
        let nombres: Vec<String> = h.reglas.iter().map(|r| r.selectores[0].to_string()).collect();
        assert_eq!(nombres, ["p", "h2"]);
    }

    #[test]
    fn nunca_entra_en_panico() {
        let basura = [
            "", "{", "}", "{{{{", "}}}}", ";;;", "p {", "p { color", "p { color:", "p{:}", "@", "@media",
            "@media {", "a,{}", "*{*:*}", "p{color:rgb(}", "p{color:rgb(1,2,3,4,5)}", "#{}", ".{}",
            "p { margin: 1px 2px 3px 4px 5px 6px }", "p{border:}", "p{background:,}", "/*", "'",
            "p{width:99999999999999999999999999999999999999999999px}", "\u{0}{\u{0}:\u{0}}", "!{!:!}",
            "p{x:y}}}}}{{{{", "url(", "p{a:url(}",
        ];
        for css in basura {
            let _ = parsear_hoja(tokenizar(css));
        }
    }

    #[test]
    fn numero_gigante_se_descarta() {
        assert_eq!(propiedades("width: 99999999999999999999999999999999999999999999999px"), Vec::<String>::new());
    }
}
