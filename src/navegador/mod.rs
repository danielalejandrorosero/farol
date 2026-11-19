//! Navegador: una página abierta en la ventana. Une la tubería (estilos → layout → pintado) con
//! lo que hace la persona: scroll, seleccionar texto y copiarlo.
//!
//! `ui` solo sabe de ventanas y eventos; `layout` y `pintado` solo saben de cajas y pixeles.
//! Este módulo es el que conoce a ambos lados, como el controlador de un MVC.

pub mod seleccion;

use crate::css::Color;
use crate::estilo::NodoEstilado;
use crate::pintado::{Lienzo, Orden};
use crate::{layout, pintado, ui};
use seleccion::{Punto, TextoPlano};

/// El azul de la selección, semitransparente para que el texto se siga viendo debajo.
const AZUL_SELECCION: Color = Color { r: 51, g: 144, b: 255, a: 96 };

pub struct VistaPagina {
    estilado: NodoEstilado,
    /// Los fragmentos de texto del último layout, para saber qué letra hay bajo el mouse.
    textos: Vec<TextoPlano>,
    /// El ancho con el que se hizo el último layout. Si cambia, los fragmentos se mueven y la
    /// selección deja de tener sentido: se borra.
    ultimo_ancho: usize,
    /// Dónde empezó el arrastre (ancla) y dónde va (foco).
    seleccion: Option<(Punto, Punto)>,
}

impl VistaPagina {
    pub fn nueva(estilado: NodoEstilado) -> Self {
        Self { estilado, textos: Vec::new(), ultimo_ancho: 0, seleccion: None }
    }
}

impl ui::Pagina for VistaPagina {
    fn dibujar(&mut self, lienzo: &mut Lienzo, desplazamiento: f32) -> f32 {
        let cajas = layout::calcular(&self.estilado, lienzo.ancho() as f32);
        if lienzo.ancho() != self.ultimo_ancho {
            self.ultimo_ancho = lienzo.ancho();
            self.seleccion = None;
        }
        self.textos = seleccion::aplanar(&cajas);

        let mut ordenes = pintado::lista_de_pintado(&cajas);
        if let Some((a, b)) = self.seleccion {
            for rect in seleccion::rectangulos(&self.textos, a, b) {
                ordenes.push(Orden::Rectangulo { rect, color: AZUL_SELECCION });
            }
        }
        lienzo.ejecutar_desplazado(&ordenes, desplazamiento);
        cajas.dimensiones.contenido.alto
    }

    fn clic(&mut self, x: f32, y: f32, clics: u32, extender: bool) -> bool {
        let Some(punto) = seleccion::punto_en(&self.textos, x, y) else { return false };
        self.seleccion = Some(match (clics, self.seleccion) {
            (1, Some((ancla, _))) if extender => (ancla, punto),
            (1, _) => (punto, punto),
            _ => seleccion::palabra_en(&self.textos, punto),
        });
        true
    }

    fn arrastrar(&mut self, x: f32, y: f32) -> bool {
        let (Some((ancla, _)), Some(foco)) = (self.seleccion, seleccion::punto_en(&self.textos, x, y))
        else {
            return false;
        };
        let cambio = self.seleccion != Some((ancla, foco));
        self.seleccion = Some((ancla, foco));
        cambio
    }

    fn seleccionar_todo(&mut self) -> bool {
        self.seleccion = seleccion::todo(&self.textos);
        self.seleccion.is_some()
    }

    fn deseleccionar(&mut self) -> bool {
        self.seleccion.take().is_some()
    }

    fn texto_seleccionado(&self) -> Option<String> {
        let (a, b) = self.seleccion?;
        let texto = seleccion::texto(&self.textos, a, b);
        (!texto.is_empty()).then_some(texto)
    }

    fn hay_texto_en(&self, x: f32, y: f32) -> bool {
        seleccion::hay_texto_en(&self.textos, x, y)
    }
}
