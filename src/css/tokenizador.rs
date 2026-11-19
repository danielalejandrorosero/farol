//! El tokenizador de CSS: corta el texto en piezas con sentido, igual que el de HTML.
//!
//! `div.caja { margin: 10px !important }` se convierte en:
//! `Identificador div`, `Simbolo .`, `Identificador caja`, `Espacio`, `Simbolo {`, `Espacio`,
//! `Identificador margin`, `Simbolo :`, `Espacio`, `Dimension 10 px`, `Espacio`, `Importante`, …
//!
//! Los comentarios `/* */` desaparecen aquí, así el parser nunca los ve. El tokenizador no
//! decide si algo es válido: solo corta. Nunca falla; un carácter que no reconoce sale como
//! `Simbolo` y el parser decide qué hacer con él.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// `div`, `color`, `solid`, `-webkit-algo`. Se guarda tal como venía (las clases distinguen
    /// mayúsculas); el parser pasa a minúsculas lo que no las distingue.
    Identificador(String),
    /// `rgb(`: el nombre en minúsculas; el `(` ya se consumió.
    Funcion(String),
    /// `@media`, `@import`: el nombre sin la arroba.
    Arroba(String),
    /// `#fff`, `#principal`: sin el `#`.
    Hash(String),
    Cadena(String),
    /// Una cadena que se cortó con un salto de línea antes de cerrarse. Vuelve inválida la
    /// declaración donde aparece.
    CadenaRota,
    /// `url(imagen.png)` sin comillas: el contenido entre paréntesis.
    Url(String),
    Numero(f32),
    /// Un número pegado a una unidad: `12px`, `1.5em`. La unidad va en minúsculas.
    Dimension(f32, String),
    Porcentaje(f32),
    /// `!important`, con o sin espacios entre `!` y la palabra.
    Importante,
    /// Cualquier otro carácter suelto: `{ } : ; , . > + ~ [ ] ( ) *` y demás.
    Simbolo(char),
    /// Uno o más espacios, tabuladores o saltos de línea seguidos. Importa porque en
    /// `div p` el espacio es el combinador descendiente.
    Espacio,
}

impl std::fmt::Display for Token {
    /// Devuelve el token como se escribiría en CSS. Sirve para mostrar valores que Farol no
    /// interpreta (`font-family: Arial, sans-serif`) y para los avisos.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Identificador(s) => write!(f, "{s}"),
            Token::Funcion(s) => write!(f, "{s}("),
            Token::Arroba(s) => write!(f, "@{s}"),
            Token::Hash(s) => write!(f, "#{s}"),
            Token::Cadena(s) => write!(f, "\"{s}\""),
            Token::CadenaRota => write!(f, "\"…"),
            Token::Url(s) => write!(f, "url({s})"),
            Token::Numero(n) => write!(f, "{n}"),
            Token::Dimension(n, u) => write!(f, "{n}{u}"),
            Token::Porcentaje(n) => write!(f, "{n}%"),
            Token::Importante => write!(f, "!important"),
            Token::Simbolo(c) => write!(f, "{c}"),
            Token::Espacio => write!(f, " "),
        }
    }
}

/// Une tokens en texto, tal como se escribirían en CSS.
pub fn a_texto(tokens: &[Token]) -> String {
    tokens.iter().map(Token::to_string).collect::<String>().trim().to_string()
}

pub fn tokenizar(css: &str) -> Vec<Token> {
    let mut t = Tokenizador { c: css.chars().collect(), pos: 0, tokens: Vec::new() };
    t.correr();
    t.tokens
}

struct Tokenizador {
    c: Vec<char>,
    pos: usize,
    tokens: Vec<Token>,
}

/// Letras, `_` y cualquier carácter no ASCII (`ñ`, `é`) pueden empezar un nombre.
fn inicia_nombre(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || !c.is_ascii()
}

/// Dentro de un nombre valen además dígitos y guiones.
fn de_nombre(c: char) -> bool {
    inicia_nombre(c) || c.is_ascii_digit() || c == '-'
}

impl Tokenizador {
    fn en(&self, i: usize) -> Option<char> {
        self.c.get(i).copied()
    }

    fn actual(&self) -> Option<char> {
        self.en(self.pos)
    }

