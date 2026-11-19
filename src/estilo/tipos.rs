//! Los tipos que entrega el motor de estilos: lo que el layout (fase 5) recibe.
//!
//! A diferencia de `css::Valor`, que es lo que alguien *escribió* (`2em`, `bold`, `inherit`),
//! aquí todo está *resuelto*: `2em` ya es `32px`, `bold` ya es `negrita: true`, y nada dice
//! `inherit`. El layout no vuelve a mirar el CSS.

use crate::css::Color;
use crate::html::dom::NodoId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    /// Ocupa su propia franja a todo lo ancho: `div`, `p`, `h1`.
    Bloque,
    /// Fluye dentro de una línea de texto: `b`, `a`, `span` y todo texto.
    EnLinea,
    /// No existe para el layout. Nunca aparece en el árbol estilado: esos nodos se eliminan.
    Ninguno,
}

/// Una medida que puede depender de algo que solo el layout conoce.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Longitud {
    /// "Decide tú": el layout la calcula (en `width`, ocupar el ancho disponible).
    Auto,
    Px(f32),
    /// Porcentaje del ancho del bloque contenedor; lo resuelve el layout.
    Porcentaje(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alineacion {
    Izquierda,
    Centro,
    Derecha,
}

/// Algo que existe una vez por cada lado de la caja, en el orden de CSS (como las agujas del
/// reloj desde arriba).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lados<T> {
    pub arriba: T,
    pub derecha: T,
    pub abajo: T,
    pub izquierda: T,
}

impl<T: Copy> Lados<T> {
    pub fn todos(valor: T) -> Self {
        Lados { arriba: valor, derecha: valor, abajo: valor, izquierda: valor }
    }

    /// Los cuatro valores en orden arriba, derecha, abajo, izquierda.
    pub fn en_orden(&self) -> [T; 4] {
        [self.arriba, self.derecha, self.abajo, self.izquierda]
    }

    pub fn desde_orden([arriba, derecha, abajo, izquierda]: [T; 4]) -> Self {
        Lados { arriba, derecha, abajo, izquierda }
    }
}

/// Los nombres de los lados como los escribe CSS, en el mismo orden que `Lados::en_orden`.
pub const LADOS_CSS: [&str; 4] = ["top", "right", "bottom", "left"];

/// Las propiedades finales ("valores computados") de un nodo.
#[derive(Debug, Clone, PartialEq)]
pub struct Estilo {
    pub display: Display,
    pub color: Color,
    pub fondo: Color,
    pub ancho: Longitud,
    pub alto: Longitud,
    /// `max-width`; su `none` se representa con `Auto` (sin límite).
    pub ancho_maximo: Longitud,
    pub margen: Lados<Longitud>,
    pub relleno: Lados<Longitud>,
    /// En px; ya vale 0 si el lado no tiene `border-style` o es `none`/`hidden`.
    pub borde_ancho: Lados<f32>,
    pub borde_color: Lados<Color>,
    /// En px.
    pub tam_fuente: f32,
    pub negrita: bool,
    pub cursiva: bool,
    /// `text-decoration: underline` propio o de algún ancestro.
    pub subrayado: bool,
    pub alineacion: Alineacion,
    /// En px.
    pub alto_linea: f32,
}

pub const NEGRO: Color = Color::rgb(0, 0, 0);
pub const TRANSPARENTE: Color = Color { r: 0, g: 0, b: 0, a: 0 };
/// El tamaño de letra de la raíz cuando nadie dice otra cosa (`medium`).
pub const TAM_FUENTE_INICIAL: f32 = 16.0;
/// `line-height: normal` es este factor por el tamaño de letra (lo que usan casi todas las
/// fuentes de escritorio).
pub const FACTOR_LINEA_NORMAL: f32 = 1.2;

impl Estilo {
    /// Los valores iniciales de CSS: lo que tiene un elemento al que ninguna regla le dice nada
    /// y que no hereda de nadie.
    pub fn inicial() -> Self {
        Estilo {
            display: Display::EnLinea,
            color: NEGRO,
            fondo: TRANSPARENTE,
            ancho: Longitud::Auto,
            alto: Longitud::Auto,
            ancho_maximo: Longitud::Auto,
            margen: Lados::todos(Longitud::Px(0.0)),
            relleno: Lados::todos(Longitud::Px(0.0)),
            borde_ancho: Lados::todos(0.0),
            borde_color: Lados::todos(NEGRO),
            tam_fuente: TAM_FUENTE_INICIAL,
            negrita: false,
            cursiva: false,
            subrayado: false,
            alineacion: Alineacion::Izquierda,
            alto_linea: TAM_FUENTE_INICIAL * FACTOR_LINEA_NORMAL,
        }
    }

    /// El estilo de un nodo de texto: hereda lo heredable de su padre y lo demás es inicial
    /// (un texto no tiene márgenes, ni fondo, ni bordes propios).
    pub fn para_texto(padre: &Estilo) -> Self {
        Estilo {
            color: padre.color,
            borde_color: Lados::todos(padre.color),
            tam_fuente: padre.tam_fuente,
            negrita: padre.negrita,
            cursiva: padre.cursiva,
            subrayado: padre.subrayado,
            alineacion: padre.alineacion,
            alto_linea: padre.alto_linea,
            ..Estilo::inicial()
        }
    }
}

/// Un nodo del árbol estilado: el DOM sin lo invisible y con su `Estilo` final.
#[derive(Debug, Clone, PartialEq)]
pub struct NodoEstilado {
    /// De qué nodo del DOM viene (para saber su etiqueta o sus atributos).
    pub nodo: NodoId,
    /// `Some` si es un nodo de texto. Se guarda tal cual, con sus espacios (los colapsa la fase 8).
    pub texto: Option<String>,
    pub estilo: Estilo,
    pub hijos: Vec<NodoEstilado>,
}
