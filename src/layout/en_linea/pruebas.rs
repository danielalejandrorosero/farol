//! Pruebas del layout en línea: HTML → estilo → layout → renglones y fragmentos.
//!
//! Los anchos esperados se calculan con la misma `fuente::ancho_texto` que usa el layout: así la
//! prueba no depende de cuánto mide cada letra, sino de que las cuentas se hagan bien.

use crate::estilo::{self, Estilo, NodoEstilado};
use crate::html;
use crate::layout::{self, Caja, Linea};
use crate::pintado::fuente;

/// Estila `cuerpo` (lo que va dentro de `<body>`, con `margin: 0` para que las cuentas empiecen
/// en x = 0) más el CSS extra.
fn estilar(css: &str, cuerpo: &str) -> NodoEstilado {
    let pagina = format!("<html><style>body{{margin:0}} {css}</style><body>{cuerpo}</body></html>");
    let doc = html::parsear(&pagina);
    estilo::estilar(&doc, &estilo::hojas_del_documento(&doc))
}

/// Todos los renglones de la página, en orden del documento.
fn lineas<'c, 'a>(caja: &'c Caja<'a>) -> Vec<&'c Linea<'a>> {
    let mut todas: Vec<&Linea> = caja.lineas.iter().collect();
    for hijo in &caja.hijos {
        todas.extend(lineas(hijo));
    }
    todas
}

/// El texto de cada renglón, juntando sus fragmentos.
fn textos(lineas: &[&Linea]) -> Vec<String> {
    lineas.iter().map(|l| l.fragmentos.iter().map(|f| f.texto.as_str()).collect()).collect()
}

