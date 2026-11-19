//! El tokenizador: recorre el texto carácter por carácter y lo corta en piezas con sentido.
//!
//! `<p class="x">hola</p>` se convierte en: `Abre p {class=x}`, `Texto "hola"`, `Cierra p`.

#[derive(Debug, PartialEq)]
pub enum Token {
    Doctype,
    Abre { nombre: String, atributos: Vec<(String, String)> },
    Cierra { nombre: String },
    Texto(String),
    Comentario(String),
}

/// Etiquetas cuyo contenido no es HTML: lo que hay adentro se toma como texto tal cual,
/// aunque contenga `<`. Por ejemplo, el CSS de `<style>` o el código de `<script>`.
const TEXTO_CRUDO: [&str; 4] = ["style", "script", "title", "textarea"];

pub fn tokenizar(html: &str) -> Vec<Token> {
    let mut t = Tokenizador { c: html.chars().collect(), pos: 0, tokens: Vec::new() };
    t.correr();
    t.tokens
}

struct Tokenizador {
    c: Vec<char>,
    pos: usize,
    tokens: Vec<Token>,
}

impl Tokenizador {
    fn correr(&mut self) {
        let mut texto = String::new();
        while let Some(&actual) = self.c.get(self.pos) {
            if actual == '<' {
                if let Some(token) = self.etiqueta() {
                    self.vaciar_texto(&mut texto);
                    let crudo = match &token {
                        Token::Abre { nombre, .. } if TEXTO_CRUDO.contains(&nombre.as_str()) => {
                            Some(nombre.clone())
                        }
                        _ => None,
                    };
                    self.tokens.push(token);
                    if let Some(nombre) = crudo {
                        self.texto_crudo(&nombre);
                    }
                    continue;
                }
                // Un `<` que no abre ninguna etiqueta, como en "a < b", es texto normal.
            }
            texto.push(actual);
            self.pos += 1;
        }
        self.vaciar_texto(&mut texto);
    }

    fn vaciar_texto(&mut self, texto: &mut String) {
        if !texto.is_empty() {
            self.tokens.push(Token::Texto(decodificar_entidades(texto)));
            texto.clear();
        }
    }

    fn actual(&self) -> Option<char> {
        self.c.get(self.pos).copied()
    }

    /// ¿Lo que sigue desde la posición actual es `s`, sin importar mayúsculas?
    fn sigue(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .all(|(i, ch)| self.c.get(self.pos + i).is_some_and(|c| c.eq_ignore_ascii_case(&ch)))
    }

    fn saltar_espacios(&mut self) {
        while self.actual().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    /// Lee caracteres mientras se cumpla `seguir`.
    fn leer_mientras(&mut self, seguir: impl Fn(char) -> bool) -> String {
        let inicio = self.pos;
        while self.actual().is_some_and(&seguir) {
            self.pos += 1;
        }
        self.c[inicio..self.pos].iter().collect()
    }

    /// Avanza hasta justo después de `fin`, o hasta el final si no aparece.
    /// Devuelve lo que había antes de `fin`.
    fn leer_hasta(&mut self, fin: &str) -> String {
        let inicio = self.pos;
        while self.pos < self.c.len() && !self.sigue(fin) {
            self.pos += 1;
        }
        let leido = self.c[inicio..self.pos].iter().collect();
        self.pos = (self.pos + fin.chars().count()).min(self.c.len());
        leido
    }

    /// Intenta leer una etiqueta empezando en un `<`. Si lo que sigue no es una etiqueta,
    /// no avanza y devuelve `None`.
    fn etiqueta(&mut self) -> Option<Token> {
        let inicio = self.pos;
        self.pos += 1;

        if self.sigue("!--") {
            self.pos += 3;
            return Some(Token::Comentario(self.leer_hasta("-->")));
        }
        if self.sigue("!") || self.sigue("?") {
            let contenido = self.leer_hasta(">");
            let es_doctype = contenido.to_ascii_lowercase().starts_with("!doctype");
            return Some(if es_doctype { Token::Doctype } else { Token::Comentario(contenido) });
        }

        let cierre = self.sigue("/");
        if cierre {
            self.pos += 1;
        }
        if !self.actual().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.pos = inicio;
            return None;
        }
        let nombre = self
            .leer_mientras(|c| !c.is_whitespace() && c != '/' && c != '>')
            .to_ascii_lowercase();

        if cierre {
            self.leer_hasta(">");
            return Some(Token::Cierra { nombre });
        }
        let atributos = self.atributos();
        Some(Token::Abre { nombre, atributos })
    }

