//! Selección de texto: qué letra hay debajo del mouse, qué parte está marcada y qué texto se
//! copia.
//!
//! Después de cada layout, todos los fragmentos de texto de la página se copian a una lista
//! plana, en el orden en que se leen. Un punto de la selección es "la letra número `car` del
//! fragmento número `frag`" de esa lista. Así, "todo lo que está entre dos puntos" es
//! simplemente un rango de la lista, igual que seleccionar en un documento de texto.

use crate::layout::{Caja, Rect};
use crate::pintado::fuente;

/// Un fragmento de texto ya ubicado, con lo necesario para medir sus letras.
#[derive(Debug, Clone, PartialEq)]
pub struct TextoPlano {
    pub rect: Rect,
    pub texto: String,
    pub tam: f32,
    pub negrita: bool,
    /// Número de renglón en toda la página: dos fragmentos con el mismo número van seguidos.
    pub linea: usize,
    /// Número de caja anónima (de párrafo, más o menos): al cambiar, el texto copiado salta de
    /// línea.
    pub bloque: usize,
}

impl TextoPlano {
    fn letras(&self) -> usize {
        self.texto.chars().count()
    }

    /// La `x` donde empieza la letra número `car` (o donde termina el texto, si `car` es el
    /// largo). Las letras de la fuente se suman una tras otra, así que basta medir el prefijo.
    fn x_de(&self, car: usize) -> f32 {
        let prefijo: String = self.texto.chars().take(car).collect();
        self.rect.x + fuente::ancho_texto(&prefijo, self.tam, self.negrita)
    }

    /// La posición entre letras más cercana a `x`: si el mouse está en la mitad derecha de una
    /// letra, el cursor va después de ella (como en cualquier editor).
    fn car_en(&self, x: f32) -> usize {
        let mut izquierda = self.rect.x;
        for (i, letra) in self.texto.chars().enumerate() {
            let ancho = fuente::ancho_texto(&letra.to_string(), self.tam, self.negrita);
            if x < izquierda + ancho / 2.0 {
                return i;
            }
            izquierda += ancho;
        }
        self.letras()
    }
}

/// Una posición entre dos letras. El orden de los campos hace que comparar dos puntos sea
/// comparar "cuál viene antes en la página".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Punto {
    pub frag: usize,
    pub car: usize,
}

/// Recorre el árbol de cajas y saca todos sus fragmentos en orden de lectura.
pub fn aplanar(raiz: &Caja) -> Vec<TextoPlano> {
    let mut textos = Vec::new();
    let (mut linea, mut bloque) = (0, 0);
    aplanar_caja(raiz, &mut textos, &mut linea, &mut bloque);
    textos
}

fn aplanar_caja(caja: &Caja, textos: &mut Vec<TextoPlano>, linea: &mut usize, bloque: &mut usize) {
    if !caja.lineas.is_empty() {
        for renglon in &caja.lineas {
            for f in &renglon.fragmentos {
                textos.push(TextoPlano {
                    rect: f.rect,
                    texto: f.texto.clone(),
                    tam: f.estilo.tam_fuente,
                    negrita: f.estilo.negrita,
                    linea: *linea,
                    bloque: *bloque,
                });
            }
            *linea += 1;
        }
        *bloque += 1;
    }
    for hijo in &caja.hijos {
        aplanar_caja(hijo, textos, linea, bloque);
    }
}

/// El punto de texto más cercano a `(x, y)`, en coordenadas de página. Primero se busca el
/// renglón más cercano en vertical y, dentro de él, el fragmento más cercano en horizontal. Así,
/// arrastrar por el espacio vacío a la derecha de un renglón selecciona hasta su final.
pub fn punto_en(textos: &[TextoPlano], x: f32, y: f32) -> Option<Punto> {
    let distancia = |desde: f32, hasta: f32, v: f32| (desde - v).max(v - hasta).max(0.0);
    let (frag, t) = textos.iter().enumerate().min_by(|(_, a), (_, b)| {
        let da = (distancia(a.rect.y, a.rect.y + a.rect.alto, y), distancia(a.rect.x, a.rect.x + a.rect.ancho, x));
        let db = (distancia(b.rect.y, b.rect.y + b.rect.alto, y), distancia(b.rect.x, b.rect.x + b.rect.ancho, x));
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })?;
    Some(Punto { frag, car: t.car_en(x) })
}

