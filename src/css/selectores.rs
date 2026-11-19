//! Selectores: la parte de la regla antes de `{`, que dice a qué elementos se aplica.
//!
//! Farol entiende etiqueta (`p`), clase (`.caja`), id (`#x`), universal (`*`), sus
//! combinaciones pegadas (`div.caja#x`) y el espacio como combinador descendiente (`nav a`).
//!
//! Cualquier otra cosa (`>`, `+`, `~`, `:hover`, `[href]`, `::before`) vuelve inválida la lista
//! entera, y la regla se descarta. Así lo manda CSS: si un navegador no entiende uno de los
//! selectores de `a, b:raro { }`, tampoco aplica la regla a `a`.

use super::tipos::{Selector, SelectorSimple};
use super::tokenizador::Token;

/// La lista de selectores separados por coma, o `None` si alguno no se entiende.
pub fn parsear_lista(tokens: &[Token]) -> Option<Vec<Selector>> {
    tokens.split(|t| *t == Token::Simbolo(',')).map(parsear_selector).collect()
}

fn parsear_selector(tokens: &[Token]) -> Option<Selector> {
    let tokens = recortar_espacios(tokens);
    let mut cadena = Vec::new();
    let mut actual = SelectorSimple::default();
    // ¿El selector simple que se está armando ya tiene algo? Una etiqueta solo puede ir primero.
    let mut empezado = false;
    let mut i = 0;
    while let Some(token) = tokens.get(i) {
        match token {
            // El espacio cierra un selector simple y empieza el siguiente.
            Token::Espacio => {
                cadena.push(std::mem::take(&mut actual));
                empezado = false;
            }
            Token::Identificador(nombre) if !empezado => {
                actual.etiqueta = Some(nombre.to_ascii_lowercase());
                empezado = true;
            }
            Token::Simbolo('*') if !empezado => empezado = true,
            // `#a#b` no puede coincidir con nada (un elemento tiene un solo id): se descarta.
            Token::Hash(id) if actual.id.is_none() && es_nombre_valido(id) => {
                actual.id = Some(id.clone());
                empezado = true;
            }
            Token::Simbolo('.') => {
                i += 1;
                let Some(Token::Identificador(clase)) = tokens.get(i) else {
                    return None;
                };
                actual.clases.push(clase.clone());
                empezado = true;
            }
            _ => return None,
        }
        i += 1;
    }
    if !empezado {
        // Lista vacía (`h1, , h2`) o selector que termina raro.
        return None;
    }
    cadena.push(actual);
    Some(Selector { cadena })
}

/// `#fff` es un hash válido como color, pero no como id: un id no empieza con dígito.
fn es_nombre_valido(nombre: &str) -> bool {
    let mut letras = nombre.trim_start_matches('-').chars();
    letras.next().is_some_and(|c| !c.is_ascii_digit()) && nombre != "-"
}

pub fn recortar_espacios(tokens: &[Token]) -> &[Token] {
    let inicio = tokens.iter().position(|t| *t != Token::Espacio).unwrap_or(tokens.len());
    let fin = tokens.iter().rposition(|t| *t != Token::Espacio).map_or(inicio, |p| p + 1);
    &tokens[inicio..fin]
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::css::tokenizador::tokenizar;

    fn lista(texto: &str) -> Option<Vec<Selector>> {
        parsear_lista(&tokenizar(texto))
    }

    fn uno(texto: &str) -> Selector {
        let mut l = lista(texto).expect("selector válido");
        assert_eq!(l.len(), 1);
        l.remove(0)
    }

    #[test]
    fn etiqueta_clase_id_y_universal() {
        let s = uno("DIV.caja.Grande#x");
        assert_eq!(s.cadena.len(), 1);
        assert_eq!(s.cadena[0].etiqueta.as_deref(), Some("div"));
        assert_eq!(s.cadena[0].id.as_deref(), Some("x"));
        assert_eq!(s.cadena[0].clases, ["caja", "Grande"]);
        assert_eq!(uno("*").cadena[0], SelectorSimple::default());
        assert_eq!(uno("*.a").cadena[0].clases, ["a"]);
        assert_eq!(uno("#principal").to_string(), "#principal");
    }

    #[test]
    fn combinador_descendiente() {
        let s = uno("  nav   ul\n a.activo ");
        assert_eq!(s.to_string(), "nav ul a.activo");
        assert_eq!(s.cadena.len(), 3);
    }

    #[test]
    fn lista_separada_por_comas() {
        let l = lista("h1, h2 ,.titulo").unwrap();
        assert_eq!(l.len(), 3);
        assert_eq!(l[2].to_string(), ".titulo");
    }

    #[test]
    fn especificidad() {
        assert_eq!(uno("p").especificidad(), (0, 0, 1));
        assert_eq!(uno(".a").especificidad(), (0, 1, 0));
        assert_eq!(uno("#x").especificidad(), (1, 0, 0));
        assert_eq!(uno("*").especificidad(), (0, 0, 0));
        assert_eq!(uno("div.caja#x").especificidad(), (1, 1, 1));
        assert_eq!(uno("ul li.item a").especificidad(), (0, 1, 3));
        assert_eq!(uno("#a .b .c p").especificidad(), (1, 2, 1));
    }

    #[test]
    fn selectores_no_soportados() {
        for texto in [
            "ul > li", "h1 + p", "h1 ~ p", "a:hover", "a::before", "input[type=text]", "p:not(.x)",
            "div>p", ".", "#123", "a#b#c", "div p,", ", p", "h1,,h2", "p *x", "", "p.", "a b:hover",
        ] {
            assert_eq!(lista(texto), None, "debería rechazar {texto:?}");
        }
    }

    #[test]
    fn un_selector_malo_tumba_la_lista() {
        assert_eq!(lista("p, a:hover, h1"), None);
    }
}
