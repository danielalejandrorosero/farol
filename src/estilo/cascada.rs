//! La cascada: la "C" de CSS. Cuando varias declaraciones le dicen cosas distintas a un mismo
//! elemento sobre la misma propiedad, decide quién gana.
//!
//! Cada declaración que le aplica al elemento recibe una prioridad, que se compara pieza por
//! pieza, como un número de versión:
//!
//! 1. `!important` le gana a lo normal.
//! 2. Origen: navegador < autor (los `<style>`) < atributo `style=""`.
//! 3. Especificidad del selector: (ids, clases, etiquetas).
//! 4. Orden de aparición: a igual todo lo demás, la que se escribió después.
//!
//! El resultado no es solo la ganadora: para cada propiedad se guardan todas las candidatas,
//! de la más fuerte a la más débil. Así, si la ganadora trae un valor que Farol no entiende
//! (`display: banana`), el cálculo pasa a la siguiente, como si la mala no existiera.

use std::collections::HashMap;
use std::fmt;

use super::emparejar::{atributo, coincide};
use crate::css::{self, Declaracion, HojaDeEstilos, Valor};
use crate::html::dom::{Documento, NodoId};

/// De dónde viene una declaración. El orden de las variantes es su fuerza.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origen {
    Navegador,
    Autor,
    /// El atributo `style=""` del propio elemento.
    EnLinea,
}

/// Lo que se compara para decidir quién gana. Rust compara las tuplas pieza por pieza, de
/// izquierda a derecha, igual que la lista de arriba.
pub type Prioridad = (bool, Origen, (u32, u32, u32), (usize, usize, usize));

/// Una declaración que le aplica al elemento, con todo lo que hace falta para ordenarla.
#[derive(Debug, Clone)]
pub struct Candidata {
    pub declaracion: Declaracion,
    pub prioridad: Prioridad,
    /// El selector que la trajo, para explicarlo (`p.nota`, o vacío si viene de `style=""`).
    pub selector: String,
}

impl Candidata {
    pub fn origen(&self) -> Origen {
        self.prioridad.1
    }
}

/// Para cada propiedad, sus candidatas de la más fuerte a la más débil.
pub struct Especificados {
    pub por_propiedad: HashMap<String, Vec<Candidata>>,
}

impl Especificados {
    /// Los valores de una propiedad, del más fuerte al más débil (vacío si nadie la menciona).
    pub fn valores(&self, propiedad: &str) -> impl Iterator<Item = &Valor> {
        self.por_propiedad.get(propiedad).into_iter().flatten().map(|c| &c.declaracion.valor)
    }
}

/// Junta todas las declaraciones que le aplican a un elemento y las ordena por prioridad.
/// `hojas` va en orden de aparición, cada una con su origen.
pub fn especificados(doc: &Documento, id: NodoId, hojas: &[(Origen, &HojaDeEstilos)]) -> Especificados {
    let mut candidatas = Vec::new();
    for (num_hoja, (origen, hoja)) in hojas.iter().enumerate() {
        for (num_regla, regla) in hoja.reglas.iter().enumerate() {
            // Si varios selectores de la lista (`h1, .titulo`) coinciden, cuenta el más específico.
            let Some(selector) = regla
                .selectores
                .iter()
                .filter(|s| coincide(doc, id, s))
                .max_by_key(|s| s.especificidad())
            else {
                continue;
            };
            for (num_decl, declaracion) in regla.declaraciones.iter().enumerate() {
                candidatas.push(Candidata {
                    declaracion: declaracion.clone(),
                    prioridad: (
                        declaracion.importante,
                        *origen,
                        selector.especificidad(),
                        (num_hoja, num_regla, num_decl),
                    ),
                    selector: selector.to_string(),
                });
            }
        }
    }
    if let Some(texto) = atributo(doc, id, "style") {
        for (num_decl, declaracion) in css::parsear_declaraciones(texto).into_iter().enumerate() {
            candidatas.push(Candidata {
                prioridad: (declaracion.importante, Origen::EnLinea, (0, 0, 0), (0, 0, num_decl)),
                declaracion,
                selector: String::new(),
            });
        }
    }

    // De la más fuerte a la más débil.
    candidatas.sort_by(|a, b| b.prioridad.cmp(&a.prioridad));
    let mut por_propiedad: HashMap<String, Vec<Candidata>> = HashMap::new();
    for candidata in candidatas {
        por_propiedad.entry(candidata.declaracion.propiedad.clone()).or_default().push(candidata);
    }
    Especificados { por_propiedad }
}

