//! Propiedades abreviadas: `margin: 10px 20px` es una forma corta de escribir cuatro
//! propiedades. Se expanden aquí, al parsear, para que el motor de estilos solo vea propiedades
//! largas (`margin-top`, `margin-right`…) y nunca tenga que saber de atajos.
//!
//! Regla de CSS que se respeta: lo que la abreviada no menciona vuelve a su valor inicial.
//! `border: 1px solid` deja `border-*-color` en `currentcolor`, aunque antes fuera rojo.

use super::tipos::{Color, Valor};
use super::valores::{Pieza, es_global, valor_de};

const LADOS: [&str; 4] = ["top", "right", "bottom", "left"];

/// Si `propiedad` es una abreviada que Farol conoce, devuelve `Some` con el resultado de
/// expandirla: la lista de propiedades largas, o `None` dentro si el valor no es válido.
/// Si no es abreviada, devuelve `None`.
pub fn expandir(propiedad: &str, piezas: &[Pieza]) -> Option<Option<Vec<(String, Valor)>>> {
    let resultado = match propiedad {
        "margin" | "padding" => cuatro_lados(piezas, |lado| format!("{propiedad}-{lado}")),
        "border-width" => cuatro_lados(piezas, |lado| format!("border-{lado}-width")),
        "border-style" => cuatro_lados(piezas, |lado| format!("border-{lado}-style")),
        "border-color" => cuatro_lados(piezas, |lado| format!("border-{lado}-color")),
        "border" => borde(piezas, &LADOS),
        "border-top" => borde(piezas, &["top"]),
        "border-right" => borde(piezas, &["right"]),
        "border-bottom" => borde(piezas, &["bottom"]),
        "border-left" => borde(piezas, &["left"]),
        "background" => fondo(piezas),
        _ => return None,
    };
    Some(resultado)
}

/// Uno a cuatro valores repartidos como las agujas del reloj desde arriba:
/// `a` → todos; `a b` → vertical, horizontal; `a b c` → arriba, horizontal, abajo;
/// `a b c d` → arriba, derecha, abajo, izquierda.
fn cuatro_lados(piezas: &[Pieza], nombre: impl Fn(&str) -> String) -> Option<Vec<(String, Valor)>> {
    // Cada pieza se valida como si fuera la propiedad larga de arriba: todas son del mismo tipo.
    let tipo = nombre("top");
    let valores: Vec<Valor> =
        piezas.iter().map(|p| valor_de(&tipo, std::slice::from_ref(p))).collect::<Option<_>>()?;
    if valores.len() > 1 && valores.iter().any(es_global) {
        return None;
    }
    let [arriba, derecha, abajo, izquierda] = match &valores[..] {
        [a] => [a, a, a, a],
        [a, b] => [a, b, a, b],
        [a, b, c] => [a, b, c, b],
        [a, b, c, d] => [a, b, c, d],
        _ => return None,
    };
    let valores = [arriba, derecha, abajo, izquierda];
    Some(LADOS.iter().zip(valores).map(|(lado, v)| (nombre(lado), v.clone())).collect())
}

/// `border: 1px solid red` y sus variantes por lado. Las tres partes van en cualquier orden y
/// todas son opcionales; cada una se reconoce por su tipo.
fn borde(piezas: &[Pieza], lados: &[&str]) -> Option<Vec<(String, Valor)>> {
    let mut ancho = None;
    let mut estilo = None;
    let mut color = None;

    if let [pieza] = piezas
        && let Some(global) = valor_de("border-top-color", std::slice::from_ref(pieza)).filter(es_global)
    {
        ancho = Some(global.clone());
        estilo = Some(global.clone());
        color = Some(global);
    } else {
        if piezas.is_empty() || piezas.len() > 3 {
            return None;
        }
        for pieza in piezas {
            let una = std::slice::from_ref(pieza);
            // Se prueba cada tipo en orden; una parte repetida invalida todo.
            let (hueco, valor) = if let Some(v) = valor_de("border-top-color", una) {
                (&mut color, v)
            } else if let Some(v) = valor_de("border-top-style", una) {
                (&mut estilo, v)
            } else if let Some(v) = valor_de("border-top-width", una) {
                (&mut ancho, v)
            } else {
                return None;
            };
            if es_global(&valor) || hueco.is_some() {
                return None;
            }
            *hueco = Some(valor);
        }
    }

    // Lo que no se mencionó vuelve a su valor inicial.
    let ancho = ancho.unwrap_or(Valor::Palabra("medium".into()));
    let estilo = estilo.unwrap_or(Valor::Palabra("none".into()));
    let color = color.unwrap_or(Valor::Palabra("currentcolor".into()));
    let mut salida = Vec::new();
    for lado in lados {
        salida.push((format!("border-{lado}-width"), ancho.clone()));
        salida.push((format!("border-{lado}-style"), estilo.clone()));
        salida.push((format!("border-{lado}-color"), color.clone()));
    }
    Some(salida)
}

