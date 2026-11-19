//! Fuente: convierte texto en pixeles. Fase 7, decisión 7.1 (fuente de mapa de bits propia).
//!
//! Una letra en pantalla no es más que un puñado de pixeles encendidos. Aquí cada letra (un
//! *glifo*) está dibujada a mano en `glifos.txt`, sobre una cuadrícula de 16 filas: la *celda*.
//! Ningún archivo de fuente del sistema ni librería interviene.
//!
//! ```text
//!   fila  0-2   tildes de mayúsculas (Á, Ñ, Ü)
//!   fila  3     tope de mayúsculas y astas (b, d, h, k, l)
//!   fila  6     tope de minúsculas (altura x); sus tildes van en las filas 3-4
//!   fila 12     última fila de tinta sobre la línea base
//!   ───────────  línea base: donde "se apoyan" las letras
//!   fila 13-15  descendentes (g, j, p, q, y)
//! ```
//!
//! **Escala.** `tam` es el `font-size` de CSS en px. La celda mide exactamente `tam` pixeles de
//! alto, así que a 16 px cada cuadrito del dibujo es un pixel de pantalla, a 32 px es un bloque
//! de 2×2, y a 12 px tres cuadritos caben en dos pixeles. Cada cuadrito encendido se pinta como
//! un rectángulo de `tam / 16` de lado (vecino más cercano), redondeado a pixeles enteros y de
//! al menos 1 pixel, para que un trazo fino no desaparezca al achicar.
//!
//! **Fuente proporcional.** Cada glifo tiene su propio ancho (la "i" mide 3, la "m" 7) y entre
//! letras se deja una columna vacía. Se lee más natural que una monoespaciada y es lo que
//! esperan casi todas las páginas web.
//!
//! **Caja de línea.** Una línea de texto mide `alto_linea_natural(tam)` = 1,25 × `tam` de alto.
//! La celda de 16 filas va centrada dentro: sobra 0,125 × `tam` arriba y otro tanto abajo (el
//! "medio interlineado" de CSS).

pub mod demostracion;

use std::collections::HashMap;
use std::sync::OnceLock;

use super::{Color, Lienzo};

/// Filas de la celda completa.
const FILAS_CELDA: usize = 16;
/// Filas que trae cada glifo en `glifos.txt` (las filas 3 a 15 de la celda).
const FILAS_ARCHIVO: usize = 13;
/// Primera fila de la celda que viene del archivo.
const PRIMERA_FILA_ARCHIVO: usize = FILAS_CELDA - FILAS_ARCHIVO;
/// La línea base está justo debajo de la fila 12: a 13 filas del tope de la celda.
const FILAS_HASTA_BASE: usize = 13;
/// Alto de la caja de línea comparado con `tam`.
const FACTOR_ALTO_LINEA: f32 = 1.25;
/// Columnas vacías entre una letra y la siguiente.
const ESPACIADO: usize = 1;
/// Cursiva: cuántas columnas se corre a la derecha cada fila por encima de la línea base.
const INCLINACION: f32 = 0.25;

/// Un glifo: su ancho en columnas y, por cada fila de la celda, qué columnas tienen tinta.
/// El bit `c` de `filas[f]` encendido significa "hay tinta en la columna `c` de la fila `f`".
#[derive(Clone, Debug, PartialEq)]
struct Glifo {
    ancho: usize,
    filas: [u32; FILAS_CELDA],
}

impl Glifo {
    fn tinta(&self, fila: usize, columna: usize) -> bool {
        self.filas[fila] >> columna & 1 == 1
    }
}

/// Todos los glifos, listos para usar.
struct Fuente {
    glifos: HashMap<char, Glifo>,
    tofu: Glifo,
}

/// Los dibujos de `glifos.txt` quedan dentro del ejecutable al compilar.
const GLIFOS: &str = include_str!("glifos.txt");

/// Marcas que se agregan encima de una letra base. Mismo formato que `glifos.txt`.
const AGUDO: &[&str] = &[".#", "#."];
const VIRGULILLA: &[&str] = &[".##.#", "#.##."];
const DIERESIS: &[&str] = &["#.#"];

