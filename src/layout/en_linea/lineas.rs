//! Paso 2 del layout en línea: repartir la cinta de piezas en renglones (decisión 8.1: voraz).
//!
//! **Partición voraz.** Igual que alguien escribiendo a mano en un cuaderno: se escribe palabra
//! tras palabra y, si la siguiente no cabe en lo que queda del renglón, se pasa al siguiente. No
//! se mira hacia adelante para buscar un reparto más parejo (eso hace TeX, y cuesta mucho más).
//! Una palabra más larga que el renglón entero va sola en su renglón y se sale por la derecha:
//! no se parte nunca.
//!
//! La unidad que se mide es la **unidad irrompible**: todo lo que hay entre dos lugares donde se
//! puede partir. Por ejemplo `<b>muy</b>bueno ` es una sola unidad (no hay espacio en medio).
//! El espacio que la cierra no cuenta para ver si cabe: al final de un renglón se elimina.
//!
//! **Alto de cada renglón y línea base.** Todo lo de un renglón se apoya en una misma línea base,
//! como las letras sobre el renglón de un cuaderno. Cada pedazo de texto pide lugar arriba de la
//! base (su `ascendente` más la mitad del interlineado que le sobra) y abajo (su `descendente`
//! más la otra mitad). El renglón mide lo más que pide alguien arriba más lo más que pide alguien
//! abajo. Siempre cuenta también el "puntal" del bloque: un pedazo invisible con su letra y su
//! `line-height`, para que un renglón con letra chica no mida menos que el `line-height` del
//! párrafo. Es el modelo de CSS 2.1 §10.8 simplificado: todo va en `vertical-align: baseline`, y
//! los elementos en línea (`span`, `b`) no aportan su propio puntal, solo su texto.

use std::ptr;

use super::piezas::Pieza;
use crate::estilo::{Alineacion, Estilo, Lados, Longitud};
use crate::layout::tipos::{CajaEnLinea, Fragmento, Linea, Rect};
use crate::pintado::fuente;

/// Lo que el bloque le impone a todos sus renglones.
pub struct Bloque {
    pub x: f32,
    pub y: f32,
    pub ancho: f32,
    pub tam_fuente: f32,
    pub alto_linea: f32,
    pub alineacion: Alineacion,
}

/// Acomoda la cinta en renglones uno debajo del otro, desde `bloque.y`.
pub fn partir<'a>(piezas: &[Pieza<'a>], bloque: &Bloque) -> Vec<Linea<'a>> {
    let mut obra = Obra::nuevo(bloque);
    let limite = bloque.x + bloque.ancho;
    let mut i = 0;
    while i < piezas.len() {
        if matches!(piezas[i], Pieza::Salto) {
            obra.terminar_linea();
            i += 1;
            continue;
        }
        let fin = fin_de_unidad(piezas, i);
        let unidad = &piezas[i..fin];
        // ¿Cabe en lo que queda del renglón? Un renglón vacío acepta cualquier cosa (si no, una
        // palabra más ancha que el renglón no entraría nunca).
        let ancho = ancho_sin_espacio_final(unidad, bloque.ancho);
        if obra.hay_contenido && obra.x + ancho > limite + 0.01 {
            obra.terminar_linea();
        }
        for pieza in unidad {
            obra.colocar(pieza);
        }
        i = fin;
    }
    if obra.hay_contenido {
        obra.terminar_linea();
    }
    obra.lineas
}

/// Dónde termina la unidad irrompible que empieza en `i`: después de su espacio y de los cierres
/// pegados a él (`<b>a </b>` cierra el `b` en este renglón, no en el siguiente). Un `<br>` va
/// siempre solo.
fn fin_de_unidad(piezas: &[Pieza], mut i: usize) -> usize {
    while i < piezas.len() {
        match piezas[i] {
            Pieza::Salto => return i,
            Pieza::Espacio { .. } => {
                i += 1;
                while matches!(piezas.get(i), Some(Pieza::Cierra(_))) {
                    i += 1;
                }
                return i;
            }
            _ => i += 1,
        }
    }
    i
}

/// Lo que ocupa una unidad en el renglón, sin contar su espacio final (que se borra si queda al
/// final del renglón).
fn ancho_sin_espacio_final(unidad: &[Pieza], contenedor: f32) -> f32 {
    unidad
        .iter()
        .map(|p| match p {
            Pieza::Palabra { texto, estilo } => ancho(texto, estilo),
            Pieza::Abre(e) => extra_izquierdo(e, contenedor),
            Pieza::Cierra(e) => extra_derecho(e, contenedor),
            Pieza::Espacio { .. } | Pieza::Salto => 0.0,
        })
        .sum()
}

