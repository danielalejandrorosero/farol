//! Emparejamiento: ¿este selector le aplica a este elemento?
//!
//! Decisión 4.1 (ingenuo): cada regla se prueba contra cada elemento, sin índices ni cachés.
//!
//! Un selector con descendientes se lee **de derecha a izquierda**. Para `nav ul a` contra un
//! `<a>`: primero se mira si el elemento es un `a` (la mayoría de las veces no lo es y se termina
//! ahí, barato); solo si lo es se sube por los padres buscando un `ul`, y desde ese `ul` se sigue
//! subiendo buscando un `nav`. Leído de izquierda a derecha habría que bajar por todos los
//! descendientes de cada `nav` de la página, que son muchísimos más caminos.

use crate::css::{Selector, SelectorSimple};
use crate::html::dom::{Documento, NodoId, TipoNodo};

/// El valor de un atributo de un elemento (los nombres llegan en minúsculas desde el parser).
pub fn atributo<'d>(doc: &'d Documento, id: NodoId, nombre: &str) -> Option<&'d str> {
    match &doc.nodo(id).tipo {
        TipoNodo::Elemento { atributos, .. } => {
            atributos.iter().find(|(clave, _)| clave == nombre).map(|(_, valor)| valor.as_str())
        }
        _ => None,
    }
}

/// ¿El selector completo (con sus descendientes) le aplica al elemento?
pub fn coincide(doc: &Documento, id: NodoId, selector: &Selector) -> bool {
    let Some((ultimo, ancestros)) = selector.cadena.split_last() else {
        return false;
    };
    if !coincide_simple(doc, id, ultimo) {
        return false;
    }
    // Para cada pieza restante, de derecha a izquierda, se sube hasta encontrar un ancestro que
    // la cumpla. Si se llega a la raíz sin encontrarlo, el selector no aplica. Basta con tomar
    // el primer ancestro que cumple: si uno más alto también cumple, no abre caminos nuevos.
    let mut actual = id;
    for simple in ancestros.iter().rev() {
        loop {
            let Some(padre) = doc.nodo(actual).padre else {
                return false;
            };
            actual = padre;
            if coincide_simple(doc, actual, simple) {
                break;
            }
        }
    }
    true
}

/// ¿Un selector sin espacios (`div.caja#x`) le aplica al elemento? Todas sus partes deben
/// cumplirse; `*` no tiene partes y le aplica a cualquier elemento.
fn coincide_simple(doc: &Documento, id: NodoId, simple: &SelectorSimple) -> bool {
    let Some(nombre) = doc.nombre(id) else {
        return false; // El documento y los textos nunca coinciden con un selector.
    };
    if simple.etiqueta.as_deref().is_some_and(|etiqueta| etiqueta != nombre) {
        return false;
    }
    if let Some(buscado) = &simple.id {
        if atributo(doc, id, "id") != Some(buscado.as_str()) {
            return false;
        }
    }
    let clases = atributo(doc, id, "class").unwrap_or("");
    simple.clases.iter().all(|buscada| clases.split_whitespace().any(|c| c == buscada))
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::{css, html};

    /// El primer elemento con esa etiqueta, recorriendo el árbol en orden.
    fn buscar(doc: &Documento, etiqueta: &str) -> NodoId {
        fn recorrer(doc: &Documento, id: NodoId, etiqueta: &str) -> Option<NodoId> {
            if doc.nombre(id) == Some(etiqueta) {
                return Some(id);
            }
            doc.nodo(id).hijos.iter().find_map(|&h| recorrer(doc, h, etiqueta))
        }
        recorrer(doc, doc.raiz(), etiqueta).expect("la etiqueta existe")
    }

    fn selector(texto: &str) -> Selector {
        css::parsear(&format!("{texto} {{}}")).reglas.remove(0).selectores.remove(0)
    }

    const PAGINA: &str = r#"<html><body><nav id="menu" class="barra"><ul><li><a class="link activo">x</a></li></ul></nav><p class="nota">y</p></body></html>"#;

    #[test]
    fn etiqueta_clase_id_y_universal() {
        let doc = html::parsear(PAGINA);
        let a = buscar(&doc, "a");
        for s in ["a", "*", ".link", ".activo", "a.link.activo", ".activo.link"] {
            assert!(coincide(&doc, a, &selector(s)), "{s} debería coincidir");
        }
        for s in ["p", ".nota", "a.otra", "#menu", "a#menu"] {
            assert!(!coincide(&doc, a, &selector(s)), "{s} no debería coincidir");
        }
        let nav = buscar(&doc, "nav");
        assert!(coincide(&doc, nav, &selector("#menu")));
        assert!(coincide(&doc, nav, &selector("nav#menu.barra")));
    }

    #[test]
    fn la_clase_se_compara_entera_y_con_mayusculas() {
        let doc = html::parsear(r#"<p class="  nota-roja   Grande ">x</p>"#);
        let p = buscar(&doc, "p");
        assert!(coincide(&doc, p, &selector(".nota-roja")));
        assert!(coincide(&doc, p, &selector(".Grande")));
        assert!(!coincide(&doc, p, &selector(".nota")));
        assert!(!coincide(&doc, p, &selector(".grande")));
    }

    #[test]
    fn descendiente_sube_por_los_padres() {
        let doc = html::parsear(PAGINA);
        let a = buscar(&doc, "a");
        for s in ["nav a", "ul a", "nav ul a", "body nav li a", "#menu .link", "html *"] {
            assert!(coincide(&doc, a, &selector(s)), "{s} debería coincidir");
        }
        for s in ["p a", "a nav", "ul nav a", "li ul a", "a a"] {
            assert!(!coincide(&doc, a, &selector(s)), "{s} no debería coincidir");
        }
        let p = buscar(&doc, "p");
        assert!(!coincide(&doc, p, &selector("nav p")));
        assert!(coincide(&doc, p, &selector("body p")));
    }

    #[test]
    fn descendiente_con_el_mismo_ancestro_repetido() {
        // `div div p`: hacen falta dos `div` distintos por encima del `p`.
        let doc = html::parsear("<div><p>x</p></div>");
        assert!(!coincide(&doc, buscar(&doc, "p"), &selector("div div p")));
        let doc = html::parsear("<div><div><p>x</p></div></div>");
        assert!(coincide(&doc, buscar(&doc, "p"), &selector("div div p")));
    }
}