fn casi(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn ancho(texto: &str) -> f32 {
    fuente::ancho_texto(texto, 16.0, false)
}

// --- Colapso de espacios ---

#[test]
fn los_espacios_seguidos_valen_uno() {
    let est = estilar("", "<p>  a \n\n\t  b  </p>");
    let raiz = layout::calcular(&est, 800.0);
    let todas = lineas(&raiz);
    assert_eq!(textos(&todas), ["a b"]);
    let f = &todas[0].fragmentos[0];
    // Sin el espacio del principio: empieza en 0. Sin el del final: mide "a b".
    assert_eq!(f.rect.x, 0.0);
    assert!(casi(f.rect.ancho, ancho("a b")));
}

#[test]
fn los_espacios_colapsan_entre_nodos() {
    let est = estilar("", "<p><b>a</b> <i>b</i></p><p><b>a </b> <i> b</i></p><p>a<b>b</b></p>");
    let raiz = layout::calcular(&est, 800.0);
    // Entre `a` y `b` hay un solo espacio aunque venga de tres nodos; y sin espacio en el HTML,
    // no se inventa uno.
    assert_eq!(textos(&lineas(&raiz)), ["a b", "a b", "ab"]);
}

#[test]
fn el_espacio_duro_no_colapsa_ni_parte() {
    // "a\u{A0}\u{A0}b" es una sola palabra de 4 caracteres con dos espacios adentro.
    let est = estilar("", "<p>x a\u{A0}\u{A0}b</p>");
    let justo = ancho("x a  ") - 1.0; // cabe "x a" pero no "x a  b"
    let raiz = layout::calcular(&est, justo);
    assert_eq!(textos(&lineas(&raiz)), ["x", "a  b"]);
}

#[test]
fn los_espacios_del_borde_de_cada_renglon_se_quitan() {
    let est = estilar("", "<p>uno dos tres</p>");
    let raiz = layout::calcular(&est, ancho("uno dos"));
    let todas = lineas(&raiz);
    assert_eq!(textos(&todas), ["uno dos", "tres"]);
    // El espacio entre "dos" y "tres" desapareció: ni al final del primero ni al principio del
    // segundo. El primer renglón mide exactamente "uno dos".
    let f = &todas[0].fragmentos[0];
    assert!(casi(f.rect.ancho, ancho("uno dos")));
    assert_eq!(todas[1].fragmentos[0].rect.x, 0.0);
}

// --- Partición voraz ---

#[test]
fn particion_voraz_con_anchos_conocidos() {
    let est = estilar("", "<p>aaa bbb ccc ddd</p>");
    // Justo "aaa bbb": entra (el espacio de después no cuenta).
    let raiz = layout::calcular(&est, ancho("aaa bbb"));
    assert_eq!(textos(&lineas(&raiz)), ["aaa bbb", "ccc ddd"]);
    // Medio pixel menos: "bbb" ya no cabe junto a "aaa" y baja; y así cada palabra.
    let raiz = layout::calcular(&est, ancho("aaa bbb") - 0.5);
    assert_eq!(textos(&lineas(&raiz)), ["aaa", "bbb", "ccc", "ddd"]);
    // Sobrado: todo en un renglón.
    let raiz = layout::calcular(&est, 800.0);
    assert_eq!(textos(&lineas(&raiz)), ["aaa bbb ccc ddd"]);
}

#[test]
fn una_palabra_mas_larga_que_el_renglon_va_sola_y_desborda() {
    let est = estilar("", "<p>a electroencefalografista b</p>");
    let raiz = layout::calcular(&est, 50.0);
    let todas = lineas(&raiz);
    assert_eq!(textos(&todas), ["a", "electroencefalografista", "b"]);
    assert!(todas[1].fragmentos[0].rect.ancho > 50.0);
    assert_eq!(todas[1].fragmentos[0].rect.x, 0.0);
}

#[test]
fn sin_espacio_no_se_parte_aunque_cambie_el_elemento() {
    // "muy<b>bueno</b>" es una sola unidad: no se puede cortar en el medio.
    let est = estilar("", "<p>es muy<b>bueno</b></p>");
    let raiz = layout::calcular(&est, ancho("es muy"));
    assert_eq!(textos(&lineas(&raiz)), ["es", "muybueno"]);
}

// --- <br> ---

#[test]
fn br_corta_el_renglon() {
    let est = estilar(
        "",
        "<p>a<br>b</p><p>c<br><br>d</p><p>e<br></p><p>f <br> g</p>",
    );
    let raiz = layout::calcular(&est, 800.0);
    // Dos `<br>` seguidos dejan un renglón vacío; uno al final no agrega nada; los espacios
    // alrededor del `<br>` desaparecen.
    assert_eq!(textos(&lineas(&raiz)), ["a", "b", "c", "", "d", "e", "f", "g"]);
}

// --- Alineación ---

#[test]
fn text_align_centro_y_derecha() {
    let est = estilar(
        "",
        "<p style='text-align:center'>hola</p><p style='text-align:right'>hola</p><p>hola</p>",
    );
    let raiz = layout::calcular(&est, 800.0);
    let x: Vec<f32> = lineas(&raiz).iter().map(|l| l.fragmentos[0].rect.x).collect();
    let w = ancho("hola");
    assert!(casi(x[0], (800.0 - w) / 2.0));
    assert!(casi(x[1], 800.0 - w));
    assert_eq!(x[2], 0.0);
}

#[test]
fn cada_renglon_se_alinea_por_separado() {
    let est = estilar("", "<p style='text-align:right'>uno dos tres</p>");
    let raiz = layout::calcular(&est, ancho("uno dos"));
    let todas = lineas(&raiz);
    // El segundo renglón ("tres") es más corto y se corre más.
    assert!(casi(todas[0].fragmentos[0].rect.x, 0.0));
    assert!(casi(todas[1].fragmentos[0].rect.x, ancho("uno dos") - ancho("tres")));
}

// --- Alto y línea base ---

#[test]
fn letras_de_distinto_tamano_comparten_la_linea_base() {
    let est = estilar("", "<p>a<span style='font-size:32px'>B</span>c</p>");
    let raiz = layout::calcular(&est, 800.0);
    let todas = lineas(&raiz);
    assert_eq!(todas.len(), 1);
    let linea = todas[0];
    assert_eq!(linea.fragmentos.len(), 3);
    for f in &linea.fragmentos {
        // Donde `Lienzo::texto` pondrá la base de cada fragmento: todas en la misma `y`.
        assert!(casi(f.rect.y + fuente::linea_base(f.estilo.tam_fuente), linea.base));
    }
    // La "B" de 32 px manda: 26 de letra + 3.2 de medio interlineado arriba de la base, y 6 + 3.2
    // abajo (line-height normal = 1.2 × 32 = 38.4). El renglón empieza en y = 16 (margen del p).
    assert!(casi(linea.base - linea.rect.y, 29.2));
    assert!(casi(linea.rect.alto, 38.4));
}

#[test]
fn el_line_height_del_bloque_es_el_minimo_de_cada_renglon() {
    // Letra chica en un párrafo de 16 px con line-height 1.5: el renglón igual mide 24, lo que
    // pide el "puntal" del párrafo (el texto de 8 px solo pediría 12).
    let est = estilar("p{line-height:1.5}", "<p><span style='font-size:8px'>chico</span></p>");
    let raiz = layout::calcular(&est, 800.0);
    assert!(casi(lineas(&raiz)[0].rect.alto, 24.0));
}

#[test]
fn el_alto_de_la_anonima_es_la_suma_de_sus_renglones() {
    let frase = "Había una vez un párrafo bastante largo. ".repeat(8);
    let est = estilar("", &format!("<p>{frase}<span style='font-size:24px'>grande</span> fin</p>"));
    let raiz = layout::calcular(&est, 300.0);
    let p = &raiz.hijos[0].hijos[0].hijos[0];
    let anonima = &p.hijos[0];
    assert!(anonima.lineas.len() > 3);
    let suma: f32 = anonima.lineas.iter().map(|l| l.rect.alto).sum();
    assert!(casi(anonima.dimensiones.contenido.alto, suma));
    assert!(casi(p.dimensiones.contenido.alto, suma));
    // Apilados sin huecos: cada renglón empieza donde termina el anterior.
    for par in anonima.lineas.windows(2) {
        assert!(casi(par[1].rect.y, par[0].rect.y + par[0].rect.alto));
    }
    // El último renglón tiene la palabra de 24 px: es más alto que los de 16 px.
    let ultimo = anonima.lineas.last().unwrap();
    assert!(ultimo.rect.alto > anonima.lineas[0].rect.alto);
}

// --- Elementos en línea ---

fn con_fondo(e: &Estilo) -> bool {
    e.fondo.a > 0
}

#[test]
fn el_relleno_y_el_borde_de_un_en_linea_ocupan_lugar() {
    let est = estilar(
        "span{padding:2px 5px; border:1px solid; background:red}",
        "<p>x <span>y</span> z</p>",
    );
    let raiz = layout::calcular(&est, 800.0);
    let linea = lineas(&raiz)[0];
    let caja = linea.cajas.iter().find(|c| con_fondo(c.estilo)).unwrap();
    assert!(casi(caja.rect.x, ancho("x ")));
    assert!(casi(caja.rect.ancho, 1.0 + 5.0 + ancho("y") + 5.0 + 1.0));
    // La "y" empieza después del borde y el relleno izquierdos.
    let y = linea.fragmentos.iter().find(|f| f.texto == "y").unwrap();
    assert!(casi(y.rect.x, ancho("x ") + 6.0));
    // De alto: la celda de la letra (16) + relleno (2 + 2) + borde (1 + 1), alrededor de la base.
    assert!(casi(caja.rect.alto, 22.0));
    assert!(casi(caja.rect.y, linea.base - fuente::ascendente(16.0) - 3.0));
    // " z" empieza después del borde derecho.
    let z = linea.fragmentos.last().unwrap();
    assert!(casi(z.rect.x, caja.rect.x + caja.rect.ancho));
}

#[test]
fn un_en_linea_partido_lleva_bordes_solo_en_sus_puntas() {
    let est = estilar(
        "span{border:2px solid; background:red}",
        "<p><span>uno dos tres</span></p>",
    );
    let raiz = layout::calcular(&est, ancho("uno") + 4.0);
    let todas = lineas(&raiz);
    assert_eq!(textos(&todas), ["uno", "dos", "tres"]);
    let bordes: Vec<(f32, f32)> = todas
        .iter()
        .map(|l| {
            let c = l.cajas.iter().find(|c| con_fondo(c.estilo)).unwrap();
            (c.borde.izquierda, c.borde.derecha)
        })
        .collect();
    assert_eq!(bordes, [(2.0, 0.0), (0.0, 0.0), (0.0, 2.0)]);
    // El pedazo del medio va de punta a punta de "dos", sin el espacio que se quitó.
    let medio = todas[1].cajas.iter().find(|c| con_fondo(c.estilo)).unwrap();
    assert!(casi(medio.rect.ancho, ancho("dos")));
}

#[test]
fn un_fragmento_por_nodo_de_texto() {
    // Las palabras y espacios de un mismo nodo se juntan en un fragmento (el subrayado no se
    // corta entre palabras); al cambiar de estilo empieza otro.
    let est = estilar("", "<p>Un <a href=x>azul y subrayado</a>.</p>");
    let raiz = layout::calcular(&est, 800.0);
    let todas = lineas(&raiz);
    let partes: Vec<&str> = todas[0].fragmentos.iter().map(|f| f.texto.as_str()).collect();
    assert_eq!(partes, ["Un ", "azul y subrayado", "."]);
    // Cada uno empieza donde termina el anterior.
    let f = &todas[0].fragmentos;
    assert!(casi(f[1].rect.x, f[0].rect.x + f[0].rect.ancho));
    assert!(casi(f[2].rect.x, f[1].rect.x + f[1].rect.ancho));
}

#[test]
fn la_negrita_mide_con_su_propia_letra() {
    let est = estilar("", "<p>a <b>negrita</b></p>");
    let raiz = layout::calcular(&est, 800.0);
    let b = &lineas(&raiz)[0].fragmentos[1];
    assert!(casi(b.rect.ancho, fuente::ancho_texto("negrita", 16.0, true)));
    assert!(b.rect.ancho > ancho("negrita"));
}