/// Letras armadas en código: (letra, base, marca, ¿es mayúscula?).
/// La "í" se arma sobre la "ı" (i sin punto) para que la tilde reemplace al punto.
const COMPUESTAS: &[(char, char, &[&str], bool)] = &[
    ('á', 'a', AGUDO, false),
    ('é', 'e', AGUDO, false),
    ('í', 'ı', AGUDO, false),
    ('ó', 'o', AGUDO, false),
    ('ú', 'u', AGUDO, false),
    ('ñ', 'n', VIRGULILLA, false),
    ('ü', 'u', DIERESIS, false),
    ('Á', 'A', AGUDO, true),
    ('É', 'E', AGUDO, true),
    ('Í', 'I', AGUDO, true),
    ('Ó', 'O', AGUDO, true),
    ('Ú', 'U', AGUDO, true),
    ('Ñ', 'N', VIRGULILLA, true),
    ('Ü', 'U', DIERESIS, true),
];

/// La fuente se arma una sola vez, la primera vez que alguien la pide.
fn fuente() -> &'static Fuente {
    static FUENTE: OnceLock<Fuente> = OnceLock::new();
    // `glifos.txt` es parte del código: si estuviera mal escrito, las pruebas lo detectan.
    FUENTE.get_or_init(|| armar_fuente(GLIFOS).expect("glifos.txt mal formado"))
}

/// Lee `glifos.txt` y agrega las letras compuestas.
fn armar_fuente(texto: &str) -> Result<Fuente, String> {
    let mut glifos = HashMap::new();
    let mut tofu = None;
    let mut lineas = texto.lines().enumerate().peekable();
    while let Some((n, linea)) = lineas.next() {
        let linea = linea.trim_end();
        if linea.is_empty() || linea.starts_with("//") {
            continue;
        }
        let Some(nombre) = linea.strip_prefix("== ") else {
            return Err(format!(
                "línea {}: se esperaba '== X' y llegó {linea:?}",
                n + 1
            ));
        };
        let mut filas = Vec::with_capacity(FILAS_ARCHIVO);
        for _ in 0..FILAS_ARCHIVO {
            match lineas.next() {
                Some((_, fila)) => filas.push(fila.trim_end()),
                None => return Err(format!("el glifo {nombre:?} está incompleto")),
            }
        }
        let glifo = dibujo_a_glifo(&filas, PRIMERA_FILA_ARCHIVO)
            .map_err(|e| format!("glifo {nombre:?}: {e}"))?;
        if nombre == "tofu" {
            tofu = Some(glifo);
        } else {
            glifos.insert(leer_nombre(nombre)?, glifo);
        }
    }

    for &(letra, base, marca, mayuscula) in COMPUESTAS {
        let base = glifos
            .get(&base)
            .ok_or(format!("falta la base de {letra:?}"))?;
        let compuesta = poner_marca(base, marca, mayuscula)?;
        glifos.insert(letra, compuesta);
    }
    // El espacio duro (U+00A0, el `&nbsp;` de HTML) se ve igual que un espacio normal.
    if let Some(espacio) = glifos.get(&' ').cloned() {
        glifos.insert('\u{A0}', espacio);
    }

    let tofu = tofu.ok_or("falta el glifo 'tofu'")?;
    Ok(Fuente { glifos, tofu })
}