/// ¿Hay texto justo debajo de `(x, y)`? Sirve para poner el cursor en forma de "I".
pub fn hay_texto_en(textos: &[TextoPlano], x: f32, y: f32) -> bool {
    textos.iter().any(|t| {
        x >= t.rect.x && x < t.rect.x + t.rect.ancho && y >= t.rect.y && y < t.rect.y + t.rect.alto
    })
}

/// La palabra que contiene `punto` (doble clic): se extiende a los lados hasta un espacio.
pub fn palabra_en(textos: &[TextoPlano], punto: Punto) -> (Punto, Punto) {
    let letras: Vec<char> = textos[punto.frag].texto.chars().collect();
    let es_palabra = |c: char| c.is_alphanumeric() || c == '_' || c == '\'';
    // Si se hizo clic justo después de la última letra, se toma la letra anterior.
    let mut centro = punto.car.min(letras.len().saturating_sub(1));
    if centro > 0 && !letras.get(centro).is_some_and(|&c| es_palabra(c)) {
        centro -= 1;
    }
    if !letras.get(centro).is_some_and(|&c| es_palabra(c)) {
        let fin = (centro + 1).min(letras.len());
        return (Punto { frag: punto.frag, car: centro }, Punto { frag: punto.frag, car: fin });
    }
    let mut inicio = centro;
    while inicio > 0 && es_palabra(letras[inicio - 1]) {
        inicio -= 1;
    }
    let mut fin = centro;
    while fin < letras.len() && es_palabra(letras[fin]) {
        fin += 1;
    }
    (Punto { frag: punto.frag, car: inicio }, Punto { frag: punto.frag, car: fin })
}

/// Toda la página (Ctrl+A).
pub fn todo(textos: &[TextoPlano]) -> Option<(Punto, Punto)> {
    let ultimo = textos.last()?;
    Some((Punto { frag: 0, car: 0 }, Punto { frag: textos.len() - 1, car: ultimo.letras() }))
}

/// Ordena dos puntos: el ancla (donde empezó el arrastre) puede quedar después del foco.
fn ordenar(a: Punto, b: Punto) -> (Punto, Punto) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Para cada fragmento tocado por la selección, el rango de letras `[desde, hasta)`.
fn tramos(textos: &[TextoPlano], a: Punto, b: Punto) -> Vec<(usize, usize, usize)> {
    let (a, b) = ordenar(a, b);
    (a.frag..=b.frag.min(textos.len().saturating_sub(1)))
        .filter_map(|i| {
            let desde = if i == a.frag { a.car } else { 0 };
            let hasta = if i == b.frag { b.car } else { textos[i].letras() };
            (desde < hasta).then_some((i, desde, hasta))
        })
        .collect()
}

/// Los rectángulos que se pintan de azul, en coordenadas de página.
pub fn rectangulos(textos: &[TextoPlano], a: Punto, b: Punto) -> Vec<Rect> {
    tramos(textos, a, b)
        .into_iter()
        .map(|(i, desde, hasta)| {
            let t = &textos[i];
            let x0 = t.x_de(desde);
            Rect { x: x0, y: t.rect.y, ancho: t.x_de(hasta) - x0, alto: t.rect.alto }
        })
        .collect()
}

