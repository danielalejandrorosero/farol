//! La display list: el árbol de cajas convertido en una lista plana de órdenes de dibujo.
//!
//! El layout dice *dónde* queda cada caja; esta lista dice *qué pintar y en qué orden*, sin tocar
//! un solo pixel todavía. Es como un guion para el pintor: "rectángulo gris aquí, borde azul
//! allá". Separarla del lienzo tiene dos ventajas: se puede probar comparando órdenes (sin mirar
//! pixeles) y se puede cambiar de pintor (CPU hoy, GPU algún día) sin tocar el layout.
//!
//! El orden importa porque cada orden pinta **encima** de las anteriores, como capas de pintura:
//! primero el fondo del lienzo, y luego, caja por caja en orden del documento, su fondo, sus
//! bordes y después sus hijos. Así un hijo siempre queda encima del fondo de su padre. Las cajas
//! anónimas agregan su texto (fase 8, `en_linea.rs`): primero los fondos y bordes de los
//! elementos en línea, después las letras y los subrayados.

use std::ptr;

use super::en_linea;
use crate::css;
use crate::estilo::Lados;
use crate::layout::{Caja, Rect};

/// Una instrucción de dibujo, en coordenadas de página.
#[derive(Debug, Clone, PartialEq)]
pub enum Orden {
    /// Cubre todo el lienzo, incluso lo que queda debajo del último bloque de la página o fuera
    /// de ella por el scroll. Siempre es la primera orden de la lista.
    FondoDelLienzo(css::Color),
    /// Un rectángulo relleno de un color (fondos y bordes).
    Rectangulo { rect: Rect, color: css::Color },
    /// Un pedazo de texto de un solo estilo. `y` es el tope de la caja de línea *natural* de la
    /// letra (`fuente::alto_linea_natural(tam)`), igual que en `Lienzo::texto`; `tam` en px.
    Texto {
        x: f32,
        y: f32,
        texto: String,
        tam: f32,
        color: css::Color,
        negrita: bool,
        cursiva: bool,
    },
}

const BLANCO: css::Color = css::Color::rgb(255, 255, 255);

/// Las órdenes para pintar la página cuya caja raíz es `raiz` (la de `#documento`).
pub fn lista_de_pintado(raiz: &Caja) -> Vec<Orden> {
    let (color, dueno) = fondo_del_lienzo(raiz);
    let mut ordenes = vec![Orden::FondoDelLienzo(color)];
    pintar_caja(raiz, dueno, &mut ordenes);
    ordenes
}

/// CSS 2.1 §14.2: el fondo del elemento raíz (`html`) cubre todo el lienzo; si es transparente,
/// se usa el de `body`; si los dos lo son, el lienzo queda blanco. Devuelve también la caja que
/// prestó su fondo, para que no lo vuelva a pintar en su propio rectángulo.
///
/// Las cajas no saben su etiqueta: `html` es el primer bloque bajo `#documento` y `body` el
/// primer bloque bajo `html` (`head` tiene `display: none` y no está en el árbol).
fn fondo_del_lienzo<'c, 'a>(raiz: &'c Caja<'a>) -> (css::Color, Option<&'c Caja<'a>>) {
    let html = primer_bloque(raiz);
    let body = html.and_then(primer_bloque);
    for caja in [html, body].into_iter().flatten() {
        if caja.estilo.fondo.a > 0 {
            return (caja.estilo.fondo, Some(caja));
        }
    }
    (BLANCO, None)
}

fn primer_bloque<'c, 'a>(caja: &'c Caja<'a>) -> Option<&'c Caja<'a>> {
    caja.hijos.iter().find(|h| matches!(h.tipo, crate::layout::TipoCaja::Bloque(_)))
}

fn pintar_caja(caja: &Caja, dueno_del_lienzo: Option<&Caja>, ordenes: &mut Vec<Orden>) {
    let es_dueno = dueno_del_lienzo.is_some_and(|d| ptr::eq(d, caja));
    if !es_dueno {
        agregar(ordenes, caja.dimensiones.caja_relleno(), caja.estilo.fondo);
    }
    pintar_bordes(caja, ordenes);
    en_linea::pintar_lineas(&caja.lineas, ordenes);
    for hijo in &caja.hijos {
        pintar_caja(hijo, dueno_del_lienzo, ordenes);
    }
}