/// `background`: Farol solo pinta colores de fondo, así que de toda la abreviada solo se queda
/// con el color. Imágenes, posiciones y repeticiones se ignoran.
fn fondo(piezas: &[Pieza]) -> Option<Vec<(String, Valor)>> {
    // Varias capas separadas por comas: Farol no las soporta.
    if piezas.is_empty() || piezas.contains(&Pieza::Coma) {
        return None;
    }
    let mut color = None;
    for pieza in piezas {
        let Some(valor) = valor_de("background-color", std::slice::from_ref(pieza)) else {
            continue;
        };
        if es_global(&valor) && piezas.len() > 1 {
            return None;
        }
        if color.is_some() {
            return None;
        }
        color = Some(valor);
    }
    let transparente = Valor::Color(Color { r: 0, g: 0, b: 0, a: 0 });
    Some(vec![("background-color".into(), color.unwrap_or(transparente))])
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::css::tipos::{Color, Unidad};
    use crate::css::tokenizador::tokenizar;
    use crate::css::valores::piezas;

    fn expandir_texto(propiedad: &str, texto: &str) -> Option<Vec<(String, Valor)>> {
        expandir(propiedad, &piezas(&tokenizar(texto))).expect("es abreviada")
    }

    fn px(n: f32) -> Valor {
        Valor::Longitud(n, Unidad::Px)
    }

    fn palabra(p: &str) -> Valor {
        Valor::Palabra(p.into())
    }

    /// Solo los valores, en orden arriba, derecha, abajo, izquierda.
    fn valores(propiedad: &str, texto: &str) -> Vec<Valor> {
        expandir_texto(propiedad, texto).unwrap().into_iter().map(|(_, v)| v).collect()
    }

    #[test]
    fn no_abreviadas_no_se_tocan() {
        assert!(expandir("color", &piezas(&tokenizar("red"))).is_none());
    }

    #[test]
    fn margin_de_uno_a_cuatro_valores() {
        assert_eq!(valores("margin", "1px"), vec![px(1.0), px(1.0), px(1.0), px(1.0)]);
        assert_eq!(valores("margin", "1px 2px"), vec![px(1.0), px(2.0), px(1.0), px(2.0)]);
        assert_eq!(valores("margin", "1px 2px 3px"), vec![px(1.0), px(2.0), px(3.0), px(2.0)]);
        assert_eq!(valores("margin", "1px 2px 3px 4px"), vec![px(1.0), px(2.0), px(3.0), px(4.0)]);
        assert_eq!(valores("margin", "0 auto"), vec![px(0.0), palabra("auto"), px(0.0), palabra("auto")]);
    }

    #[test]
    fn margin_nombres_de_las_largas() {
        let nombres: Vec<String> =
            expandir_texto("padding", "1px").unwrap().into_iter().map(|(n, _)| n).collect();
        assert_eq!(nombres, ["padding-top", "padding-right", "padding-bottom", "padding-left"]);
    }

    #[test]
    fn margin_invalido() {
        assert_eq!(expandir_texto("margin", "1px 2px 3px 4px 5px"), None);
        assert_eq!(expandir_texto("margin", "red"), None);
        assert_eq!(expandir_texto("padding", "-1px"), None);
        assert_eq!(expandir_texto("margin", "1px inherit"), None);
        assert_eq!(expandir_texto("margin", ""), None);
    }

    #[test]
    fn border_en_cualquier_orden() {
        let esperado = vec![px(1.0), palabra("solid"), Valor::Color(Color::rgb(255, 0, 0))];
        assert_eq!(valores("border-top", "1px solid red"), esperado);
        assert_eq!(valores("border-top", "red 1px solid"), esperado);
        assert_eq!(valores("border-top", "solid red 1px"), esperado);
    }

    #[test]
    fn border_completa_lo_que_falta() {
        assert_eq!(
            valores("border-left", "solid"),
            vec![palabra("medium"), palabra("solid"), palabra("currentcolor")]
        );
        assert_eq!(
            valores("border-left", "none"),
            vec![palabra("medium"), palabra("none"), palabra("currentcolor")]
        );
        assert_eq!(valores("border-left", "0")[0], px(0.0));
        assert_eq!(valores("border-left", "thick")[0], palabra("thick"));
    }

    #[test]
    fn border_da_doce_largas() {
        let todo = expandir_texto("border", "2px dashed #000").unwrap();
        assert_eq!(todo.len(), 12);
        assert_eq!(todo[0], ("border-top-width".into(), px(2.0)));
        assert_eq!(todo[11], ("border-left-color".into(), Valor::Color(Color::rgb(0, 0, 0))));
    }

    #[test]
    fn border_invalido() {
        assert_eq!(expandir_texto("border", "1px 2px"), None, "dos anchos");
        assert_eq!(expandir_texto("border", "solid dashed"), None, "dos estilos");
        assert_eq!(expandir_texto("border", "1px solid banana"), None);
        assert_eq!(expandir_texto("border", "1px solid red blue"), None);
    }

    #[test]
    fn border_global() {
        assert_eq!(valores("border", "inherit").len(), 12);
        assert!(valores("border", "inherit").iter().all(|v| *v == palabra("inherit")));
    }

    #[test]
    fn border_de_un_tipo_y_cuatro_lados() {
        assert_eq!(valores("border-style", "solid none"), vec![
            palabra("solid"),
            palabra("none"),
            palabra("solid"),
            palabra("none")
        ]);
        assert_eq!(valores("border-width", "thin 2px 3px")[3], px(2.0));
        assert_eq!(valores("border-color", "red")[2], Valor::Color(Color::rgb(255, 0, 0)));
        assert_eq!(expandir_texto("border-style", "wavy"), None);
        assert_eq!(expandir_texto("border-width", "auto"), None);
    }

    #[test]
    fn background_se_queda_con_el_color() {
        let fondo = |t| valores("background", t);
        assert_eq!(fondo("#eee"), vec![Valor::Color(Color::rgb(238, 238, 238))]);
        assert_eq!(fondo("url(x.png) no-repeat blue"), vec![Valor::Color(Color::rgb(0, 0, 255))]);
        assert_eq!(fondo("none"), vec![Valor::Color(Color { r: 0, g: 0, b: 0, a: 0 })]);
        assert_eq!(fondo("inherit"), vec![palabra("inherit")]);
        assert_eq!(expandir_texto("background", "red blue"), None);
        assert_eq!(expandir_texto("background", "url(a), url(b)"), None);
    }
}