    /// ¿Lo que sigue desde la posición actual es `s`, sin importar mayúsculas?
    fn sigue(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .all(|(i, ch)| self.en(self.pos + i).is_some_and(|c| c.eq_ignore_ascii_case(&ch)))
    }

    /// ¿En `i` empieza un escape válido? Una barra invertida seguida de algo que no sea salto de línea.
    fn escape_en(&self, i: usize) -> bool {
        self.en(i) == Some('\\') && self.en(i + 1).is_some_and(|c| c != '\n')
    }

    /// ¿En `i` empieza un identificador? `abc`, `_x`, `-abc`, `--var`, `\31 0`.
    fn identificador_en(&self, i: usize) -> bool {
        match self.en(i) {
            Some('-') => match self.en(i + 1) {
                Some(c) if inicia_nombre(c) || c == '-' => true,
                Some('\\') => self.escape_en(i + 1),
                _ => false,
            },
            Some('\\') => self.escape_en(i),
            Some(c) => inicia_nombre(c),
            None => false,
        }
    }

    /// ¿En `i` empieza un número? `5`, `.5`, `-5`, `+.5`.
    fn numero_en(&self, i: usize) -> bool {
        let digito = |j| self.en(j).is_some_and(|c: char| c.is_ascii_digit());
        match self.en(i) {
            Some('+' | '-') => digito(i + 1) || (self.en(i + 1) == Some('.') && digito(i + 2)),
            Some('.') => digito(i + 1),
            Some(c) => c.is_ascii_digit(),
            None => false,
        }
    }

    fn correr(&mut self) {
        while let Some(c) = self.actual() {
            if self.sigue("/*") {
                self.saltar_comentario();
                continue;
            }
            // Restos de cuando el CSS se escondía en comentarios HTML dentro de <style>.
            if self.sigue("<!--") {
                self.pos += 4;
                continue;
            }
            if self.sigue("-->") {
                self.pos += 3;
                continue;
            }
            let token = if c.is_whitespace() {
                while self.actual().is_some_and(char::is_whitespace) {
                    self.pos += 1;
                }
                Token::Espacio
            } else if c == '"' || c == '\'' {
                self.cadena(c)
            } else if c == '#' {
                self.pos += 1;
                if self.actual().is_some_and(de_nombre) || self.escape_en(self.pos) {
                    Token::Hash(self.nombre())
                } else {
                    Token::Simbolo('#')
                }
            } else if c == '@' && self.identificador_en(self.pos + 1) {
                self.pos += 1;
                Token::Arroba(self.nombre().to_ascii_lowercase())
            } else if c == '!' {
                self.importante()
            } else if self.numero_en(self.pos) {
                self.numero()
            } else if self.identificador_en(self.pos) {
                self.identificador_o_funcion()
            } else {
                self.pos += 1;
                Token::Simbolo(c)
            };
            // Dos espacios seguidos (por ejemplo, con un comentario en medio) cuentan como uno.
            if token == Token::Espacio && self.tokens.last() == Some(&Token::Espacio) {
                continue;
            }
            self.tokens.push(token);
        }
    }

    fn saltar_comentario(&mut self) {
        self.pos += 2;
        while self.pos < self.c.len() && !self.sigue("*/") {
            self.pos += 1;
        }
        // Un comentario sin cerrar se come el resto del archivo, como en los navegadores.
        self.pos = (self.pos + 2).min(self.c.len());
    }

    /// Lee un escape que empieza en `\`. `\26` es el carácter con código hexadecimal 26 (`&`);
    /// `\.` es un punto literal.
    fn escape(&mut self) -> char {
        self.pos += 1;
        let inicio = self.pos;
        while self.pos - inicio < 6 && self.actual().is_some_and(|c| c.is_ascii_hexdigit()) {
            self.pos += 1;
        }
        if self.pos > inicio {
            let hex: String = self.c[inicio..self.pos].iter().collect();
            // Un espacio después del código hexadecimal es parte del escape.
            if self.actual().is_some_and(char::is_whitespace) {
                self.pos += 1;
            }
            let codigo = u32::from_str_radix(&hex, 16).unwrap_or(0xFFFD);
            return char::from_u32(codigo).filter(|&c| c != '\0').unwrap_or('\u{FFFD}');
        }
        match self.actual() {
            Some(c) => {
                self.pos += 1;
                c
            }
            None => '\u{FFFD}',
        }
    }