/// El borde es el anillo entre la caja de borde (afuera) y la de relleno (adentro), partido en
/// cuatro rectángulos. Arriba y abajo van de punta a punta y se quedan con las esquinas;
/// izquierda y derecha van solo entre ellos, para que ningún pixel se pinte dos veces (importa
/// con colores semitransparentes).
///
/// ```text
/// ┌───────── arriba ─────────┐
/// │izq │    relleno    │ der │
/// └───────── abajo ──────────┘
/// ```
fn pintar_bordes(caja: &Caja, ordenes: &mut Vec<Orden>) {
    let d = &caja.dimensiones;
    pintar_anillo(d.caja_borde(), d.caja_relleno(), &d.borde, &caja.estilo.borde_color, ordenes);
}

/// Los cuatro lados del borde entre `fuera` (la caja de borde) y `dentro` (la de relleno). Lo
/// usan también los elementos en línea (`en_linea.rs`).
pub(super) fn pintar_anillo(
    fuera: Rect,
    dentro: Rect,
    ancho: &Lados<f32>,
    color: &Lados<css::Color>,
    ordenes: &mut Vec<Orden>,
) {
    let (fuera_der, fuera_abajo) = (fuera.x + fuera.ancho, fuera.y + fuera.alto);
    let (dentro_der, dentro_abajo) = (dentro.x + dentro.ancho, dentro.y + dentro.alto);

    // Cada lado se arma desde las aristas de los dos rectángulos, no sumando grosores: así su
    // arista interior es *exactamente* la del fondo y al redondear caen en el mismo pixel.
    let lados = [
        (ancho.arriba, color.arriba, entre(fuera.x, fuera.y, fuera_der, dentro.y)),
        (ancho.derecha, color.derecha, entre(dentro_der, dentro.y, fuera_der, dentro_abajo)),
        (ancho.abajo, color.abajo, entre(fuera.x, dentro_abajo, fuera_der, fuera_abajo)),
        (ancho.izquierda, color.izquierda, entre(fuera.x, dentro.y, dentro.x, dentro_abajo)),
    ];
    for (grosor, color, rect) in lados {
        if grosor > 0.0 {
            agregar(ordenes, rect, color);
        }
    }
}

/// El rectángulo de esquinas `(x0, y0)` arriba a la izquierda y `(x1, y1)` abajo a la derecha.
fn entre(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect { x: x0, y: y0, ancho: x1 - x0, alto: y1 - y0 }
}

