//! El DOM: el árbol de nodos de una página, guardado como una arena.
//!
//! Todos los nodos viven en una sola lista (`nodos`). Un nodo no apunta a otro directamente:
//! guarda su posición en la lista, como una columna `padre_id` en una tabla SQL.

use std::fmt::Write;

/// La posición de un nodo dentro de la arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodoId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum TipoNodo {
    /// La raíz invisible de la que cuelga todo; no corresponde a ninguna etiqueta.
    Documento,
    Elemento { nombre: String, atributos: Vec<(String, String)> },
    Texto(String),
}

#[derive(Debug)]
pub struct Nodo {
    pub tipo: TipoNodo,
    // Lo usará el motor de estilos (fase 4) para heredar propiedades del padre.
    #[allow(dead_code)]
    pub padre: Option<NodoId>,
    pub hijos: Vec<NodoId>,
}

pub struct Documento {
    nodos: Vec<Nodo>,
}

impl Documento {
    /// Un documento vacío: solo la raíz, en la posición 0.
    pub fn nuevo() -> Self {
        let raiz = Nodo { tipo: TipoNodo::Documento, padre: None, hijos: Vec::new() };
        Self { nodos: vec![raiz] }
    }

    pub fn raiz(&self) -> NodoId {
        NodoId(0)
    }

    pub fn nodo(&self, id: NodoId) -> &Nodo {
        &self.nodos[id.0]
    }

    /// El nombre de la etiqueta, si el nodo es un elemento.
    pub fn nombre(&self, id: NodoId) -> Option<&str> {
        match &self.nodo(id).tipo {
            TipoNodo::Elemento { nombre, .. } => Some(nombre),
            _ => None,
        }
    }

    /// Agrega un nodo al final de la lista y lo cuelga como último hijo de `padre`.
    pub fn agregar(&mut self, padre: NodoId, tipo: TipoNodo) -> NodoId {
        let id = NodoId(self.nodos.len());
        self.nodos.push(Nodo { tipo, padre: Some(padre), hijos: Vec::new() });
        self.nodos[padre.0].hijos.push(id);
        id
    }

    /// Agrega texto a `padre`. Si su último hijo ya es texto, lo une a ese en vez de crear otro.
    pub fn agregar_texto(&mut self, padre: NodoId, texto: &str) {
        if let Some(&ultimo) = self.nodos[padre.0].hijos.last() {
            if let TipoNodo::Texto(existente) = &mut self.nodos[ultimo.0].tipo {
                existente.push_str(texto);
                return;
            }
        }
        self.agregar(padre, TipoNodo::Texto(texto.to_string()));
    }

    /// El árbol dibujado con líneas, como lo imprime `farol --dom`.
    /// Los textos que son solo espacios en blanco existen en el DOM, pero no se muestran.
    pub fn imprimir(&self) -> String {
        let mut salida = String::from("#documento\n");
        self.imprimir_hijos(self.raiz(), "", &mut salida);
        salida
    }

    fn imprimir_hijos(&self, id: NodoId, prefijo: &str, salida: &mut String) {
        let visibles: Vec<(NodoId, String)> = self
            .nodo(id)
            .hijos
            .iter()
            .filter_map(|&hijo| self.rotulo(hijo).map(|r| (hijo, r)))
            .collect();
        for (i, (hijo, rotulo)) in visibles.iter().enumerate() {
            let ultimo = i + 1 == visibles.len();
            let rama = if ultimo { "└── " } else { "├── " };
            let _ = writeln!(salida, "{prefijo}{rama}{rotulo}");
            let siguiente = format!("{prefijo}{}", if ultimo { "    " } else { "│   " });
            self.imprimir_hijos(*hijo, &siguiente, salida);
        }
    }

    /// Cómo se ve un nodo en una línea, o `None` si es texto vacío.
    fn rotulo(&self, id: NodoId) -> Option<String> {
        match &self.nodo(id).tipo {
            TipoNodo::Documento => Some("#documento".into()),
            TipoNodo::Elemento { nombre, atributos } => {
                let mut r = format!("<{nombre}");
                for (clave, valor) in atributos {
                    let _ = if valor.is_empty() {
                        write!(r, " {clave}")
                    } else {
                        write!(r, " {clave}=\"{valor}\"")
                    };
                }
                r.push('>');
                Some(r)
            }
            TipoNodo::Texto(texto) => {
                let limpio = texto.split_whitespace().collect::<Vec<_>>().join(" ");
                if limpio.is_empty() {
                    return None;
                }
                let corto: String = limpio.chars().take(60).collect();
                let puntos = if limpio.chars().count() > 60 { "…" } else { "" };
                Some(format!("\"{corto}{puntos}\""))
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn agregar_enlaza_padre_e_hijo() {
        let mut doc = Documento::nuevo();
        let div = doc.agregar(doc.raiz(), TipoNodo::Elemento { nombre: "div".into(), atributos: vec![] });
        assert_eq!(div, NodoId(1));
        assert_eq!(doc.nodo(div).padre, Some(doc.raiz()));
        assert_eq!(doc.nodo(doc.raiz()).hijos, vec![div]);
    }

    #[test]
    fn el_texto_seguido_se_une_en_un_solo_nodo() {
        let mut doc = Documento::nuevo();
        doc.agregar_texto(doc.raiz(), "ho");
        doc.agregar_texto(doc.raiz(), "la");
        assert_eq!(doc.nodo(doc.raiz()).hijos.len(), 1);
        assert_eq!(doc.nodo(NodoId(1)).tipo, TipoNodo::Texto("hola".into()));
    }
}
