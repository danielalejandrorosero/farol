//! El scroll: cuántos pixeles está bajada la página y hasta dónde puede bajar.
//!
//! La página nunca se mueve de verdad. Es como una hoja larga detrás de una ventana de papel:
//! "bajar" es correr la hoja hacia arriba. `posicion` dice qué fila de la página queda pegada
//! al borde de arriba de la ventana.

/// Pixeles que baja una "línea" de la rueda del mouse o una flecha del teclado.
pub const PASO_DE_LINEA: f32 = 48.0;

/// Fracción de la ventana que baja `AvPág`: se deja un pedazo visible para no perder el hilo.
const FRACCION_DE_PAGINA: f32 = 0.9;

#[derive(Debug, Default)]
pub struct Desplazamiento {
    posicion: f32,
    alto_contenido: f32,
    alto_ventana: f32,
}

impl Desplazamiento {
    pub fn posicion(&self) -> f32 {
        self.posicion
    }

    /// Lo más que se puede bajar: hasta que el final de la página toque el borde de abajo.
    pub fn maximo(&self) -> f32 {
        (self.alto_contenido - self.alto_ventana).max(0.0)
    }

    /// Después de cada repintado se sabe cuánto mide la página y la ventana. Si la ventana creció
    /// o la página se acortó (al ensancharla, el texto ocupa menos líneas), se corrige la posición.
    pub fn medidas(&mut self, alto_contenido: f32, alto_ventana: f32) {
        self.alto_contenido = alto_contenido;
        self.alto_ventana = alto_ventana;
        self.posicion = self.posicion.clamp(0.0, self.maximo());
    }

    /// Mueve la página `pixeles` (positivo = bajar). Devuelve si cambió algo, para no repintar
    /// de gusto cuando ya se está en el tope o en el fondo.
    pub fn mover(&mut self, pixeles: f32) -> bool {
        self.ir_a(self.posicion + pixeles)
    }

    pub fn pagina(&mut self, hacia_abajo: bool) -> bool {
        let salto = self.alto_ventana * FRACCION_DE_PAGINA;
        self.mover(if hacia_abajo { salto } else { -salto })
    }

    pub fn inicio(&mut self) -> bool {
        self.ir_a(0.0)
    }

    pub fn fin(&mut self) -> bool {
        self.ir_a(self.maximo())
    }

    fn ir_a(&mut self, destino: f32) -> bool {
        let nueva = destino.clamp(0.0, self.maximo());
        let cambio = nueva != self.posicion;
        self.posicion = nueva;
        cambio
    }

    /// La barra de desplazamiento: dónde va el "pulgar" (la parte que se arrastra) y cuánto mide,
    /// en pixeles de la ventana. `None` si la página cabe entera y no hace falta barra.
    pub fn pulgar(&self) -> Option<(f32, f32)> {
        if self.maximo() <= 0.0 {
            return None;
        }
        // El pulgar mide lo que la ventana es de la página: si se ve un cuarto, mide un cuarto.
        let alto = (self.alto_ventana * self.alto_ventana / self.alto_contenido).max(24.0);
        let recorrido = self.alto_ventana - alto;
        Some((recorrido * self.posicion / self.maximo(), alto))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn con(contenido: f32, ventana: f32) -> Desplazamiento {
        let mut d = Desplazamiento::default();
        d.medidas(contenido, ventana);
        d
    }

    #[test]
    fn no_se_puede_subir_mas_alla_del_tope() {
        let mut d = con(2000.0, 600.0);
        assert!(!d.mover(-100.0));
        assert_eq!(d.posicion(), 0.0);
    }

    #[test]
    fn no_se_puede_bajar_mas_alla_del_final() {
        let mut d = con(2000.0, 600.0);
        assert!(d.mover(5000.0));
        assert_eq!(d.posicion(), 1400.0);
        assert!(!d.mover(10.0));
    }

    #[test]
    fn una_pagina_que_cabe_no_se_mueve_ni_tiene_barra() {
        let mut d = con(500.0, 600.0);
        assert!(!d.mover(100.0));
        assert_eq!(d.pulgar(), None);
    }

    #[test]
    fn avpag_baja_casi_una_ventana() {
        let mut d = con(5000.0, 600.0);
        d.pagina(true);
        assert_eq!(d.posicion(), 540.0);
        d.pagina(false);
        assert_eq!(d.posicion(), 0.0);
    }

    #[test]
    fn inicio_y_fin() {
        let mut d = con(3000.0, 1000.0);
        d.fin();
        assert_eq!(d.posicion(), 2000.0);
        d.inicio();
        assert_eq!(d.posicion(), 0.0);
    }

    #[test]
    fn agrandar_la_ventana_corrige_la_posicion() {
        let mut d = con(2000.0, 600.0);
        d.fin();
        d.medidas(2000.0, 1500.0);
        assert_eq!(d.posicion(), 500.0);
    }

    #[test]
    fn el_pulgar_mide_la_parte_visible_y_baja_con_la_pagina() {
        let mut d = con(2400.0, 600.0);
        assert_eq!(d.pulgar(), Some((0.0, 150.0)));
        d.fin();
        assert_eq!(d.pulgar(), Some((450.0, 150.0)));
    }
}