    /// Lee los caracteres de un nombre (identificador, hash, unidad).
    fn nombre(&mut self) -> String {
        let mut nombre = String::new();
        while let Some(c) = self.actual() {
            if de_nombre(c) {
                nombre.push(c);
                self.pos += 1;
            } else if self.escape_en(self.pos) {
                let c = self.escape();
                nombre.push(c);
            } else {
                break;
            }
        }
        nombre
    }

    fn identificador_o_funcion(&mut self) -> Token {
        let nombre = self.nombre();
        if self.actual() != Some('(') {
            return Token::Identificador(nombre);
        }
        self.pos += 1;
        let nombre = nombre.to_ascii_lowercase();
        if nombre == "url" {
            // `url(` con comillas es una función normal; sin comillas, todo hasta `)` es la
            // dirección, aunque traiga `;` o `,` (como en las imágenes `data:`).
            let mut i = self.pos;
            while self.en(i).is_some_and(char::is_whitespace) {
                i += 1;
            }
            if !matches!(self.en(i), Some('"' | '\'')) {
                let inicio = self.pos;
                while self.actual().is_some_and(|c| c != ')') {
                    self.pos += 1;
                }
                let url: String = self.c[inicio..self.pos].iter().collect();
                self.pos = (self.pos + 1).min(self.c.len());
                return Token::Url(url.trim().to_string());
            }
        }
        Token::Funcion(nombre)
    }

    fn numero(&mut self) -> Token {
        let inicio = self.pos;
        if matches!(self.actual(), Some('+' | '-')) {
            self.pos += 1;
        }
        let digitos = |t: &mut Self| {
            while t.actual().is_some_and(|c| c.is_ascii_digit()) {
                t.pos += 1;
            }
        };
        digitos(self);
        if self.actual() == Some('.') && self.en(self.pos + 1).is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
            digitos(self);
        }
        let texto: String = self.c[inicio..self.pos].iter().collect();
        // Por cómo se leyó, el texto siempre es un número válido; un número gigante da infinito
        // y lo rechaza el parser de valores.
        let n = texto.trim_start_matches('+').parse::<f32>().unwrap_or(f32::NAN);
        if self.actual() == Some('%') {
            self.pos += 1;
            Token::Porcentaje(n)
        } else if self.identificador_en(self.pos) {
            Token::Dimension(n, self.nombre().to_ascii_lowercase())
        } else {
            Token::Numero(n)
        }
    }

    fn cadena(&mut self, comilla: char) -> Token {
        self.pos += 1;
        let mut texto = String::new();
        loop {
            match self.actual() {
                // Una cadena sin cerrar al final del archivo se da por cerrada.
                None => return Token::Cadena(texto),
                Some('\n') => return Token::CadenaRota,
                Some(c) if c == comilla => {
                    self.pos += 1;
                    return Token::Cadena(texto);
                }
                Some('\\') => match self.en(self.pos + 1) {
                    // Barra al final de una línea: la cadena continúa en la siguiente.
                    Some('\n') => self.pos += 2,
                    None => self.pos += 1,
                    Some(_) => {
                        let c = self.escape();
                        texto.push(c);
                    }
                },
                Some(c) => {
                    texto.push(c);
                    self.pos += 1;
                }
            }
        }
    }

    /// `!` seguido (quizá tras espacios) de `important`. Si no, es un `!` suelto.
    fn importante(&mut self) -> Token {
        let mut i = self.pos + 1;
        while self.en(i).is_some_and(char::is_whitespace) {
            i += 1;
        }
        let guardado = self.pos;
        self.pos = i;
        if self.sigue("important") && !self.en(i + 9).is_some_and(de_nombre) {
            self.pos = i + 9;
            return Token::Importante;
        }
        self.pos = guardado + 1;
        Token::Simbolo('!')
    }
}

#[cfg(test)]
mod pruebas {
    use super::Token::*;
    use super::*;

    fn id(s: &str) -> Token {
        Identificador(s.to_string())
    }

    #[test]
    fn regla_sencilla() {
        assert_eq!(
            tokenizar("p { color: red; }"),
            vec![
                id("p"),
                Espacio,
                Simbolo('{'),
                Espacio,
                id("color"),
                Simbolo(':'),
                Espacio,
                id("red"),
                Simbolo(';'),
                Espacio,
                Simbolo('}'),
            ]
        );
    }