fn ancho(texto: &str, estilo: &Estilo) -> f32 {
    fuente::ancho_texto(texto, estilo.tam_fuente, estilo.negrita)
}

/// Un `%` de relleno o margen se mide contra el ancho del bloque contenedor, como en los bloques.
fn px(l: Longitud, contenedor: f32) -> f32 {
    match l {
        Longitud::Px(px) => px,
        Longitud::Porcentaje(p) => p / 100.0 * contenedor,
        Longitud::Auto => 0.0,
    }
}

fn extra_izquierdo(e: &Estilo, contenedor: f32) -> f32 {
    px(e.margen.izquierda, contenedor) + e.borde_ancho.izquierda + px(e.relleno.izquierda, contenedor)
}

fn extra_derecho(e: &Estilo, contenedor: f32) -> f32 {
    px(e.relleno.derecha, contenedor) + e.borde_ancho.derecha + px(e.margen.derecha, contenedor)
}

/// Un elemento en línea abierto: su estilo y dónde está su pedazo en el renglón en obra.
struct Abierto<'a> {
    estilo: &'a Estilo,
    indice: usize,
}

/// El renglón que se está llenando, más los ya terminados.
struct Obra<'a, 'b> {
    bloque: &'b Bloque,
    lineas: Vec<Linea<'a>>,
    /// La `y` donde empieza el renglón en obra.
    y: f32,
    /// Dónde va la próxima pieza (la pluma).
    x: f32,
    hay_contenido: bool,
    fragmentos: Vec<Fragmento<'a>>,
    cajas: Vec<CajaEnLinea<'a>>,
    abiertos: Vec<Abierto<'a>>,
    /// Si lo último colocado fue un espacio, cuánto mide (para borrarlo si el renglón termina).
    espacio_final: Option<f32>,
}

impl<'a, 'b> Obra<'a, 'b> {
    fn nuevo(bloque: &'b Bloque) -> Self {
        Obra {
            bloque,
            lineas: Vec::new(),
            y: bloque.y,
            x: bloque.x,
            hay_contenido: false,
            fragmentos: Vec::new(),
            cajas: Vec::new(),
            abiertos: Vec::new(),
            espacio_final: None,
        }
    }

    fn colocar(&mut self, pieza: &Pieza<'a>) {
        let contenedor = self.bloque.ancho;
        self.hay_contenido = true;
        match *pieza {
            Pieza::Palabra { ref texto, estilo } => {
                self.escribir(texto, estilo);
                self.espacio_final = None;
            }
            Pieza::Espacio { estilo } => {
                self.escribir(" ", estilo);
                self.espacio_final = Some(ancho(" ", estilo));
            }
            Pieza::Abre(e) => {
                self.x += px(e.margen.izquierda, contenedor);
                let borde = Lados { derecha: 0.0, ..e.borde_ancho };
                self.abiertos.push(Abierto { estilo: e, indice: self.cajas.len() });
                let rect = Rect { x: self.x, ..Rect::default() };
                self.cajas.push(CajaEnLinea { rect, borde, estilo: e });
                self.x += e.borde_ancho.izquierda + px(e.relleno.izquierda, contenedor);
                self.espacio_final = None;
            }
            Pieza::Cierra(e) => {
                self.x += px(e.relleno.derecha, contenedor) + e.borde_ancho.derecha;
                if let Some(abierto) = self.abiertos.pop() {
                    let caja = &mut self.cajas[abierto.indice];
                    caja.rect.ancho = self.x - caja.rect.x;
                    caja.borde.derecha = e.borde_ancho.derecha;
                }
                self.x += px(e.margen.derecha, contenedor);
            }
            Pieza::Salto => {}
        }
    }

    /// Agrega texto en la pluma. Si el fragmento anterior tiene el mismo estilo (el mismo nodo de
    /// texto) y termina justo aquí, se alarga en vez de crear otro: así "azul y subrayado" es un
    /// solo fragmento y su subrayado no se corta entre palabras.
    fn escribir(&mut self, texto: &str, estilo: &'a Estilo) {
        let w = ancho(texto, estilo);
        let x = self.x;
        match self.fragmentos.last_mut() {
            Some(f) if ptr::eq(f.estilo, estilo) && (f.rect.x + f.rect.ancho - x).abs() < 0.01 => {
                f.texto.push_str(texto);
                f.rect.ancho += w;
            }
            _ => self.fragmentos.push(Fragmento {
                rect: Rect { x, y: 0.0, ancho: w, alto: 0.0 },
                texto: texto.to_string(),
                estilo,
            }),
        }
        self.x += w;
    }