/// "A" → 'A'; "U+00F1" → 'ñ'.
fn leer_nombre(nombre: &str) -> Result<char, String> {
    if let Some(hex) = nombre.strip_prefix("U+") {
        return u32::from_str_radix(hex, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or(format!("código inválido {nombre:?}"));
    }
    let mut letras = nombre.chars();
    match (letras.next(), letras.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(format!("nombre de glifo inválido {nombre:?}")),
    }
}

/// Convierte filas como `".#.#."` en un glifo, empezando en la fila `primera` de la celda.
fn dibujo_a_glifo(dibujo: &[&str], primera: usize) -> Result<Glifo, String> {
    let ancho = dibujo.first().map_or(0, |f| f.chars().count());
    if ancho == 0 || ancho > 32 {
        return Err(format!("ancho inválido: {ancho}"));
    }
    let mut filas = [0u32; FILAS_CELDA];
    for (i, fila) in dibujo.iter().enumerate() {
        if fila.chars().count() != ancho {
            return Err(format!("la fila {i} ({fila:?}) no mide {ancho}"));
        }
        for (columna, c) in fila.chars().enumerate() {
            match c {
                '#' => filas[primera + i] |= 1 << columna,
                '.' => {}
                otro => return Err(format!("carácter {otro:?} en la fila {i}")),
            }
        }
    }
    Ok(Glifo { ancho, filas })
}

/// Pone una marca (tilde, virgulilla, diéresis) centrada encima de la letra base.
/// En minúsculas la marca termina en la fila 4 (justo encima de la altura x, dejando la fila 5
/// libre); en mayúsculas termina en la fila 1 (dejando libre la fila 2).
fn poner_marca(base: &Glifo, marca: &[&str], mayuscula: bool) -> Result<Glifo, String> {
    let ultima_fila = if mayuscula { 1 } else { 4 };
    let marca = dibujo_a_glifo(marca, ultima_fila + 1 - marca.len())?;
    let corrimiento = base.ancho.saturating_sub(marca.ancho).div_ceil(2);
    let mut resultado = base.clone();
    for (fila, bits) in marca.filas.iter().enumerate() {
        resultado.filas[fila] |= bits << corrimiento;
    }
    Ok(resultado)
}

/// El glifo de un carácter, o el tofu si la fuente no lo tiene.
fn glifo(c: char) -> &'static Glifo {
    let f = fuente();
    f.glifos.get(&c).unwrap_or(&f.tofu)
}

/// ¿La fuente tiene un dibujo propio para este carácter?
#[allow(dead_code)] // lo usan las pruebas
pub fn tiene_glifo(c: char) -> bool {
    fuente().glifos.contains_key(&c)
}

/// Cuántos pixeles de pantalla mide un cuadrito del dibujo. Un `tam` raro (negativo, cero,
/// infinito o NaN) da 0: no se dibuja nada y nada mide.
fn escala(tam: f32) -> f32 {
    if tam.is_finite() && tam > 0.0 {
        tam / FILAS_CELDA as f32
    } else {
        0.0
    }
}

/// Lo que avanza la pluma después de dibujar `c`, en columnas del dibujo.
fn avance(c: char, negrita: bool) -> usize {
    glifo(c).ancho + ESPACIADO + usize::from(negrita)
}

/// Ancho en pixeles que ocupa `texto` escrito a tamaño `tam`.
/// La negrita suma un cuadrito escalado por letra.
pub fn ancho_texto(texto: &str, tam: f32, negrita: bool) -> f32 {
    let columnas: usize = texto.chars().map(|c| avance(c, negrita)).sum();
    columnas as f32 * escala(tam)
}

/// Alto de una línea de texto con `line-height: normal`: 1,25 × `tam`.
pub fn alto_linea_natural(tam: f32) -> f32 {
    FACTOR_ALTO_LINEA * escala(tam) * FILAS_CELDA as f32
}

/// Distancia desde el tope de la caja de línea hasta la línea base.
/// Sirve para alinear en una misma línea textos de distinto tamaño.
pub fn linea_base(tam: f32) -> f32 {
    medio_interlineado(tam) + ascendente(tam)
}

/// Desde el tope de la celda hasta la línea base (incluye el espacio de las tildes).
pub fn ascendente(tam: f32) -> f32 {
    FILAS_HASTA_BASE as f32 * escala(tam)
}

/// Desde la línea base hasta el fondo de la celda (lo que bajan la g, la p...).
pub fn descendente(tam: f32) -> f32 {
    (FILAS_CELDA - FILAS_HASTA_BASE) as f32 * escala(tam)
}

/// El espacio que sobra arriba (y abajo) de la celda dentro de la caja de línea.
fn medio_interlineado(tam: f32) -> f32 {
    (alto_linea_natural(tam) - escala(tam) * FILAS_CELDA as f32) / 2.0
}

