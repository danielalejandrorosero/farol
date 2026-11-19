//! El constructor del árbol: toma los tokens en orden y decide quién es hijo de quién.
//!
//! Usa una pila de elementos abiertos, como una torre de platos: abrir una etiqueta pone un
//! plato encima, cerrarla lo quita. El plato de arriba es el padre de lo que llegue después.

use super::dom::{Documento, NodoId, TipoNodo};
use super::tokenizador::Token;

/// Elementos que nunca tienen hijos ni etiqueta de cierre.
const VACIOS: [&str; 13] =
    ["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"];

/// Elementos de bloque que, al abrirse, cierran un `<p>` que haya quedado abierto.
const CIERRAN_P: [&str; 26] = [
    "address", "article", "aside", "blockquote", "details", "div", "dl", "fieldset", "figure",
    "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "main", "nav", "ol", "p",
    "pre", "section", "ul",
];

const TITULOS: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];

/// Elementos que funcionan como pared: la búsqueda de algo abierto no los atraviesa.
const PAREDES: [&str; 6] = ["html", "table", "td", "th", "caption", "button"];

pub fn construir(tokens: Vec<Token>) -> Documento {
    let mut doc = Documento::nuevo();
    let mut pila = vec![doc.raiz()];

    for token in tokens {
        let arriba = *pila.last().expect("la raíz nunca sale de la pila");
        match token {
            Token::Doctype | Token::Comentario(_) => {}
            Token::Texto(texto) => doc.agregar_texto(arriba, &texto),
            Token::Abre { nombre, atributos } => {
                cerrar_implicitos(&doc, &mut pila, &nombre);
                let padre = *pila.last().expect("la raíz nunca sale de la pila");
                let vacio = VACIOS.contains(&nombre.as_str());
                let id = doc.agregar(padre, TipoNodo::Elemento { nombre, atributos });
                if !vacio {
                    pila.push(id);
                }
            }
            Token::Cierra { nombre } => {
                // Se cierra el elemento abierto más cercano con ese nombre, y todo lo que haya
                // quedado abierto encima de él. Un cierre sin apertura se ignora.
                let abierto = pila.iter().rposition(|&id| doc.nombre(id) == Some(nombre.as_str()));
                if let Some(i) = abierto {
                    pila.truncate(i);
                }
            }
        }
    }
    // Al final del archivo, todo lo que siga abierto se cierra solo.
    doc
}

/// Las reglas de tolerancia: lo que un navegador cierra solo cuando la gente lo olvida.
fn cerrar_implicitos(doc: &Documento, pila: &mut Vec<NodoId>, nuevo: &str) {
    let nuevo_es = |lista: &[&str]| lista.contains(&nuevo);

    if nuevo_es(&CIERRAN_P) {
        cerrar_abierto(doc, pila, &["p"], &PAREDES);
    }
    if nuevo_es(&TITULOS) && doc.nombre(*pila.last().unwrap()).is_some_and(|n| TITULOS.contains(&n)) {
        pila.pop();
    }
    match nuevo {
        "li" => cerrar_abierto(doc, pila, &["li"], &[&PAREDES[..], &["ul", "ol"]].concat()),
        "dt" | "dd" => cerrar_abierto(doc, pila, &["dt", "dd"], &[&PAREDES[..], &["dl"]].concat()),
        "tr" => cerrar_abierto(doc, pila, &["tr"], &["table"]),
        "td" | "th" => cerrar_abierto(doc, pila, &["td", "th"], &["tr", "table"]),
        "option" => cerrar_abierto(doc, pila, &["option"], &["select"]),
        _ => {}
    }
}

/// Busca en la pila, de arriba hacia abajo, un elemento de `objetivos` abierto. Si lo encuentra
/// antes de chocar con una pared, lo cierra junto con todo lo que tenga encima.
fn cerrar_abierto(doc: &Documento, pila: &mut Vec<NodoId>, objetivos: &[&str], paredes: &[&str]) {
    for i in (1..pila.len()).rev() {
        let nombre = doc.nombre(pila[i]).unwrap_or("");
        if objetivos.contains(&nombre) {
            pila.truncate(i);
            return;
        }
        if paredes.contains(&nombre) {
            return;
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::super::parsear;

    /// Compara el árbol impreso, sin la primera línea (`#documento`).
    fn arbol(html: &str) -> String {
        parsear(html).imprimir().lines().skip(1).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn anida_elementos() {
        assert_eq!(arbol("<div><p>hola</p></div>"), "└── <div>\n    └── <p>\n        └── \"hola\"");
    }

    #[test]
    fn un_p_nuevo_cierra_el_anterior() {
        assert_eq!(arbol("<p>uno<p>dos"), "├── <p>\n│   └── \"uno\"\n└── <p>\n    └── \"dos\"");
    }

    #[test]
    fn un_div_cierra_un_p_abierto() {
        assert_eq!(arbol("<p>uno<div>dos</div>"), "├── <p>\n│   └── \"uno\"\n└── <div>\n    └── \"dos\"");
    }

    #[test]
    fn un_li_nuevo_cierra_el_anterior() {
        assert_eq!(
            arbol("<ul><li>a<li>b</ul>"),
            "└── <ul>\n    ├── <li>\n    │   └── \"a\"\n    └── <li>\n        └── \"b\""
        );
    }

    #[test]
    fn las_listas_anidadas_no_se_cierran_de_mas() {
        assert_eq!(
            arbol("<ul><li>a<ul><li>b</ul></ul>"),
            "└── <ul>\n    └── <li>\n        ├── \"a\"\n        └── <ul>\n            └── <li>\n                └── \"b\""
        );
    }

    #[test]
    fn los_elementos_vacios_no_reciben_hijos() {
        assert_eq!(arbol("<p>a<br>b<img src=x>c</p>"), "└── <p>\n    ├── \"a\"\n    ├── <br>\n    ├── \"b\"\n    ├── <img src=\"x\">\n    └── \"c\"");
    }

    #[test]
    fn un_cierre_sin_apertura_se_ignora() {
        assert_eq!(arbol("<div>a</span>b</div>"), "└── <div>\n    └── \"ab\"");
    }

    #[test]
    fn un_cierre_cierra_lo_que_quedo_abierto_adentro() {
        assert_eq!(arbol("<div><b>a</div>c"), "├── <div>\n│   └── <b>\n│       └── \"a\"\n└── \"c\"");
    }

    #[test]
    fn el_final_del_archivo_cierra_todo() {
        assert_eq!(arbol("<div><p>sin cerrar"), "└── <div>\n    └── <p>\n        └── \"sin cerrar\"");
    }

    #[test]
    fn los_espacios_entre_etiquetas_existen_aunque_no_se_impriman() {
        let doc = parsear("<div> <b>a</b> <i>b</i> </div>");
        let div = doc.nodo(doc.raiz()).hijos[0];
        // Texto, b, texto, i, texto: los espacios importan para separar palabras más adelante.
        assert_eq!(doc.nodo(div).hijos.len(), 5);
    }
}