    /// Cierra el renglón en obra: quita el espacio final, calcula alto y línea base, alinea, y
    /// deja la pluma al principio del siguiente.
    fn terminar_linea(&mut self) {
        self.quitar_espacio_final();
        // Los elementos que siguen abiertos se cortan aquí: su pedazo en este renglón no lleva
        // borde derecho.
        for abierto in &self.abiertos {
            let caja = &mut self.cajas[abierto.indice];
            caja.rect.ancho = self.x - caja.rect.x;
        }

        let bloque = self.bloque;
        let (arriba, abajo) = self.fragmentos.iter().map(|f| f.estilo).fold(
            pide(bloque.tam_fuente, bloque.alto_linea),
            |(arriba, abajo), e| {
                let (a, b) = pide(e.tam_fuente, e.alto_linea);
                (arriba.max(a), abajo.max(b))
            },
        );
        let base = self.y + arriba;
        // Lo que sobra a la derecha del renglón se pone toda a la izquierda (`text-align: right`),
        // mitad y mitad (`center`) o nada. Si el renglón desborda, no se corre.
        let sobra = (bloque.x + bloque.ancho - self.x).max(0.0);
        let corrimiento = sobra
            * match bloque.alineacion {
                Alineacion::Izquierda => 0.0,
                Alineacion::Centro => 0.5,
                Alineacion::Derecha => 1.0,
            };

        let mut fragmentos = std::mem::take(&mut self.fragmentos);
        for f in &mut fragmentos {
            let tam = f.estilo.tam_fuente;
            f.rect.x += corrimiento;
            f.rect.y = base - fuente::linea_base(tam);
            f.rect.alto = fuente::alto_linea_natural(tam);
        }
        let mut cajas = std::mem::take(&mut self.cajas);
        for c in &mut cajas {
            c.rect.x += corrimiento;
            // El fondo cubre la celda de la letra del elemento más su relleno y borde de arriba y
            // abajo. Eso no agranda el renglón (en CSS tampoco): puede pisar los renglones vecinos.
            let tam = c.estilo.tam_fuente;
            let encima = px(c.estilo.relleno.arriba, bloque.ancho) + c.borde.arriba;
            let debajo = px(c.estilo.relleno.abajo, bloque.ancho) + c.borde.abajo;
            c.rect.y = base - fuente::ascendente(tam) - encima;
            c.rect.alto = tam + encima + debajo;
        }
        let alto = arriba + abajo;
        self.lineas.push(Linea {
            rect: Rect { x: bloque.x, y: self.y, ancho: bloque.ancho, alto },
            base,
            fragmentos,
            cajas,
        });

        self.y += alto;
        self.x = bloque.x;
        self.hay_contenido = false;
        // Los elementos que quedaron abiertos siguen en el renglón nuevo, sin borde izquierdo.
        for abierto in &mut self.abiertos {
            let e = abierto.estilo;
            abierto.indice = self.cajas.len();
            self.cajas.push(CajaEnLinea {
                rect: Rect { x: bloque.x, ..Rect::default() },
                borde: Lados { izquierda: 0.0, derecha: 0.0, ..e.borde_ancho },
                estilo: e,
            });
        }
    }

    /// El espacio con que terminó el renglón no se ve: se quita del último fragmento que termina
    /// en espacio y la pluma retrocede. Los elementos que se cerraron después de ese espacio
    /// (`<b>hola </b>` al final del renglón) también se acortan.
    fn quitar_espacio_final(&mut self) {
        let Some(w) = self.espacio_final.take() else { return };
        if let Some(i) = self.fragmentos.iter().rposition(|f| f.texto.ends_with(' ')) {
            let f = &mut self.fragmentos[i];
            let fin = f.rect.x + f.rect.ancho;
            f.texto.pop();
            f.rect.ancho -= w;
            if f.texto.is_empty() {
                self.fragmentos.remove(i);
            }
            let abiertos: Vec<usize> = self.abiertos.iter().map(|a| a.indice).collect();
            for (j, c) in self.cajas.iter_mut().enumerate() {
                if !abiertos.contains(&j) && c.rect.x + c.rect.ancho >= fin - 0.01 {
                    c.rect.ancho -= w;
                }
            }
        }
        self.x -= w;
    }
}

/// Cuánto pide un texto de letra `tam` y `line-height` `alto_linea` arriba y abajo de la línea
/// base. La letra ocupa `tam` (su ascendente arriba, su descendente abajo) y lo que sobra del
/// `line-height` se reparte mitad arriba y mitad abajo (el "medio interlineado" de CSS).
fn pide(tam: f32, alto_linea: f32) -> (f32, f32) {
    let medio = (alto_linea - tam) / 2.0;
    (fuente::ascendente(tam) + medio, fuente::descendente(tam) + medio)
}
