//! UI: la ventana del sistema operativo y los eventos de mouse y teclado. Fases 1 y 11.
//!
//! Es el único módulo que conoce `winit`, `softbuffer` y el portapapeles de Windows. No sabe
//! qué es una página: le habla a cualquier cosa que cumpla el contrato `Pagina`.

mod desplazamiento;
mod portapapeles;

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{CursorIcon, Window, WindowId};

use crate::pintado::Lienzo;
use desplazamiento::{Desplazamiento, PASO_DE_LINEA};

/// Ancho de la barra de desplazamiento, en pixeles.
const ANCHO_BARRA: i32 = 10;
const COLOR_CANAL: u32 = 0xEDEFF2;
const COLOR_PULGAR: u32 = 0x9AA3AE;

/// Dos clics cuentan como doble clic si llegan así de seguidos y así de cerca (Windows usa
/// valores parecidos por defecto).
const TIEMPO_DOBLE_CLIC: Duration = Duration::from_millis(500);
const DISTANCIA_DOBLE_CLIC: f64 = 4.0;

/// Lo que la ventana necesita de lo que muestra. Las coordenadas `x`, `y` ya vienen en pixeles
/// de **página** (la `y` incluye el scroll). Los métodos que devuelven `bool` dicen si hay que
/// repintar. Todos menos `dibujar` tienen una versión por defecto que no hace nada, para que
/// una escena fija (como la de la fase 1) no tenga que implementarlos.
pub trait Pagina {
    /// Pinta con la página bajada `desplazamiento` pixeles y devuelve su alto total.
    fn dibujar(&mut self, lienzo: &mut Lienzo, desplazamiento: f32) -> f32;
    /// Botón izquierdo apretado. `clics` es 1, o 2 en un doble clic. `extender` es Shift.
    fn clic(&mut self, _x: f32, _y: f32, _clics: u32, _extender: bool) -> bool {
        false
    }
    /// El mouse se movió con el botón apretado.
    fn arrastrar(&mut self, _x: f32, _y: f32) -> bool {
        false
    }
    fn seleccionar_todo(&mut self) -> bool {
        false
    }
    fn deseleccionar(&mut self) -> bool {
        false
    }
    fn texto_seleccionado(&self) -> Option<String> {
        None
    }
    fn hay_texto_en(&self, _x: f32, _y: f32) -> bool {
        false
    }
}

/// Cualquier función `fn(&mut Lienzo)` (las escenas de prueba) sirve como página fija.
impl Pagina for fn(&mut Lienzo) {
    fn dibujar(&mut self, lienzo: &mut Lienzo, _desplazamiento: f32) -> f32 {
        self(lienzo);
        lienzo.alto() as f32
    }
}

/// Abre la ventana y se queda atendiendo eventos hasta que el usuario la cierra.
pub fn abrir<P: Pagina>(titulo: &str, pagina: P) -> Result<(), Box<dyn std::error::Error>> {
    let bucle = EventLoop::new()?;
    let mut app = App {
        titulo: titulo.to_string(),
        pagina,
        ventana: None,
        desplazamiento: Desplazamiento::default(),
        shift: false,
        ctrl: false,
        mouse: (0.0, 0.0),
        arrastrando: false,
        ultimo_clic: None,
        clics: 0,
    };
    bucle.run_app(&mut app)?;
    Ok(())
}

struct App<P> {
    titulo: String,
    pagina: P,
    ventana: Option<Ventana>,
    desplazamiento: Desplazamiento,
    shift: bool,
    ctrl: bool,
    /// Dónde está el mouse, en pixeles de la ventana.
    mouse: (f64, f64),
    /// Si el botón izquierdo sigue apretado desde un clic sobre la página.
    arrastrando: bool,
    /// Cuándo y dónde fue el último clic, para reconocer el doble clic.
    ultimo_clic: Option<(Instant, (f64, f64))>,
    clics: u32,
}

struct Ventana {
    ventana: Rc<Window>,
    superficie: Surface<Rc<Window>, Rc<Window>>,
}

impl<P: Pagina> ApplicationHandler for App<P> {
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

    /// Llega un evento de la ventana: cerrar, cambiar de tamaño, repintar, mouse, teclas...
    fn window_event(&mut self, bucle: &ActiveEventLoop, _id: WindowId, evento: WindowEvent) {
        if self.ventana.is_none() {
            return;
        }
        let repintar = match evento {
            WindowEvent::CloseRequested => {
                bucle.exit();
                false
            }
            WindowEvent::Resized(_) => true,
            WindowEvent::RedrawRequested => {
                self.repintar();
                false
            }
            WindowEvent::ModifiersChanged(modificadores) => {
                self.shift = modificadores.state().shift_key();
                self.ctrl = modificadores.state().control_key();
                false
            }
            // La rueda llega en "líneas" (la mayoría de ratones) o en pixeles (touchpads).
            // Hacia arriba es positivo, por eso se le cambia el signo.
            WindowEvent::MouseWheel { delta, .. } => {
                let pixeles = match delta {
                    MouseScrollDelta::LineDelta(_, lineas) => -lineas * PASO_DE_LINEA,
                    MouseScrollDelta::PixelDelta(p) => -p.y as f32,
                };
                let movio = self.desplazamiento.mover(pixeles);
                // Si se está arrastrando, la selección sigue al mouse por la página que se movió.
                let (x, y) = self.en_pagina();
                movio | (self.arrastrando && self.pagina.arrastrar(x, y))
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse = (position.x, position.y);
                let (x, y) = self.en_pagina();
                let icono =
                    if self.pagina.hay_texto_en(x, y) { CursorIcon::Text } else { CursorIcon::Default };
                if let Some(v) = &self.ventana {
                    v.ventana.set_cursor(icono);
                }
                self.arrastrando && self.pagina.arrastrar(x, y)
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => match state {
                ElementState::Pressed => {
                    self.clics = self.contar_clics();
                    self.arrastrando = true;
                    let (x, y) = self.en_pagina();
                    let marco = self.pagina.clic(x, y, self.clics, self.shift);
                    marco || self.pagina.deseleccionar()
                }
                ElementState::Released => {
                    self.arrastrando = false;
                    false
                }
            },
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.tecla(&event.logical_key)
            }
            _ => false,
        };
        if repintar {
            if let Some(v) = &self.ventana {
                v.ventana.request_redraw();
            }
        }
    }
}