impl Lienzo {
    /// Escribe `texto` empezando en `(x, y)`.
    ///
    /// `y` es el **tope de la caja de línea** (de alto `alto_linea_natural(tam)`), no la línea
    /// base: así el layout puede apilar líneas sumando altos. Si una línea tiene otro alto,
    /// basta con correr `y` la mitad de la diferencia.
    ///
    /// Lo que quede fuera del lienzo se recorta; coordenadas negativas o enormes no fallan.
    #[allow(clippy::too_many_arguments)]
    pub fn texto(
        &mut self,
        x: f32,
        y: f32,
        texto: &str,
        tam: f32,
        color: Color,
        negrita: bool,
        cursiva: bool,
    ) {
        let s = escala(tam);
        if s == 0.0 || !x.is_finite() || !y.is_finite() {
            return;
        }
        // Cada glifo arranca en un pixel entero (arriba y a la izquierda). Así todas las "a" de
        // una línea se ven idénticas, aunque `tam / 16` no sea entero.
        let tope = (y + medio_interlineado(tam)).round();
        // Si toda la línea queda por encima o por debajo del lienzo, no hay nada que hacer.
        if tope + s * FILAS_CELDA as f32 <= 0.0 || tope >= self.alto() as f32 {
            return;
        }
        // La negrita repite el glifo un cuadrito escalado más a la derecha (al menos 1 pixel).
        let grosor = if negrita {
            (s.round() as i32).max(1)
        } else {
            0
        };

        let mut pluma = x;
        for c in texto.chars() {
            let paso = avance(c, negrita) as f32 * s;
            if pluma >= self.ancho() as f32 {
                break; // de aquí en adelante todo cae a la derecha del lienzo
            }
            // La cursiva puede correr la letra hasta ~3 columnas: se deja ese margen.
            if pluma + paso + 4.0 * s > 0.0 {
                let origen = (pluma.round() as i32, tope as i32);
                self.dibujar_glifo(glifo(c), origen, s, color, grosor, cursiva);
            }
            pluma += paso;
        }
    }

