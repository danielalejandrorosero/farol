# Bitácora — Fase 5: layout de bloques (el modelo de caja)

Fecha: 2026-11-18. Estado: cerrada.

## Qué se construyó

- `src/layout/arbol.rs`: convierte el árbol estilado en un árbol de cajas. Cada bloque (`div`,
  `p`, `h1`…) da una caja; el texto suelto y los elementos en línea que quedan entre bloques se
  envuelven en **cajas anónimas**. Los saltos de línea entre `</h1>` y `<p>` no generan nada.
- `src/layout/bloque.rs`: el modelo de caja. Calcula anchos (`auto`, fijos, `%`, centrado con
  `margin: 0 auto`, `max-width`), apila los bloques uno debajo de otro y calcula los altos.
- `src/layout/en_linea.rs`: un alto **provisional** para el texto, mientras llega la fase 8.
- `src/layout/tipos.rs` (`Rect`, `Dimensiones`, `TipoCaja`, `Caja`) e `imprimir.rs`.
- `farol --layout <archivo> [ancho]`: imprime el árbol de cajas con la posición y el tamaño de
  cada una (ancho por defecto: 800).
- `pruebas/06-cajas.html`: anchos fijos y en %, centrado, `max-width`, relleno y bordes, bloques
  anidados, `height` fijo y un párrafo largo; las cuentas esperadas van en comentarios.
- 21 pruebas automáticas nuevas (162 en total).

## Hito

`cargo run -- --layout pruebas/01-hola.html`:

```
#documento bloque · (0, 0) 800 × 263.68
└── <html> bloque · (0, 0) 800 × 263.68
    └── <body> bloque · (8, 8) 784 × 247.68 · margen 8
        ├── <h1> bloque · (8, 29.44) 784 × 38.4 · margen 21.44 0
        │   └── anónima · (8, 29.44) 784 × 38.4 · "Hola, mundo"
        ├── <p> bloque · (8, 105.28) 784 × 19.2 · margen 16 0
        │   └── anónima · (8, 105.28) 784 × 19.2 · "Primera página de prueba del navegador."
        └── <div class="caja"> bloque · (40, 172.48) 720 × 51.2 · margen 20 · borde 2 · relleno 10
            └── <p> bloque · (40, 188.48) 720 × 19.2 · margen 16 0
                └── anónima · (40, 188.48) 720 × 19.2 · "Una caja con negrita, cursiva y un enlac…"
```

`(x, y)` es la esquina del **contenido** y `ancho × alto` su tamaño. Los anchos y las `x`
coinciden con Chrome: el contenido de la `.caja` empieza en 8 + 20 + 2 + 10 = 40 y mide
800 − 2·(8 + 20 + 2 + 10) = 720. Las `y` no: Chrome pone el `h1` en 21.44 porque fusiona el
margen del `body` (8) con el del `h1` (21.44) y se queda con el mayor; Farol los suma (decisión
5.1). Es la diferencia esperada y está anotada como deuda.

## Conceptos nuevos

- **El modelo de caja: cajas de cartón dentro de cajas.** Toma una caja de mudanza. Adentro va
  lo que guardas (el **contenido**). Entre lo guardado y las paredes pones papel de burbujas: el
  **relleno** (`padding`). Las paredes de cartón tienen grosor: el **borde**. Y cuando apilas la
  caja en el camión dejas aire alrededor para que no se golpee con las otras: el **margen**. En
  CSS, `width` mide solo lo de adentro; la caja "de verdad" que ves en pantalla es contenido +
  relleno + borde (por eso un `div` de `width: 200px; padding: 15px; border: 5px` ocupa 240).
  Farol guarda solo el rectángulo del contenido y el grosor de cada capa; los otros tres
  rectángulos se calculan sumando hacia afuera (`caja_relleno`, `caja_borde`, `caja_margen`).
