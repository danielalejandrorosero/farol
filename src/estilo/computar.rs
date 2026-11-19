//! Valores computados: de "lo que ganó en la cascada" (`2em`, `bold`, `inherit`) a números
//! listos para el layout (`32px`, `negrita: true`).
//!
//! Para cada propiedad hay tres preguntas:
//!
//! 1. **¿Alguien la declaró con un valor que Farol entiende?** Se recorren las candidatas de la
//!    más fuerte a la más débil y se toma la primera interpretable. Un valor desconocido se
//!    salta, como si la declaración no existiera.
//! 2. **Si nadie la declaró, ¿se hereda?** Las propiedades de texto (`color`, `font-*`,
//!    `text-align`, `line-height`) pasan de padre a hijo; las de la caja (márgenes, bordes,
//!    fondo, ancho) no: arrancan en su valor inicial.
//! 3. **¿Depende de algo?** `em` depende del tamaño de letra, y `currentcolor` del color. Por eso
//!    se calcula primero `font-size` (con el del padre), después `color`, y al final lo demás.

use super::cascada::Especificados;
use super::tipos::{
    Alineacion, Display, Estilo, FACTOR_LINEA_NORMAL, LADOS_CSS, Lados, Longitud, NEGRO,
    TAM_FUENTE_INICIAL, TRANSPARENTE,
};
use crate::css::{Color, Unidad, Valor};

/// `line-height` tal como se hereda. Con un número (`1.5`) lo que pasa a los hijos es el
/// **factor**, no los px: un hijo con letra más grande tiene líneas proporcionalmente más altas.
/// Con una longitud (`24px`, `1.5em`) lo que pasa son los px ya calculados en el padre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AltoLinea {
    Factor(f32),
    Px(f32),
}

/// Lo que un elemento le pasa a sus hijos: su estilo y la forma de su `line-height`, que el
/// `Estilo` del contrato no guarda (solo guarda los px).
#[derive(Debug, Clone)]
pub struct Calculado {
    pub estilo: Estilo,
    pub linea: AltoLinea,
}

impl Calculado {
    /// El nodo raíz del documento: todo inicial, salvo que es un bloque (es la "hoja de papel"
    /// donde va toda la página).
    pub fn raiz() -> Self {
        let estilo = Estilo { display: Display::Bloque, ..Estilo::inicial() };
        Calculado { estilo, linea: AltoLinea::Factor(FACTOR_LINEA_NORMAL) }
    }
}

/// Lo que dijo la cascada sobre una propiedad, ya interpretado.
enum Elegido<T> {
    Valor(T),
    Inherit,
    Initial,
    Unset,
    /// Nadie la declaró (o nada de lo declarado se entendió).
    Nada,
}

/// La primera candidata que se entiende, de la más fuerte a la más débil.
fn elegir<'a, T>(
    valores: impl Iterator<Item = &'a Valor>,
    interpretar: impl Fn(&Valor) -> Option<T>,
) -> Elegido<T> {
    for valor in valores {
        match valor {
            Valor::Palabra(p) if p == "inherit" => return Elegido::Inherit,
            Valor::Palabra(p) if p == "initial" => return Elegido::Initial,
            Valor::Palabra(p) if p == "unset" => return Elegido::Unset,
            otro => {
                if let Some(t) = interpretar(otro) {
                    return Elegido::Valor(t);
                }
            }
        }
    }
    Elegido::Nada
}

/// Resuelve una propiedad que se hereda: sin declaración, vale lo del padre.
fn heredada<T>(elegido: Elegido<T>, padre: T, inicial: T) -> T {
    match elegido {
        Elegido::Valor(t) => t,
        Elegido::Initial => inicial,
        Elegido::Inherit | Elegido::Unset | Elegido::Nada => padre,
    }
}

/// Resuelve una propiedad que no se hereda: sin declaración, vale su inicial.
fn propia<T>(elegido: Elegido<T>, padre: T, inicial: T) -> T {
    match elegido {
        Elegido::Valor(t) => t,
        Elegido::Inherit => padre,
        Elegido::Initial | Elegido::Unset | Elegido::Nada => inicial,
    }
}

fn palabra(valor: &Valor) -> Option<&str> {
    match valor {
        Valor::Palabra(p) => Some(p),
        _ => None,
    }
}