    /// Lee `clave="valor"` hasta el `>` que cierra la etiqueta.
    fn atributos(&mut self) -> Vec<(String, String)> {
        let mut atributos = Vec::new();
        loop {
            self.saltar_espacios();
            match self.actual() {
                None => break,
                Some('>') => {
                    self.pos += 1;
                    break;
                }
                // En HTML, la `/` de `<br/>` no cambia nada: se ignora.
                Some('/') => {
                    self.pos += 1;
                    continue;
                }
                Some(_) => {}
            }
            let mut clave = self.leer_mientras(|c| !c.is_whitespace() && !"=>/".contains(c));
            if clave.is_empty() {
                // Un `=` suelto: se toma como parte del nombre para no quedarse atascado.
                clave.push('=');
                self.pos += 1;
            }
            self.saltar_espacios();
            let mut valor = String::new();
            if self.actual() == Some('=') {
                self.pos += 1;
                self.saltar_espacios();
                valor = match self.actual() {
                    Some(comilla @ ('"' | '\'')) => {
                        self.pos += 1;
                        self.leer_hasta(&comilla.to_string())
                    }
                    _ => self.leer_mientras(|c| !c.is_whitespace() && c != '>'),
                };
            }
            atributos.push((clave.to_ascii_lowercase(), decodificar_entidades(&valor)));
        }
        atributos
    }

    /// Después de `<style>` o `<script>`: todo hasta `</style>` es texto, sin buscar etiquetas.
    fn texto_crudo(&mut self, nombre: &str) {
        let fin = format!("</{nombre}");
        let inicio = self.pos;
        while self.pos < self.c.len() && !self.sigue(&fin) {
            self.pos += 1;
        }
        let contenido: String = self.c[inicio..self.pos].iter().collect();
        if contenido.is_empty() {
            return;
        }
        // En `title` y `textarea` sí se traducen las entidades; en `style` y `script`, no.
        let texto = if nombre == "title" || nombre == "textarea" {
            decodificar_entidades(&contenido)
        } else {
            contenido
        };
        self.tokens.push(Token::Texto(texto));
        // El `</style>` lo lee el bucle principal como una etiqueta de cierre normal.
    }
}

/// Traduce `&amp;`, `&lt;`, `&#241;`, `&#xF1;`... a su carácter. Lo desconocido se deja igual.
pub fn decodificar_entidades(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len());
    let mut resto = texto;
    while let Some(i) = resto.find('&') {
        salida.push_str(&resto[..i]);
        resto = &resto[i..];
        let fin = resto.char_indices().take(12).find(|&(_, c)| c == ';').map(|(j, _)| j);
        let traducido = fin.and_then(|j| entidad(&resto[1..j]).map(|c| (c, j)));
        match traducido {
            Some((caracter, j)) => {
                salida.push(caracter);
                resto = &resto[j + 1..];
            }
            None => {
                salida.push('&');
                resto = &resto[1..];
            }
        }
    }
    salida.push_str(resto);
    salida
}

fn entidad(nombre: &str) -> Option<char> {
    if let Some(numero) = nombre.strip_prefix('#') {
        let codigo = match numero.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => numero.parse().ok()?,
        };
        return char::from_u32(codigo);
    }
    Some(match nombre {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "copy" => '©',
        "ntilde" => 'ñ',
        _ => return None,
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn abre(nombre: &str, atributos: &[(&str, &str)]) -> Token {
        Token::Abre {
            nombre: nombre.into(),
            atributos: atributos.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }
    fn cierra(nombre: &str) -> Token {
        Token::Cierra { nombre: nombre.into() }
    }
    fn texto(t: &str) -> Token {
        Token::Texto(t.into())
    }

    #[test]
    fn etiquetas_y_texto() {
        assert_eq!(tokenizar("<p>hola</p>"), vec![abre("p", &[]), texto("hola"), cierra("p")]);
    }

    #[test]
    fn atributos_con_y_sin_comillas() {
        assert_eq!(
            tokenizar(r#"<a href="x.html" class='y' id=z hidden>"#),
            vec![abre("a", &[("href", "x.html"), ("class", "y"), ("id", "z"), ("hidden", "")])]
        );
    }

    #[test]
    fn las_mayusculas_se_normalizan() {
        assert_eq!(tokenizar("<DIV ID=a></Div>"), vec![abre("div", &[("id", "a")]), cierra("div")]);
    }

    #[test]
    fn entidades() {
        assert_eq!(tokenizar("a &amp; b &lt;3 &#241; &#xF1; &raro;"), vec![texto("a & b <3 ñ ñ &raro;")]);
    }

    #[test]
    fn un_menor_que_suelto_es_texto() {
        assert_eq!(tokenizar("1 < 2"), vec![texto("1 < 2")]);
    }

    #[test]
    fn comentarios_y_doctype() {
        assert_eq!(
            tokenizar("<!DOCTYPE html><!-- nota --><br/>"),
            vec![Token::Doctype, Token::Comentario(" nota ".into()), abre("br", &[])]
        );
    }

    #[test]
    fn el_contenido_de_style_no_se_parsea() {
        assert_eq!(
            tokenizar("<style>a > b { }</style>"),
            vec![abre("style", &[]), texto("a > b { }"), cierra("style")]
        );
    }

    #[test]
    fn una_etiqueta_sin_terminar_no_rompe_nada() {
        assert_eq!(tokenizar("<p class=\"a"), vec![abre("p", &[("class", "a")])]);
    }
}