impl fmt::Display for Origen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Origen::Navegador => "navegador",
            Origen::Autor => "autor",
            Origen::EnLinea => "style=\"\"",
        })
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::html;

    fn ganadora(css_autor: &str, pagina: &str, propiedad: &str) -> Valor {
        let doc = html::parsear(pagina);
        let hoja = css::parsear(css_autor);
        // El primer elemento de la página es el que se examina.
        let id = doc.nodo(doc.raiz()).hijos[0];
        let esp = especificados(&doc, id, &[(Origen::Autor, &hoja)]);
        esp.valores(propiedad).next().cloned().expect("hay una ganadora")
    }

    fn color(r: u8, g: u8, b: u8) -> Valor {
        Valor::Color(css::Color::rgb(r, g, b))
    }

    #[test]
    fn la_mas_especifica_gana_aunque_venga_antes() {
        let css = "#x { color: red } p.a { color: blue } p { color: lime }";
        assert_eq!(ganadora(css, r#"<p id="x" class="a">t</p>"#, "color"), color(255, 0, 0));
        assert_eq!(ganadora(css, r#"<p class="a">t</p>"#, "color"), color(0, 0, 255));
    }

    #[test]
    fn a_igual_especificidad_gana_la_ultima() {
        let css = ".a { color: red } .b { color: blue } .a { color: lime }";
        assert_eq!(ganadora(css, r#"<p class="a b">t</p>"#, "color"), color(0, 255, 0));
        // Dentro de una misma regla, la declaración repetida de más abajo.
        assert_eq!(ganadora("p { color: red; color: blue }", "<p>t</p>", "color"), color(0, 0, 255));
    }

    #[test]
    fn el_atributo_style_le_gana_a_cualquier_selector() {
        let css = "#x.a.b { color: red }";
        let pagina = r#"<p id="x" class="a b" style="color: blue">t</p>"#;
        assert_eq!(ganadora(css, pagina, "color"), color(0, 0, 255));
    }

    #[test]
    fn important_le_gana_a_la_especificidad_y_al_style() {
        let css = "p { color: red !important } #x { color: blue }";
        let pagina = r#"<p id="x" style="color: lime">t</p>"#;
        assert_eq!(ganadora(css, pagina, "color"), color(255, 0, 0));
        // Pero un `!important` en style="" le gana a un `!important` del autor.
        let pagina = r#"<p id="x" style="color: lime !important">t</p>"#;
        assert_eq!(ganadora(css, pagina, "color"), color(0, 255, 0));
    }

    #[test]
    fn de_una_lista_de_selectores_cuenta_el_mas_especifico_que_coincide() {
        // `p, #x` contra `<p id=x>`: pesa (1,0,0) y le gana a `.a` aunque `.a` venga después.
        let css = "p, #x { color: red } .a { color: blue }";
        assert_eq!(ganadora(css, r#"<p id="x" class="a">t</p>"#, "color"), color(255, 0, 0));
    }

    #[test]
    fn el_autor_le_gana_al_navegador_aunque_sea_menos_especifico() {
        let doc = html::parsear(r#"<p id="x">t</p>"#);
        let navegador = css::parsear("p#x { color: red }");
        let autor = css::parsear("* { color: blue }");
        let id = doc.nodo(doc.raiz()).hijos[0];
        let esp = especificados(&doc, id, &[(Origen::Navegador, &navegador), (Origen::Autor, &autor)]);
        let valores: Vec<&Valor> = esp.valores("color").collect();
        assert_eq!(valores, [&color(0, 0, 255), &color(255, 0, 0)]);
    }
}