    #[test]
    fn numeros_y_unidades() {
        assert_eq!(
            tokenizar("10px 1.5em 50% 0 -3px .5 +2EM"),
            vec![
                Dimension(10.0, "px".into()),
                Espacio,
                Dimension(1.5, "em".into()),
                Espacio,
                Porcentaje(50.0),
                Espacio,
                Numero(0.0),
                Espacio,
                Dimension(-3.0, "px".into()),
                Espacio,
                Numero(0.5),
                Espacio,
                Dimension(2.0, "em".into()),
            ]
        );
    }

    #[test]
    fn hash_cadenas_y_funciones() {
        assert_eq!(
            tokenizar("#fff 'hola' \"a\\\"b\" rgb(1,2)"),
            vec![
                Hash("fff".into()),
                Espacio,
                Cadena("hola".into()),
                Espacio,
                Cadena("a\"b".into()),
                Espacio,
                Funcion("rgb".into()),
                Numero(1.0),
                Simbolo(','),
                Numero(2.0),
                Simbolo(')'),
            ]
        );
    }

    #[test]
    fn simbolos_de_selectores() {
        let tokens = tokenizar("a>b+c~d [x] *.y");
        assert!(tokens.contains(&Simbolo('>')));
        assert!(tokens.contains(&Simbolo('+')));
        assert!(tokens.contains(&Simbolo('~')));
        assert!(tokens.contains(&Simbolo('[')));
        assert!(tokens.contains(&Simbolo(']')));
        assert!(tokens.contains(&Simbolo('*')));
        assert!(tokens.contains(&Simbolo('.')));
    }

    #[test]
    fn importante_con_y_sin_espacio() {
        assert_eq!(tokenizar("!important"), vec![Importante]);
        assert_eq!(tokenizar("! IMPORTANT"), vec![Importante]);
        assert_eq!(tokenizar("!importantisimo"), vec![Simbolo('!'), id("importantisimo")]);
    }

    #[test]
    fn comentarios_en_cualquier_parte() {
        assert_eq!(
            tokenizar("a/* x */{/**/color/*y*/:red}"),
            vec![id("a"), Simbolo('{'), id("color"), Simbolo(':'), id("red"), Simbolo('}')]
        );
        // Un comentario entre espacios deja un solo espacio.
        assert_eq!(tokenizar("a /* x */ b"), vec![id("a"), Espacio, id("b")]);
        // Sin cerrar: se come el resto.
        assert_eq!(tokenizar("a /* nunca cierra { }"), vec![id("a"), Espacio]);
    }

    #[test]
    fn arroba_y_url() {
        assert_eq!(
            tokenizar("@MEDIA url(data:a;b,c) url('x')"),
            vec![
                Arroba("media".into()),
                Espacio,
                Url("data:a;b,c".into()),
                Espacio,
                Funcion("url".into()),
                Cadena("x".into()),
                Simbolo(')'),
            ]
        );
    }

    #[test]
    fn identificadores_raros() {
        assert_eq!(tokenizar("-webkit-box"), vec![id("-webkit-box")]);
        assert_eq!(tokenizar("--mi-var"), vec![id("--mi-var")]);
        assert_eq!(tokenizar("niño"), vec![id("niño")]);
        assert_eq!(tokenizar("a\\.b"), vec![id("a.b")]);
        assert_eq!(tokenizar("\\31 0"), vec![id("10")]);
        // Un guion solo no es identificador ni número.
        assert_eq!(tokenizar("- x"), vec![Simbolo('-'), Espacio, id("x")]);
    }

    #[test]
    fn cadena_rota_por_salto_de_linea() {
        assert_eq!(tokenizar("'abc\nx"), vec![CadenaRota, Espacio, id("x")]);
        assert_eq!(tokenizar("'sin cerrar"), vec![Cadena("sin cerrar".into())]);
    }

    #[test]
    fn nunca_falla_con_basura() {
        for basura in ["\\", "#", "@", "!", "-", "+", ".", "url(", "'\\", "1e", "\u{0}", "/*", "/"] {
            let _ = tokenizar(basura);
        }
        let gigante = "9".repeat(80);
        assert!(matches!(tokenizar(&gigante)[..], [Numero(n)] if n.is_infinite()));
    }
}