/// Agrega un rectángulo, salvo que sea invisible (alfa 0) o no tenga área.
pub(super) fn agregar(ordenes: &mut Vec<Orden>, rect: Rect, color: css::Color) {
    if color.a > 0 && rect.ancho > 0.0 && rect.alto > 0.0 {
        ordenes.push(Orden::Rectangulo { rect, color });
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::estilo;
    use crate::html;
    use crate::layout;
    use crate::pintado::fuente;

    /// HTML → DOM → estilos → layout a 800 px → display list. Envuelve la página en `<html>` si
    /// no lo trae (el parser no lo inventa, y sin él el primer bloque haría de raíz).
    fn ordenes(pagina: &str) -> Vec<Orden> {
        let pagina = if pagina.starts_with("<html") {
            pagina.to_string()
        } else {
            format!("<html>{pagina}</html>")
        };
        let doc = html::parsear(&pagina);
        let estilado = estilo::estilar(&doc, &estilo::hojas_del_documento(&doc));
        lista_de_pintado(&layout::calcular(&estilado, 800.0))
    }

    fn rectangulos(ordenes: &[Orden]) -> Vec<(Rect, css::Color)> {
        ordenes
            .iter()
            .filter_map(|o| match o {
                Orden::Rectangulo { rect, color } => Some((*rect, *color)),
                _ => None,
            })
            .collect()
    }

    const ROJO: css::Color = css::Color::rgb(255, 0, 0);
    const VERDE: css::Color = css::Color::rgb(0, 255, 0);
    const AZUL: css::Color = css::Color::rgb(0, 0, 255);

    fn r(x: f32, y: f32, ancho: f32, alto: f32) -> Rect {
        Rect { x, y, ancho, alto }
    }

    #[test]
    fn fondo_antes_que_bordes_antes_que_hijos() {
        let lista = ordenes(
            "<body style='margin:0'>\
             <div style='background:#f00; border:1px solid #00f; height:10px'>\
             <p style='margin:0; background:#0f0; height:4px'></p></div></body>",
        );
        assert_eq!(lista[0], Orden::FondoDelLienzo(BLANCO));
        let colores: Vec<css::Color> = rectangulos(&lista).iter().map(|(_, c)| *c).collect();
        assert_eq!(colores, [ROJO, AZUL, AZUL, AZUL, AZUL, VERDE]);
        // El fondo cubre la caja de relleno: dentro del borde de 1 px.
        assert_eq!(rectangulos(&lista)[0].0, r(1.0, 1.0, 798.0, 10.0));
    }

    #[test]
    fn cuatro_bordes_con_anchos_distintos() {
        // Contenido 100 × 50 en (4, 1): borde arriba 1, derecha 2, abajo 3, izquierda 4.
        let lista = ordenes(
            "<body style='margin:0'><div style='width:100px; height:50px; border-style:solid;\
             border-width:1px 2px 3px 4px; border-color:#f00 #0f0 #00f #000'></div></body>",
        );
        let bordes = rectangulos(&lista);
        assert_eq!(
            bordes,
            [
                (r(0.0, 0.0, 106.0, 1.0), ROJO),   // arriba, de punta a punta
                (r(104.0, 1.0, 2.0, 50.0), VERDE), // derecha, entre arriba y abajo
                (r(0.0, 51.0, 106.0, 3.0), AZUL),  // abajo, de punta a punta
                (r(0.0, 1.0, 4.0, 50.0), css::Color::rgb(0, 0, 0)), // izquierda
            ]
        );
    }

    #[test]
    fn lo_transparente_no_genera_ordenes() {
        let lista = ordenes(
            "<body><div style='height:10px'></div>\
             <div style='height:10px; background: rgba(255, 0, 0, 0);\
             border: 3px solid transparent'></div>\
             <div style='height:10px; border-left: 0 solid red'></div></body>",
        );
        assert_eq!(lista, [Orden::FondoDelLienzo(BLANCO)]);
    }

    #[test]
    fn el_fondo_del_body_cubre_el_lienzo_y_no_se_repite() {
        let lista = ordenes("<body style='background:#00f'><div style='height:5px'></div></body>");
        assert_eq!(lista, [Orden::FondoDelLienzo(AZUL)]);
    }

    #[test]
    fn si_html_tiene_fondo_gana_html_y_body_pinta_el_suyo() {
        let lista = ordenes(
            "<html style='background:#f00'><body style='background:#0f0; margin:0'>\
             <div style='height:5px'></div></body></html>",
        );
        assert_eq!(lista[0], Orden::FondoDelLienzo(ROJO));
        assert_eq!(rectangulos(&lista), [(r(0.0, 0.0, 800.0, 5.0), VERDE)]);
    }

    #[test]
    fn las_cajas_anonimas_no_pintan_el_fondo_del_padre() {
        let lista =
            ordenes("<body style='margin:0'><div style='background:#f00'>hola</div></body>");
        assert_eq!(rectangulos(&lista).len(), 1);
    }

    fn textos(ordenes: &[Orden]) -> Vec<&Orden> {
        ordenes.iter().filter(|o| matches!(o, Orden::Texto { .. })).collect()
    }

    #[test]
    fn el_texto_se_pinta_en_la_posicion_de_su_fragmento() {
        // body sin margen; p con margen 16 arriba. El renglón empieza en y = 16 y mide 19.2: la
        // línea base queda en 16 + 13 + 1.6 = 30.6 y `y` (el tope de la caja de línea natural de
        // 20 px) en 30.6 − 15 = 15.6.
        let lista = ordenes("<body style='margin:0'><p style='color:#f00'>Hola <b>mundo</b></p></body>");
        let t = textos(&lista);
        let ancho_hola = fuente::ancho_texto("Hola ", 16.0, false);
        let esperado = [
            Orden::Texto {
                x: 0.0,
                y: 30.6 - fuente::linea_base(16.0),
                texto: "Hola ".into(),
                tam: 16.0,
                color: ROJO,
                negrita: false,
                cursiva: false,
            },
            Orden::Texto {
                x: ancho_hola,
                y: 30.6 - fuente::linea_base(16.0),
                texto: "mundo".into(),
                tam: 16.0,
                color: ROJO,
                negrita: true,
                cursiva: false,
            },
        ];
        assert_eq!(t.len(), 2);
        for (orden, esperada) in t.iter().zip(&esperado) {
            let (Orden::Texto { x, y, texto, .. }, Orden::Texto { x: ex, y: ey, texto: et, .. }) =
                (orden, esperada)
            else {
                unreachable!()
            };
            assert!((x - ex).abs() < 0.01 && (y - ey).abs() < 0.01, "{orden:?}");
            assert_eq!(texto, et);
        }
        // Lo demás (tamaño, color, negrita) coincide exacto.
        let sin_posicion = |o: &Orden| match o {
            Orden::Texto { texto, tam, color, negrita, cursiva, .. } => {
                (texto.clone(), *tam, *color, *negrita, *cursiva)
            }
            _ => unreachable!(),
        };
        assert_eq!(sin_posicion(t[1]), sin_posicion(&esperado[1]));
    }

    #[test]
    fn el_enlace_lleva_subrayado_debajo_de_la_base() {
        let lista = ordenes("<body style='margin:0'><p style='margin:0'>a <a href=x>enlace</a></p></body>");
        // Después del texto del enlace viene su raya, en el color del enlace (#0000ee).
        let i = lista
            .iter()
            .position(|o| matches!(o, Orden::Texto { texto, .. } if texto == "enlace"))
            .unwrap();
        let Orden::Texto { x, .. } = lista[i] else { unreachable!() };
        let Orden::Rectangulo { rect, color } = lista[i + 1] else {
            panic!("falta el subrayado: {:?}", lista.get(i + 1))
        };
        assert_eq!(color, css::Color::rgb(0, 0, 0xee));
        // Base en 13 + 1.6 = 14.6; la raya, un cuadrito (1 px a 16 px) más abajo, de 1 px.
        assert!((rect.y - 15.6).abs() < 0.01 && rect.alto == 1.0, "{rect:?}");
        assert_eq!(rect.x, x);
        assert!((rect.ancho - fuente::ancho_texto("enlace", 16.0, false)).abs() < 0.01);
        // El texto que no está subrayado no lleva raya.
        assert_eq!(rectangulos(&lista).len(), 1);
    }

    #[test]
    fn el_fondo_de_un_en_linea_va_antes_que_su_texto() {
        let lista = ordenes(
            "<body style='margin:0'><p style='margin:0'>x <span style='background:#0f0;             padding:0 4px'>y</span></p></body>",
        );
        let fondo = lista.iter().position(|o| matches!(o, Orden::Rectangulo { color, .. } if *color == VERDE));
        let texto = lista.iter().position(|o| matches!(o, Orden::Texto { texto, .. } if texto == "y"));
        assert!(fondo.unwrap() < texto.unwrap());
        let (rect, _) = rectangulos(&lista)[0];
        // Del alto de la celda (16), con la base en 14.6: de 14.6 − 13 = 1.6 a 17.6.
        let esperado = r(fuente::ancho_texto("x ", 16.0, false), 1.6, fuente::ancho_texto("y", 16.0, false) + 8.0, 16.0);
        let cerca = |a: f32, b: f32| (a - b).abs() < 0.01;
        assert!(
            cerca(rect.x, esperado.x)
                && cerca(rect.y, esperado.y)
                && cerca(rect.ancho, esperado.ancho)
                && cerca(rect.alto, esperado.alto),
            "{rect:?}"
        );
    }

    #[test]
    fn un_en_linea_con_borde_pinta_sus_cuatro_lados() {
        let lista = ordenes(
            "<body style='margin:0'><p style='margin:0'><span style='border:1px solid #00f'>y</span></p></body>",
        );
        let azules = rectangulos(&lista).iter().filter(|(_, c)| *c == AZUL).count();
        assert_eq!(azules, 4);
    }
}
