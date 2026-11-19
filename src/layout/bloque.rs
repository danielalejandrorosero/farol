//! Medidas de las cajas de bloque: el ancho baja del padre, el alto sube de los hijos.
//!
//! 1. **Ancho (de arriba hacia abajo).** Un bloque sabe cuánto mide de ancho solo cuando sabe
//!    cuánto mide su contenedor: `width: auto` es "lo que sobre", y `50%` es "la mitad del
//!    padre". Por eso el padre se mide antes que los hijos.
//! 2. **Posición.** Cada hijo se apoya debajo del anterior, como libros apilados.
//! 3. **Alto (de abajo hacia arriba).** Con `height: auto` el bloque mide lo que midan sus hijos
//!    apilados, así que solo se sabe después de medirlos a todos.
//!
//! Decisión 5.1: **sin colapso de márgenes**. Entre dos hermanos queda margen-inferior +
//! margen-superior (CSS dejaría el mayor de los dos), y el margen de un primer hijo no se fusiona
//! con el del padre.

use super::en_linea;
use super::tipos::{Caja, Rect, TipoCaja};
use crate::estilo::{Estilo, Lados, Longitud};

/// Mide y ubica `caja` dentro de un contenedor cuyo contenido empieza en `x` y mide
/// `ancho_contenedor`; el borde de arriba del margen de la caja queda en `y`.
pub fn colocar(caja: &mut Caja, x: f32, y: f32, ancho_contenedor: f32) {
    if let TipoCaja::Anonima(nodos) = &caja.tipo {
        // Una caja anónima no tiene márgenes, bordes ni relleno: ocupa todo el ancho, y mide de
        // alto lo que midan sus renglones apilados.
        caja.lineas = en_linea::colocar(nodos, &caja.estilo, x, y, ancho_contenedor);
        let alto = caja.lineas.iter().map(|l| l.rect.alto).sum();
        caja.dimensiones.contenido = Rect { x, y, ancho: ancho_contenedor, alto };
        return;
    }

    calcular_anchos(caja, ancho_contenedor);
    let d = &mut caja.dimensiones;
    d.contenido.x = x + d.margen.izquierda + d.borde.izquierda + d.relleno.izquierda;
    d.contenido.y = y + d.margen.arriba + d.borde.arriba + d.relleno.arriba;

    // Los hijos, uno debajo del otro. `cursor` es dónde empieza el siguiente.
    let contenido = d.contenido;
    let mut cursor = contenido.y;
    for hijo in &mut caja.hijos {
        colocar(hijo, contenido.x, cursor, contenido.ancho);
        cursor += hijo.dimensiones.caja_margen().alto;
    }

    caja.dimensiones.contenido.alto = match caja.estilo.alto {
        Longitud::Px(alto) => alto,
        // `height: 50%` solo vale si el contenedor tiene alto fijo; casi nunca lo tiene, y
        // entonces CSS lo trata como `auto`. Farol lo trata siempre así (deuda anotada).
        Longitud::Auto | Longitud::Porcentaje(_) => cursor - contenido.y,
    };
}

/// CSS 2.1 §10.3.3: las siete medidas horizontales tienen que sumar el ancho del contenedor.
///
/// ```text
/// margen-izq + borde-izq + relleno-izq + ancho + relleno-der + borde-der + margen-der = contenedor
/// ```
///
/// Bordes y relleno nunca son `auto`. Lo que sobra (o falta) se reparte entre lo que sí es
/// `auto`; si nada lo es, se lo lleva el margen derecho. Luego `max-width` pone un tope.
fn calcular_anchos(caja: &mut Caja, contenedor: f32) {
    let estilo = &caja.estilo;
    let a_px = |l: Longitud| resolver(l, contenedor);
    // Relleno y márgenes verticales en % también se miden contra el ANCHO del contenedor.
    let relleno = Lados::desde_orden(estilo.relleno.en_orden().map(|l| a_px(l).unwrap_or(0.0)));
    let borde = estilo.borde_ancho;
    let mut ancho = anchos_horizontales(estilo, contenedor, a_px(estilo.ancho), &relleno, &borde);
    if let Some(maximo) = a_px(estilo.ancho_maximo) {
        if ancho.1 > maximo {
            ancho = anchos_horizontales(estilo, contenedor, Some(maximo), &relleno, &borde);
        }
    }
    let (margen_izq, ancho, margen_der) = ancho;
    let d = &mut caja.dimensiones;
    d.relleno = relleno;
    d.borde = borde;
    d.margen = Lados {
        arriba: a_px(estilo.margen.arriba).unwrap_or(0.0),
        derecha: margen_der,
        abajo: a_px(estilo.margen.abajo).unwrap_or(0.0),
        izquierda: margen_izq,
    };
    d.contenido.ancho = ancho;
}

/// Resuelve `margen-izq`, `ancho` y `margen-der` con un `ancho` dado (`None` = `auto`).
fn anchos_horizontales(
    estilo: &Estilo,
    contenedor: f32,
    ancho: Option<f32>,
    relleno: &Lados<f32>,
    borde: &Lados<f32>,
) -> (f32, f32, f32) {
    let mut izq = resolver(estilo.margen.izquierda, contenedor);
    let mut der = resolver(estilo.margen.derecha, contenedor);
    let fijo = relleno.izquierda + relleno.derecha + borde.izquierda + borde.derecha;

    // Si con ancho fijo ya nos pasamos del contenedor, los márgenes `auto` valen 0.
    if let Some(ancho) = ancho {
        let total = izq.unwrap_or(0.0) + fijo + ancho + der.unwrap_or(0.0);
        if total > contenedor {
            izq = Some(izq.unwrap_or(0.0));
            der = Some(der.unwrap_or(0.0));
        }
    }
    // Lo que sobra (positivo) o falta (negativo), contando los `auto` como 0.
    let sobra = contenedor - (izq.unwrap_or(0.0) + fijo + ancho.unwrap_or(0.0) + der.unwrap_or(0.0));

    match (ancho, izq, der) {
        // `width: auto` llena el espacio; los márgenes `auto` que hubiera valen 0.
        (None, izq, der) => {
            let (izq, der) = (izq.unwrap_or(0.0), der.unwrap_or(0.0));
            if sobra >= 0.0 {
                (izq, sobra, der)
            } else {
                // No cabe ni con ancho 0: el ancho queda en 0 y el margen derecho absorbe.
                (izq, 0.0, der + sobra)
            }
        }
        // Todo fijo (sobre-especificado): el margen derecho cede, como en un texto de
        // izquierda a derecha.
        (Some(ancho), Some(izq), Some(der)) => (izq, ancho, der + sobra),
        // Un solo margen `auto`: se lleva todo lo que sobra.
        (Some(ancho), None, Some(der)) => (sobra, ancho, der),
        (Some(ancho), Some(izq), None) => (izq, ancho, sobra),
        // Los dos `auto`: se reparten mitad y mitad, y la caja queda centrada.
        (Some(ancho), None, None) => (sobra / 2.0, ancho, sobra / 2.0),
    }
}

/// Una longitud en px, o `None` si es `auto`. Los `%` se miden contra el ancho del contenedor.
fn resolver(l: Longitud, contenedor: f32) -> Option<f32> {
    match l {
        Longitud::Auto => None,
        Longitud::Px(px) => Some(px),
        Longitud::Porcentaje(p) => Some(p / 100.0 * contenedor),
    }
}