/// Calcula el estilo de un elemento a partir de lo que dijo la cascada y de lo que hereda.
pub fn computar(esp: &Especificados, padre: &Calculado) -> Calculado {
    let p = &padre.estilo;

    // 1. El tamaño de letra: todo lo demás en `em` depende de él.
    let tam = heredada(
        elegir(esp.valores("font-size"), |v| tam_fuente(v, p.tam_fuente)),
        p.tam_fuente,
        TAM_FUENTE_INICIAL,
    );
    // 2. El color: `currentcolor` en bordes y fondo depende de él. En `color` mismo,
    //    `currentcolor` significa "el color que traigo", o sea, el del padre.
    let color = heredada(elegir(esp.valores("color"), |v| un_color(v, p.color)), p.color, NEGRO);

    let display = propia(elegir(esp.valores("display"), un_display), p.display, Display::EnLinea);
    let fondo =
        propia(elegir(esp.valores("background-color"), |v| un_color(v, color)), p.fondo, TRANSPARENTE);

    let caja = |propiedad: &str, del_padre: Longitud, acepta_auto: bool| {
        let interpretar = |v: &Valor| longitud(v, tam).filter(|l| acepta_auto || *l != Longitud::Auto);
        propia(elegir(esp.valores(propiedad), interpretar), del_padre, Longitud::Auto)
    };
    let ancho = caja("width", p.ancho, true);
    let alto = caja("height", p.alto, true);
    let ancho_maximo = {
        let interpretar = |v: &Valor| match v {
            Valor::Palabra(n) if n == "none" => Some(Longitud::Auto),
            Valor::Palabra(_) => None,
            otro => longitud(otro, tam),
        };
        propia(elegir(esp.valores("max-width"), interpretar), p.ancho_maximo, Longitud::Auto)
    };

    let por_lado = |prefijo: &str, padre: Lados<Longitud>, acepta_auto: bool| {
        let padre = padre.en_orden();
        Lados::desde_orden(std::array::from_fn(|i| {
            let interpretar =
                |v: &Valor| longitud(v, tam).filter(|l| acepta_auto || *l != Longitud::Auto);
            let propiedad = format!("{prefijo}-{}", LADOS_CSS[i]);
            propia(elegir(esp.valores(&propiedad), interpretar), padre[i], Longitud::Px(0.0))
        }))
    };
    let margen = por_lado("margin", p.margen, true);
    let relleno = por_lado("padding", p.relleno, false);

    let (borde_ancho, borde_color) = bordes(esp, p, tam, color);

    let negrita = heredada(elegir(esp.valores("font-weight"), peso), p.negrita, false);
    let cursiva = heredada(
        elegir(esp.valores("font-style"), |v| match palabra(v)? {
            "italic" | "oblique" => Some(true),
            "normal" => Some(false),
            _ => None,
        }),
        p.cursiva,
        false,
    );
    let alineacion = heredada(
        elegir(esp.valores("text-align"), |v| match palabra(v)? {
            // `justify` necesita repartir espacios entre palabras (fase 8): por ahora, izquierda.
            "left" | "start" | "justify" => Some(Alineacion::Izquierda),
            "center" | "-webkit-center" => Some(Alineacion::Centro),
            "right" | "end" => Some(Alineacion::Derecha),
            _ => None,
        }),
        p.alineacion,
        Alineacion::Izquierda,
    );

    // El subrayado no se hereda en CSS, pero se *propaga*: la raya del `<a>` cruza también el
    // texto de un `<b>` que vaya adentro, y el hijo no puede quitarla. Por eso es un "o".
    let subrayado_propio = |propiedad: &str| {
        propia(
            elegir(esp.valores(propiedad), |v| match v {
                Valor::Palabra(p) => Some(p.split_whitespace().any(|w| w == "underline")),
                Valor::Color(_) => Some(false), // `text-decoration: red` no trae línea.
                _ => None,
            }),
            false,
            false,
        )
    };
    let subrayado =
        p.subrayado || subrayado_propio("text-decoration") || subrayado_propio("text-decoration-line");

    let linea = heredada(
        elegir(esp.valores("line-height"), |v| alto_linea(v, tam)),
        padre.linea,
        AltoLinea::Factor(FACTOR_LINEA_NORMAL),
    );
    let alto_linea = match linea {
        AltoLinea::Factor(f) => f * tam,
        AltoLinea::Px(px) => px,
    };

    let estilo = Estilo {
        display,
        color,
        fondo,
        ancho,
        alto,
        ancho_maximo,
        margen,
        relleno,
        borde_ancho,
        borde_color,
        tam_fuente: tam,
        negrita,
        cursiva,
        subrayado,
        alineacion,
        alto_linea,
    };
    Calculado { estilo, linea }
}

/// Ancho y color de los cuatro bordes. Un borde sin estilo (o `none`/`hidden`) mide 0, aunque
/// tenga ancho declarado: `border-width: 5px` solo no dibuja nada.
fn bordes(esp: &Especificados, p: &Estilo, tam: f32, color: Color) -> (Lados<f32>, Lados<Color>) {
    let anchos_padre = p.borde_ancho.en_orden();
    let colores_padre = p.borde_color.en_orden();
    let mut anchos = [0.0; 4];
    let mut colores = [color; 4];
    for (i, lado) in LADOS_CSS.iter().enumerate() {
        // Estilo: solo importa si se ve o no. Con `inherit`, se ve si el del padre se ve.
        let visible = propia(
            elegir(esp.valores(&format!("border-{lado}-style")), |v| match palabra(v)? {
                "none" | "hidden" => Some(false),
                "dotted" | "dashed" | "solid" | "double" | "groove" | "ridge" | "inset"
                | "outset" => Some(true),
                _ => None,
            }),
            anchos_padre[i] > 0.0,
            false,
        );
        let ancho = propia(
            elegir(esp.valores(&format!("border-{lado}-width")), |v| ancho_de_borde(v, tam)),
            anchos_padre[i],
            3.0, // `medium`
        );
        anchos[i] = if visible { ancho } else { 0.0 };
        colores[i] = propia(
            elegir(esp.valores(&format!("border-{lado}-color")), |v| un_color(v, color)),
            colores_padre[i],
            color, // el inicial es `currentcolor`
        );
    }
    (Lados::desde_orden(anchos), Lados::desde_orden(colores))
}