/// El texto seleccionado, listo para el portapapeles. Entre fragmentos del mismo renglón se
/// pone un espacio solo si en pantalla había un hueco; entre renglones de un mismo párrafo, un
/// espacio (el salto lo hizo el layout, no el autor); entre párrafos, un salto de línea.
pub fn texto(textos: &[TextoPlano], a: Punto, b: Punto) -> String {
    let mut salida = String::new();
    let mut anterior: Option<usize> = None;
    for (i, desde, hasta) in tramos(textos, a, b) {
        let t = &textos[i];
        if let Some(p) = anterior {
            let previo = &textos[p];
            let separador = if previo.bloque != t.bloque {
                "\n"
            } else if previo.linea != t.linea || t.rect.x - (previo.rect.x + previo.rect.ancho) > 0.5 {
                " "
            } else {
                ""
            };
            if !separador.is_empty() && !salida.ends_with([' ', '\n']) {
                salida.push_str(separador);
            }
        }
        salida.extend(t.texto.chars().skip(desde).take(hasta - desde));
        anterior = Some(i);
    }
    salida
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Un fragmento de 16 px, sin negrita, que empieza en `x`.
    fn frag(texto: &str, x: f32, linea: usize, bloque: usize) -> TextoPlano {
        TextoPlano {
            rect: Rect { x, y: linea as f32 * 20.0, ancho: fuente::ancho_texto(texto, 16.0, false), alto: 20.0 },
            texto: texto.into(),
            tam: 16.0,
            negrita: false,
            linea,
            bloque,
        }
    }

    fn ancho(texto: &str) -> f32 {
        fuente::ancho_texto(texto, 16.0, false)
    }

    #[test]
    fn el_clic_cae_entre_las_letras_mas_cercanas() {
        let textos = vec![frag("hola", 0.0, 0, 0)];
        // Un poquito antes del final de la "h": el cursor va después de la "h".
        let x = ancho("h") - 1.0;
        assert_eq!(punto_en(&textos, x, 5.0), Some(Punto { frag: 0, car: 1 }));
        assert_eq!(punto_en(&textos, -50.0, 5.0), Some(Punto { frag: 0, car: 0 }));
        assert_eq!(punto_en(&textos, 999.0, 5.0), Some(Punto { frag: 0, car: 4 }));
    }

    #[test]
    fn el_clic_a_la_derecha_de_un_renglon_va_a_su_final_y_no_al_de_abajo() {
        let textos = vec![frag("arriba", 0.0, 0, 0), frag("abajo y largo largo", 0.0, 1, 0)];
        assert_eq!(punto_en(&textos, 500.0, 10.0), Some(Punto { frag: 0, car: 6 }));
    }

    #[test]
    fn sin_texto_no_hay_punto() {
        assert_eq!(punto_en(&[], 0.0, 0.0), None);
    }

    #[test]
    fn doble_clic_toma_la_palabra_entera() {
        let textos = vec![frag("señor Ñandú", 0.0, 0, 0)];
        let (a, b) = palabra_en(&textos, Punto { frag: 0, car: 2 });
        assert_eq!(texto(&textos, a, b), "señor");
        let (a, b) = palabra_en(&textos, Punto { frag: 0, car: 8 });
        assert_eq!(texto(&textos, a, b), "Ñandú");
    }

    #[test]
    fn el_texto_respeta_el_sentido_del_arrastre() {
        let textos = vec![frag("abcdef", 0.0, 0, 0)];
        let a = Punto { frag: 0, car: 4 };
        let b = Punto { frag: 0, car: 1 };
        assert_eq!(texto(&textos, a, b), "bcd");
    }

    #[test]
    fn copiar_une_fragmentos_renglones_y_parrafos() {
        let x_negrita = ancho("Una ") ;
        let textos = vec![
            frag("Una ", 0.0, 0, 0),
            frag("caja", x_negrita, 0, 0),
            frag("sigue", 0.0, 1, 0),
            frag("Otro párrafo", 0.0, 2, 1),
        ];
        let (a, b) = todo(&textos).unwrap();
        assert_eq!(texto(&textos, a, b), "Una caja sigue\nOtro párrafo");
    }

    #[test]
    fn los_rectangulos_cubren_solo_las_letras_elegidas() {
        let textos = vec![frag("hola", 10.0, 0, 0)];
        let r = rectangulos(&textos, Punto { frag: 0, car: 1 }, Punto { frag: 0, car: 3 });
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].x, 10.0 + ancho("h"));
        assert_eq!(r[0].ancho, ancho("ol"));
    }

    #[test]
    fn una_seleccion_vacia_no_pinta_nada() {
        let textos = vec![frag("hola", 0.0, 0, 0)];
        let p = Punto { frag: 0, car: 2 };
        assert!(rectangulos(&textos, p, p).is_empty());
        assert_eq!(texto(&textos, p, p), "");
    }
}