- **El ancho baja del padre, el alto sube de los hijos.** Piensa en un estante de biblioteca:
  el ancho de cada repisa lo decide el mueble (no puedes saber cuánto mide la repisa sin saber
  cuánto mide el mueble), pero el alto de una pila de libros lo deciden los libros. En CSS es
  igual: `width: auto` es "todo lo que me deje mi padre" y `50%` es "la mitad del padre", así que
  primero se mide el padre y después los hijos; en cambio `height: auto` es "lo que midan mis
  hijos apilados", así que primero se miden los hijos. Un solo recorrido del árbol hace las dos
  cosas: al bajar reparte anchos; al volver, suma altos. Es como un `SELECT` recursivo que pasa
  un parámetro hacia abajo y agrega (`SUM`) hacia arriba.
- **La ecuación de las siete medidas.** Horizontalmente, CSS exige que margen + borde + relleno +
  ancho + relleno + borde + margen sume exactamente el ancho del padre. Lo que sobra se lo lleva
  lo que diga `auto`: si es el `width`, la caja se estira; si son los dos márgenes, se lo reparten
  mitad y mitad y la caja queda centrada (el famoso `margin: 0 auto`); si nada es `auto`, el
  margen derecho se estira en silencio (por eso un `div` de 300 px "termina" en el borde derecho,
  aunque no lo veas). `max-width` es un tope: se calcula normal y, si se pasó, se repite la
  cuenta con el tope como ancho.
- **Los `%` de arriba y abajo también miran el ancho.** `padding-top: 10%` es el 10 % del
  **ancho** del padre, no de su alto. Parece raro, pero el alto del padre todavía no se sabe
  cuando se mide el hijo (sube de los hijos), así que CSS usa lo único que ya conoce.
- **Cajas anónimas: el bloque que nadie escribió.** En `<div>hola <p>chao</p></div>` el "hola"
  no está dentro de ninguna etiqueta de bloque, pero tiene que ocupar su propia franja encima del
  `p`. El navegador inventa una caja sin etiqueta para él. Es como las cajas sin rótulo que
  aparecen en el camión de mudanza para lo que quedó suelto. Son invisibles: heredan la letra
  del padre, pero no su fondo ni sus bordes.
- **`Cow` en Rust ("copy on write").** Un valor que puede ser **prestado** (apunta a algo que ya
  existe, sin copiarlo) o **propio** (una copia que es suya). Se lee igual en los dos casos. Las
  cajas de bloque prestan el estilo de su nodo; las anónimas, que no tienen nodo, llevan uno
  propio. Parecido a un objeto JavaScript que a veces es una referencia compartida y a veces un
  clon, pero con el compilador vigilando quién es dueño de qué.
- **Un alto honesto pero provisional.** Para saber cuántas líneas ocupa un párrafo hay que
  partirlo palabra por palabra con la fuente: eso es la fase 8. Mientras tanto se estima: se mide
  todo el texto como si fuera una sola línea (con los espacios colapsados como en HTML) y se
  divide entre el ancho disponible. Se queda corta cuando una palabra no cabe al final de una
  línea, pero da altos razonables para ver la página.

## Decisiones

- 5.1 sin colapso de márgenes y 5.2 pruebas automáticas del árbol de cajas, delegadas a Claude.
- Ajuste al contrato de `layout` (detalle de implementación, no de arquitectura): `TipoCaja`
  lleva el `NodoId` o los nodos en línea que envuelve, y `estilo` es `Cow` para que las cajas
  anónimas tengan su propio estilo invisible. Documentado en `docs/ARQUITECTURA.md`.

## Lo que salió mal o quedó pendiente

- Nada falló al compilar ni en las pruebas.
- Deudas anotadas en `DECISIONES.md`: sin colapso de márgenes, alto de texto estimado, bloque
  dentro de un elemento en línea simplificado, sin `box-sizing`, `height` en % como `auto` y sin
  `min-*`/`max-height`, y sin `float`/`position`/`overflow`.
- Para probar: `cargo run -- --layout pruebas/06-cajas.html 400` y compara con 800. Los anchos
  `auto` y `%` cambian, los fijos no, y el párrafo largo gana líneas y empuja todo hacia abajo.
