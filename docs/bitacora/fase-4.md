# Bitácora — Fase 4: motor de estilos (la cascada)

Fecha: 2026-11-18. Estado: cerrada.

## Qué se construyó

- `src/estilo/por_defecto.css`: la hoja de estilos del navegador, con los valores de Chrome
  (`body` 8px de margen, `h1` de 2em en negrita, `a` azul y subrayado, listas con 40px de
  sangría, `head` y `script` ocultos…). Va metida dentro del ejecutable al compilar.
- `src/estilo/emparejar.rs`: decide si un selector le aplica a un elemento, incluido el
  combinador descendiente (`nav a`).
- `src/estilo/cascada.rs`: junta todo lo que le aplica a un elemento y lo ordena de más fuerte a
  más débil.
- `src/estilo/computar.rs`: convierte lo que ganó en números finales (herencia, `em`, `inherit`,
  `currentcolor`, bordes sin estilo).
- `src/estilo/tipos.rs` (el contrato con el layout) e `imprimir.rs`.
- `farol --estilo <archivo>`: imprime el árbol estilado. `farol --estilo <archivo> <selector>`
  muestra, para cada elemento que coincide, qué declaraciones pelean por cada propiedad y cuál gana.
- `pruebas/05-cascada.html`: especificidad, orden, `!important`, `style=""`, herencia, `em`
  anidados, clases múltiples, descendientes, `display: none` y un valor inválido.
- 45 pruebas automáticas nuevas (126 en total).

## Hito

`cargo run -- --estilo pruebas/01-hola.html`:

```
#documento
└── <html> bloque
    └── <body> bloque · color #1b2430 · fondo #ffffff · margen 8px
        ├── <h1> bloque · color #0f6e6e · 32px negrita · margen 21.44px 0
        │   └── "Hola, mundo"
        ├── <p> bloque · margen 16px 0
        └── <div class="caja"> bloque · fondo #e3f1f0 · margen 20px · relleno 10px · borde 2px #0f6e6e
            └── <p> bloque · margen 16px 0
                ├── <b> en línea · negrita
                └── <a> en línea · color #0000ee · subrayado
```

Y qué regla ganó, con `cargo run -- --estilo pruebas/05-cascada.html "p.nota"`:

```
<p id="especifico" class="nota">
  color:     #ff0000    gana   autor · #especifico (1,0,0)
             #0000ff    pierde autor · .nota (0,1,0)
             #008000    pierde autor · p (0,0,1)
  margin-top: 1em       gana   navegador · p (0,0,1)
```

Los números se revisaron a mano contra los valores que calcula Chrome (pestaña "Computed").

## Conceptos nuevos

- **La C de CSS: una pila de papeles transparentes.** Imagina tres acetatos encima de la página:
  abajo el del navegador (el `h1` grande), encima el del autor (los `<style>`), arriba el del
  atributo `style=""`. Lo que está escrito en un acetato de arriba tapa lo de abajo; donde el de
  arriba no dice nada, se ve lo de abajo. Dentro de un mismo acetato manda la especificidad, y
  a igual especificidad, lo que se escribió después. `!important` es una nota pegada con cinta
  roja: va encima de todo lo que no la tenga. En código es una tupla que Rust compara pieza por
  pieza: `(importante, origen, especificidad, orden)`.
- **Por qué los selectores se leen de derecha a izquierda.** Piensa en `nav ul a` como "un `a`
  que vive dentro de un `ul` que vive dentro de un `nav`". Si empiezas por la izquierda, tienes
  que bajar por todos los descendientes de cada `nav` probando caminos, como buscar a alguien
  recorriendo todo el árbol genealógico desde el bisabuelo. Si empiezas por la derecha, el 95%
  de los elementos se descartan en la primera pregunta ("¿eres un `a`?") y, para los que sí, solo
  hay un camino hacia arriba: tu padre, tu abuelo… cada nodo tiene un solo padre. Es la columna
  `padre_id` de la arena (fase 2) que por fin se usa: `SELECT padre_id` en bucle hasta la raíz.
- **Herencia: el ADN baja, la ropa no.** Las propiedades de texto (`color`, `font-size`,
  `font-weight`, `text-align`, `line-height`) pasan de padre a hijo sin que nadie lo pida; por eso
  basta con `body { color: #333 }` para teñir toda la página. Las de la caja (márgenes, bordes,
  fondo, ancho) no: si el `div` tiene borde, su `p` no hereda otro borde. `inherit` obliga a
  heredar algo que no se hereda; `initial` vuelve al valor de fábrica; `unset` hace "lo natural"
  de cada propiedad.
- **`em` es una regla elástica.** `1.5em` significa "1.5 veces la letra". En `font-size`, la letra
  es la del padre (si no, la medida dependería de sí misma); en `margin` o `width`, la propia.
  Por eso los `em` anidados se multiplican: 18 × 1.2 × 1.2 × 1.2 = 31.1px en la página 05. Aquí
  todo `em` se convierte a px; los `%` no, porque dependen del ancho del bloque contenedor y eso
  solo lo sabe el layout.
- **`line-height: 1.5` contra `line-height: 24px`.** Un número se hereda como receta ("1.5 veces
  tu letra"); una longitud, como resultado ("24px, punto"). Con la receta, un título grande dentro
  de un párrafo tiene líneas proporcionalmente altas. Como el `Estilo` del contrato solo guarda
  px, el motor lleva aparte, de padre a hijo, la forma original (`AltoLinea::Factor` o `Px`).
- **`currentcolor`: "el color de la tinta que tengo".** `border: 1px solid` sin color usa el
  `color` del propio elemento. Por eso se calcula primero `color` y después los bordes.
- **Un valor desconocido no existe.** `display: banana` no borra el `display: block` del
  navegador: la cascada guarda todas las candidatas ordenadas y el cálculo toma la primera que
  entiende. Es lo que permite que CSS nuevo no rompa navegadores viejos.
- **`HashMap` en Rust:** un diccionario clave → valor, como un objeto de JavaScript o un hash de
  Redis. Aquí guarda "propiedad → candidatas ordenadas".
- **Closures (`|v| ...`):** funciones anónimas, como `v => ...` en JavaScript. Cada propiedad
  pasa su propia closure que sabe interpretar sus valores; el mecanismo de herencia es el mismo
  para todas.

## Decisiones

- 4.1 emparejamiento ingenuo, delegada a Claude. Detalles de implementación tomados dentro de ella:
  - La cascada guarda todas las candidatas (no solo la ganadora) para ignorar valores inválidos
    y para poder explicar qué ganó (`--estilo <archivo> <selector>`).
  - `line-height` con número se hereda como factor; documentado en el contrato.
  - El subrayado se propaga como en CSS: un hijo no puede quitarlo.
  - La raíz del árbol (el nodo Documento) es un bloque.
  - `--estilo` muestra lo heredable solo donde cambia respecto al padre, para no repetir el mismo
    color en cada línea.

## Lo que salió mal o quedó pendiente

- Todo compiló y pasó a la primera; el único ajuste fue de presentación: el resumen de bordes
  mostraba los colores de los lados sin borde (`borde 0 0 0 3px #333 #333 #333 #0f6e6e`).
- La deuda de la fase 2 sobre `<html>/<body>` implícitos no hizo falta para los estilos por
  defecto, pero una página sin `<body>` no recibe los 8px de margen. Sigue anotada.
- Ocho deudas nuevas en `DECISIONES.md`: `display` reducido a tres valores, dos pesos de letra,
  `justify`, estilos de borde, `line-through`, `:link`/`[hidden]`, `line-height: normal` fijo y
  `pre` sin monoespaciada.
