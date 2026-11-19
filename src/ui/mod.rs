//! UI: la ventana del sistema operativo y los eventos de mouse y teclado. Fases 1 y 11.
//!
//! Es el único módulo que conoce `winit` y `softbuffer`.

use std::num::NonZeroU32;
use std::rc::Rc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::pintado::Lienzo;

/// Abre la ventana y se queda atendiendo eventos hasta que el usuario la cierra.
/// `dibujar` se llama cada vez que hay que repintar (al abrir, al redimensionar), con un lienzo
/// del tamaño actual.
///
/// `dibujar` es un *cierre* (`FnMut`): una función que puede llevar datos consigo, como una
/// función de JavaScript que captura variables de afuera. Así la página (su DOM y sus estilos)
/// viaja dentro de `dibujar` y `ui` no necesita saber qué es una página. `FnMut` quiere decir que
/// puede modificar lo que lleva entre una llamada y otra. Una función común (`fn`) también sirve.
pub fn abrir<F: FnMut(&mut Lienzo)>(
    titulo: &str,
    dibujar: F,
) -> Result<(), Box<dyn std::error::Error>> {
    let bucle = EventLoop::new()?;
    let mut app = App { titulo: titulo.to_string(), dibujar, ventana: None };
    bucle.run_app(&mut app)?;
    Ok(())
}

struct App<F> {
    titulo: String,
    dibujar: F,
    ventana: Option<Ventana>,
}

struct Ventana {
    ventana: Rc<Window>,
    superficie: Surface<Rc<Window>, Rc<Window>>,
}

impl<F: FnMut(&mut Lienzo)> ApplicationHandler for App<F> {
    /// El sistema operativo avisa que ya se puede crear la ventana.
    fn resumed(&mut self, bucle: &ActiveEventLoop) {
        if self.ventana.is_some() {
            return;
        }
        let atributos = Window::default_attributes()
            .with_title(&self.titulo)
            .with_inner_size(LogicalSize::new(900.0, 600.0));
        let ventana = Rc::new(bucle.create_window(atributos).expect("no se pudo crear la ventana"));
        let contexto = Context::new(ventana.clone()).expect("no se pudo conectar con la pantalla");
        let superficie =
            Surface::new(&contexto, ventana.clone()).expect("no se pudo crear la superficie");
        self.ventana = Some(Ventana { ventana, superficie });
    }

    /// Llega un evento de la ventana: cerrar, cambiar de tamaño, repintar...
    fn window_event(&mut self, bucle: &ActiveEventLoop, _id: WindowId, evento: WindowEvent) {
        let Some(v) = self.ventana.as_mut() else { return };
        match evento {
            WindowEvent::CloseRequested => bucle.exit(),
            WindowEvent::Resized(_) => v.ventana.request_redraw(),
            WindowEvent::RedrawRequested => {
                let tam = v.ventana.inner_size();
                // Una ventana minimizada mide 0x0: no hay nada que pintar.
                let (Some(ancho), Some(alto)) =
                    (NonZeroU32::new(tam.width), NonZeroU32::new(tam.height))
                else {
                    return;
                };
                v.superficie.resize(ancho, alto).expect("no se pudo redimensionar");

                let mut lienzo = Lienzo::nuevo(tam.width as usize, tam.height as usize, 0xFFFFFF);
                (self.dibujar)(&mut lienzo);

                let mut bufer = v.superficie.buffer_mut().expect("no hay búfer de pixeles");
                bufer.copy_from_slice(lienzo.pixeles());
                bufer.present().expect("no se pudo mostrar el búfer");
            }
            _ => {}
        }
    }
}