    /// Dibuja un glifo con la esquina de arriba a la izquierda de su celda en `origen`.
    fn dibujar_glifo(
        &mut self,
        g: &Glifo,
        (x, tope): (i32, i32),
        s: f32,
        color: Color,
        grosor: i32,
        cursiva: bool,
    ) {
        // Dónde empieza el cuadrito número `i` (de columna o de fila), contado desde el origen.
        let borde = |i: f32| (i * s).round() as i32;
        for fila in 0..FILAS_CELDA {
            if g.filas[fila] == 0 {
                continue;
            }
            // Vecino más cercano: el cuadrito (columna, fila) cubre de `columna·s` a
            // `(columna+1)·s` desde el origen, redondeado a pixeles enteros y nunca menos de 1.
            let y0 = tope.saturating_add(borde(fila as f32));
            let y1 = tope
                .saturating_add(borde((fila + 1) as f32))
                .max(y0.saturating_add(1));
            // Cursiva: las filas de arriba se corren a la derecha, las de abajo a la izquierda;
            // la línea base no se mueve.
            let corrimiento = if cursiva {
                (FILAS_HASTA_BASE as f32 - fila as f32 - 0.5) * INCLINACION
            } else {
                0.0
            };
            // Se pinta cada tramo seguido de tinta como un solo rectángulo.
            let mut columna = 0;
            while columna < g.ancho {
                if !g.tinta(fila, columna) {
                    columna += 1;
                    continue;
                }
                let inicio = columna;
                while columna < g.ancho && g.tinta(fila, columna) {
                    columna += 1;
                }
                let x0 = x.saturating_add(borde(corrimiento + inicio as f32));
                let x1 = x
                    .saturating_add(borde(corrimiento + columna as f32))
                    .max(x0.saturating_add(1));
                let ancho = x1.saturating_sub(x0).saturating_add(grosor);
                self.rectangulo(x0, y0, ancho, y1.saturating_sub(y0), color);
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const NEGRO: Color = 0x000000;
    const BLANCO: Color = 0xFFFFFF;

    /// Las coordenadas de los pixeles que no son del fondo.
    fn pintados(lienzo: &Lienzo) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        for (i, &p) in lienzo.pixeles().iter().enumerate() {
            if p != BLANCO {
                v.push((i % lienzo.ancho(), i / lienzo.ancho()));
            }
        }
        v
    }

    #[test]
    fn glifos_txt_esta_bien_formado() {
        assert!(
            armar_fuente(GLIFOS).is_ok(),
            "{:?}",
            armar_fuente(GLIFOS).err()
        );
    }

    #[test]
    fn toda_la_cobertura_tiene_glifo() {
        let mut faltan = Vec::new();
        for c in (32u8..=126).map(char::from) {
            if !tiene_glifo(c) {
                faltan.push(c);
            }
        }
        for c in "áéíóúÁÉÍÓÚñÑüÜ¿¡©°«»–—“”‘’…·\u{A0}".chars() {
            if !tiene_glifo(c) {
                faltan.push(c);
            }
        }
        assert!(faltan.is_empty(), "sin glifo: {faltan:?}");
    }

    #[test]
    fn un_caracter_desconocido_usa_el_tofu() {
        assert!(!tiene_glifo('漢'));
        assert_eq!(glifo('漢'), &fuente().tofu);
        assert_eq!(glifo('€'), &fuente().tofu);
        assert_ne!(glifo('A'), &fuente().tofu);
        // Y el tofu se dibuja: no es un hueco invisible.
        let mut lienzo = Lienzo::nuevo(20, 25, BLANCO);
        lienzo.texto(0.0, 0.0, "漢", 16.0, NEGRO, false, false);
        assert!(!pintados(&lienzo).is_empty());
    }

    #[test]
    fn las_tildes_no_tocan_la_letra() {
        // Entre la tilde y el cuerpo de la letra queda una fila vacía.
        let a = glifo('á');
        assert_ne!(a.filas[4], 0);
        assert_eq!(a.filas[5], 0);
        assert_ne!(a.filas[6], 0);
        let a = glifo('Á');
        assert_ne!(a.filas[1], 0);
        assert_eq!(a.filas[2], 0);
        // La í lleva tilde en lugar del punto de la i.
        assert_ne!(glifo('í'), glifo('i'));
        assert_eq!(glifo('í').filas[5], 0);
    }

    #[test]
    fn el_ancho_crece_con_el_texto() {
        let corto = ancho_texto("Hola", 16.0, false);
        let largo = ancho_texto("Hola, mundo", 16.0, false);
        assert!(corto > 0.0);
        assert!(largo > corto);
        assert_eq!(ancho_texto("", 16.0, false), 0.0);
    }

    #[test]
    fn la_negrita_es_mas_ancha() {
        let normal = ancho_texto("Farol", 16.0, false);
        let negrita = ancho_texto("Farol", 16.0, true);
        assert_eq!(negrita, normal + 5.0);
    }

    #[test]
    fn el_doble_de_tamano_mide_el_doble() {
        let a16 = ancho_texto("¿Qué tal, señor Ñandú?", 16.0, false);
        let a32 = ancho_texto("¿Qué tal, señor Ñandú?", 32.0, false);
        assert!((a32 - 2.0 * a16).abs() < 0.01);
        assert_eq!(alto_linea_natural(16.0), 20.0);
        assert_eq!(alto_linea_natural(32.0), 40.0);
    }

    #[test]
    fn la_fuente_es_proporcional() {
        assert!(ancho_texto("m", 16.0, false) > ancho_texto("i", 16.0, false));
        // "rn" no se confunde con "m": es más ancho.
        assert!(ancho_texto("rn", 16.0, false) > ancho_texto("m", 16.0, false));
    }

    #[test]
    fn las_metricas_suman_la_caja_de_linea() {
        let tam = 16.0;
        let total = linea_base(tam) + descendente(tam) + medio_interlineado(tam);
        assert_eq!(total, alto_linea_natural(tam));
        assert_eq!(ascendente(tam) + descendente(tam), tam);
    }

    #[test]
    fn dibujar_a_pinta_solo_dentro_de_su_caja() {
        let mut lienzo = Lienzo::nuevo(40, 40, BLANCO);
        let (x, y, tam) = (10.0, 5.0, 16.0);
        lienzo.texto(x, y, "A", tam, NEGRO, false, false);
        let pixeles = pintados(&lienzo);
        assert!(!pixeles.is_empty());
        let ancho = ancho_texto("A", tam, false);
        let alto = alto_linea_natural(tam);
        for &(px, py) in &pixeles {
            assert!(px as f32 >= x && (px as f32) < x + ancho, "x fuera: {px}");
            assert!(py as f32 >= y && (py as f32) < y + alto, "y fuera: {py}");
            assert_eq!(lienzo.pixeles()[py * 40 + px], NEGRO);
        }
        // La A no tiene descendente: nada por debajo de la línea base.
        let base = y + linea_base(tam);
        assert!(pixeles.iter().all(|&(_, py)| (py as f32) < base));
    }

    #[test]
    fn la_g_baja_de_la_linea_base() {
        let mut lienzo = Lienzo::nuevo(20, 25, BLANCO);
        lienzo.texto(0.0, 0.0, "g", 16.0, NEGRO, false, false);
        let base = linea_base(16.0);
        assert!(pintados(&lienzo).iter().any(|&(_, py)| py as f32 >= base));
    }

    #[test]
    fn a_32_px_la_letra_es_el_doble() {
        let mut chico = Lienzo::nuevo(40, 50, BLANCO);
        chico.texto(0.0, 0.0, "H", 16.0, NEGRO, false, false);
        let mut grande = Lienzo::nuevo(40, 50, BLANCO);
        grande.texto(0.0, 0.0, "H", 32.0, NEGRO, false, false);
        assert_eq!(pintados(&grande).len(), 4 * pintados(&chico).len());
    }

    #[test]
    fn la_negrita_y_la_cursiva_cambian_el_dibujo() {
        let dibujo = |negrita, cursiva| {
            let mut l = Lienzo::nuevo(30, 25, BLANCO);
            l.texto(2.0, 0.0, "l", 16.0, NEGRO, negrita, cursiva);
            pintados(&l)
        };
        let normal = dibujo(false, false);
        assert!(dibujo(true, false).len() > normal.len());
        assert_ne!(dibujo(false, true), normal);
    }

    #[test]
    fn un_tamano_chico_no_pierde_trazos() {
        // A 8 px dos cuadritos caben en un pixel, pero ninguna fila con tinta desaparece.
        let mut lienzo = Lienzo::nuevo(20, 20, BLANCO);
        lienzo.texto(0.0, 0.0, "E", 8.0, NEGRO, false, false);
        assert!(!pintados(&lienzo).is_empty());
    }

    #[test]
    fn dibujar_fuera_del_lienzo_no_falla() {
        let mut lienzo = Lienzo::nuevo(30, 20, BLANCO);
        lienzo.texto(-1e9, -1e9, "Hola", 16.0, NEGRO, true, true);
        lienzo.texto(1e9, 1e9, "Hola", 16.0, NEGRO, true, true);
        lienzo.texto(f32::MAX, f32::MIN, "Hola", f32::MAX, NEGRO, true, true);
        lienzo.texto(f32::NAN, 0.0, "Hola", 16.0, NEGRO, false, false);
        lienzo.texto(0.0, 0.0, "Hola", -5.0, NEGRO, false, false);
        lienzo.texto(0.0, 0.0, "Hola", 0.0, NEGRO, false, false);
        assert!(pintados(&lienzo).is_empty());
        // A medio salir por la izquierda y por arriba: se recorta sin fallar.
        lienzo.texto(-7.0, -9.0, "ÑÁ¿g", 24.0, NEGRO, true, true);
        assert!(!pintados(&lienzo).is_empty());
    }
}