/// `font-size` en px. `em` y `%` son relativos al tamaño del **padre** (si no, `2em` dependería
/// de sí mismo).
fn tam_fuente(valor: &Valor, padre: f32) -> Option<f32> {
    match valor {
        Valor::Longitud(n, Unidad::Px) => Some(*n),
        Valor::Longitud(n, Unidad::Em) => Some(n * padre),
        Valor::Longitud(n, Unidad::Porcentaje) => Some(n / 100.0 * padre),
        // Las palabras clave siguen la tabla de Chrome con `medium` = 16px.
        Valor::Palabra(p) => Some(match p.as_str() {
            "xx-small" => 9.0,
            "x-small" => 10.0,
            "small" => 13.0,
            "medium" => 16.0,
            "large" => 18.0,
            "x-large" => 24.0,
            "xx-large" => 32.0,
            "xxx-large" => 48.0,
            "larger" => padre * 1.2,
            "smaller" => padre / 1.2,
            _ => return None,
        }),
        _ => None,
    }
}

/// Un color, o `currentcolor` reemplazado por `actual`.
fn un_color(valor: &Valor, actual: Color) -> Option<Color> {
    match valor {
        Valor::Color(c) => Some(*c),
        Valor::Palabra(p) if p == "currentcolor" => Some(actual),
        _ => None,
    }
}

/// `display`. Farol solo distingue bloque, en línea y nada: las demás formas se acercan a la
/// más parecida (deuda: `flex`, `grid`, tablas e `inline-block` se dibujan sin sus reglas).
fn un_display(valor: &Valor) -> Option<Display> {
    match palabra(valor)? {
        "block" | "list-item" | "flex" | "grid" | "flow-root" | "table" | "table-row-group"
        | "table-header-group" | "table-footer-group" | "table-row" | "table-cell"
        | "table-caption" => Some(Display::Bloque),
        "inline" | "inline-block" | "inline-flex" | "inline-grid" | "inline-table" => {
            Some(Display::EnLinea)
        }
        "none" => Some(Display::Ninguno),
        _ => None,
    }
}

/// Una medida de caja: px, `em` (con el tamaño de letra **propio**), `%` (lo resuelve el
/// layout) o `auto`.
fn longitud(valor: &Valor, tam: f32) -> Option<Longitud> {
    match valor {
        Valor::Longitud(n, Unidad::Px) => Some(Longitud::Px(*n)),
        Valor::Longitud(n, Unidad::Em) => Some(Longitud::Px(n * tam)),
        Valor::Longitud(n, Unidad::Porcentaje) => Some(Longitud::Porcentaje(*n)),
        Valor::Palabra(p) if p == "auto" => Some(Longitud::Auto),
        _ => None,
    }
}

fn ancho_de_borde(valor: &Valor, tam: f32) -> Option<f32> {
    match valor {
        Valor::Palabra(p) => match p.as_str() {
            "thin" => Some(1.0),
            "medium" => Some(3.0),
            "thick" => Some(5.0),
            _ => None,
        },
        Valor::Longitud(n, Unidad::Px) => Some(*n),
        Valor::Longitud(n, Unidad::Em) => Some(n * tam),
        _ => None,
    }
}

/// `font-weight`: Farol solo tiene normal y negrita. 600 o más es negrita.
fn peso(valor: &Valor) -> Option<bool> {
    match valor {
        Valor::Numero(n) if (1.0..=1000.0).contains(n) => Some(*n >= 600.0),
        Valor::Palabra(p) => match p.as_str() {
            // `bolder` siempre llega a negrita y `lighter` siempre baja a normal: con solo dos
            // pesos, "uno más" y "uno menos" no tienen otra opción.
            "bold" | "bolder" => Some(true),
            "normal" | "lighter" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn alto_linea(valor: &Valor, tam: f32) -> Option<AltoLinea> {
    match valor {
        Valor::Palabra(p) if p == "normal" => Some(AltoLinea::Factor(FACTOR_LINEA_NORMAL)),
        Valor::Numero(n) if *n >= 0.0 => Some(AltoLinea::Factor(*n)),
        Valor::Longitud(n, Unidad::Px) if *n >= 0.0 => Some(AltoLinea::Px(*n)),
        Valor::Longitud(n, Unidad::Em) if *n >= 0.0 => Some(AltoLinea::Px(n * tam)),
        Valor::Longitud(n, Unidad::Porcentaje) if *n >= 0.0 => Some(AltoLinea::Px(n / 100.0 * tam)),
        _ => None,
    }
}