impl<P: Pagina> App<P> {
    /// El mouse en pixeles de página: la `y` de la ventana más lo que está bajada la página.
    fn en_pagina(&self) -> (f32, f32) {
        (self.mouse.0 as f32, self.mouse.1 as f32 + self.desplazamiento.posicion())
    }

    /// 2 si este clic llega rápido y cerca del anterior; si no, 1.
    fn contar_clics(&mut self) -> u32 {
        let ahora = Instant::now();
        let doble = self.ultimo_clic.is_some_and(|(cuando, (x, y))| {
            ahora - cuando < TIEMPO_DOBLE_CLIC
                && (x - self.mouse.0).abs() <= DISTANCIA_DOBLE_CLIC
                && (y - self.mouse.1).abs() <= DISTANCIA_DOBLE_CLIC
        });
        // Después de un doble clic, el siguiente vuelve a ser simple.
        self.ultimo_clic = if doble { None } else { Some((ahora, self.mouse)) };
        if doble { 2 } else { 1 }
    }

    fn tecla(&mut self, tecla: &Key) -> bool {
        if self.ctrl {
            if let Key::Character(c) = tecla {
                match c.to_lowercase().as_str() {
                    "a" => return self.pagina.seleccionar_todo(),
                    "c" => {
                        if let Some(texto) = self.pagina.texto_seleccionado() {
                            if let Err(error) = portapapeles::copiar(&texto) {
                                eprintln!("No se pudo copiar: {error}");
                            }
                        }
                        return false;
                    }
                    _ => {}
                }
            }
        }
        let d = &mut self.desplazamiento;
        match tecla {
            Key::Named(NamedKey::ArrowDown) => d.mover(PASO_DE_LINEA),
            Key::Named(NamedKey::ArrowUp) => d.mover(-PASO_DE_LINEA),
            Key::Named(NamedKey::PageDown) => d.pagina(true),
            Key::Named(NamedKey::PageUp) => d.pagina(false),
            Key::Named(NamedKey::Space) => d.pagina(!self.shift),
            Key::Named(NamedKey::Home) => d.inicio(),
            Key::Named(NamedKey::End) => d.fin(),
            Key::Named(NamedKey::Escape) => self.pagina.deseleccionar(),
            _ => false,
        }
    }

    fn repintar(&mut self) {
        let Some(v) = self.ventana.as_mut() else { return };
        let tam = v.ventana.inner_size();
        // Una ventana minimizada mide 0x0: no hay nada que pintar.
        let (Some(ancho), Some(alto)) = (NonZeroU32::new(tam.width), NonZeroU32::new(tam.height))
        else {
            return;
        };
        v.superficie.resize(ancho, alto).expect("no se pudo redimensionar");

        let (w, h) = (tam.width as usize, tam.height as usize);
        let mut lienzo = Lienzo::nuevo(w, h, 0xFFFFFF);
        let alto_contenido = self.pagina.dibujar(&mut lienzo, self.desplazamiento.posicion());

        // Con las medidas nuevas, la posición pudo quedar fuera de rango (por ejemplo, al
        // ensanchar la ventana la página se acorta). Si se corrigió, se pinta otra vez.
        let antes = self.desplazamiento.posicion();
        self.desplazamiento.medidas(alto_contenido, h as f32);
        if self.desplazamiento.posicion() != antes {
            lienzo = Lienzo::nuevo(w, h, 0xFFFFFF);
            self.pagina.dibujar(&mut lienzo, self.desplazamiento.posicion());
        }
        pintar_barra(&mut lienzo, &self.desplazamiento);

        let mut bufer = v.superficie.buffer_mut().expect("no hay búfer de pixeles");
        bufer.copy_from_slice(lienzo.pixeles());
        bufer.present().expect("no se pudo mostrar el búfer");
    }
}

/// La barra de la derecha: un canal claro y el pulgar gris. Se pinta encima de la página.
fn pintar_barra(lienzo: &mut Lienzo, desplazamiento: &Desplazamiento) {
    let Some((y, alto)) = desplazamiento.pulgar() else { return };
    let x = lienzo.ancho() as i32 - ANCHO_BARRA;
    lienzo.rectangulo(x, 0, ANCHO_BARRA, lienzo.alto() as i32, COLOR_CANAL);
    lienzo.rectangulo(x + 2, y.round() as i32, ANCHO_BARRA - 4, alto.round() as i32, COLOR_PULGAR);
}
