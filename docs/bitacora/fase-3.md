# Bitácora — Fase 3: parser de CSS

Fecha: 2026-11-18. Estado: cerrada.

## Qué se construyó

- `src/css/tokenizador.rs`: corta el texto en tokens: nombres, números con unidad (`10px`,
  `1.5em`, `50%`), `#hash`, cadenas, funciones (`rgb(`), `url(...)`, `!important`, símbolos y
  espacios. Los comentarios `/* */` desaparecen aquí, estén donde estén.
- `src/css/parser.rs`: arma las reglas (selectores + bloque de declaraciones) y se recupera de los
  errores sin perder el resto de la hoja.
- `src/css/selectores.rs`: etiqueta, `.clase`, `#id`, `*`, combinaciones pegadas
  (`div.caja#x`) y el espacio como combinador descendiente (`nav a`). Cualquier otro selector
  descarta la regla completa.
- `src/css/valores.rs`: longitudes, números, palabras clave y colores (unos 50 nombres más `transparent`, `#rgb`,
  `#rrggbb`, con alfa, `rgb()` y `rgba()`), con una validación ligera por tipo de propiedad.
- `src/css/abreviadas.rs`: `margin`, `padding`, `border`, `border-top`…, `border-width/-style/-color`
  y `background` se expanden a sus propiedades largas.
- `src/css/tipos.rs`: lo que se entrega a la fase 4, con la especificidad de cada selector.
- `farol --css <archivo>`: imprime las reglas, su especificidad y la lista de lo descartado.
- `pruebas/03-estilos.css` (CSS realista) y `pruebas/04-roto.css` (nueve tipos de error).
- 57 pruebas automáticas nuevas (81 en total), incluida una que le lanza 3000 textos al azar.

## Hito

`cargo run -- --css pruebas/03-estilos.css` imprime 11 reglas y 85 declaraciones.
`cargo run -- --css pruebas/04-roto.css` no se cae: se queda con 8 reglas y lista los 20 pedazos
que descartó, cada uno con su motivo. Un fragmento:

```
Regla 2: p (0,0,1)
  color: #0000ff
  line-height: 1.4
...
Descartado (20):
  - Se descartó `color: banana`: valor no válido para esta propiedad
  - Se descartó la regla `ul > li`: selector no soportado
```

## Conceptos nuevos

- **Un color son tres lucecitas y un velo.** Con una lupa sobre el monitor se ven tres
  lucecitas por pixel: roja, verde y azul. `#ff8800` es la intensidad de cada una, de 0 a 255
  (`ff` = 255 la roja a tope, `88` = 136 la verde a media, `00` la azul apagada). Cada número
  cabe en un byte (`u8` en Rust: un entero sin signo de 8 bits, de 0 a 255). El cuarto número,
  alfa, no es una luz: dice cuánto deja ver lo de atrás, como un vidrio más o menos empañado.
  `transparent` es negro con alfa 0, un vidrio limpio. `rgba(0,0,0,.5)` llega como alfa 128.
- **Medidas absolutas: todas son pixeles disfrazados.** CSS define que una pulgada son 96 px
  (no importa el tamaño real de la pantalla), así que `12pt`, `1in` o `2cm` se convierten a px
  al parsear. `em` y `%` no se pueden convertir aún: dependen del elemento (de su tamaño de letra
  o del ancho del padre) y se resuelven en las fases 4 y 5.
- **Recuperación de errores: contar llaves como paréntesis en Excel.** Cuando una regla tiene un
  selector raro, el parser no busca la siguiente `}` a ciegas: lleva un contador que sube con cada
  `{` y baja con cada `}`, y la regla termina cuando vuelve a cero. Así `a:visited { x { y } }`
  se salta entera aunque traiga llaves adentro. Por eso un navegador de 2010 puede leer CSS de hoy:
  lo nuevo que no entiende queda dentro de límites claros (`;` para declaraciones, `}` balanceada
  para reglas) y lo salta sin romper lo demás.
- **Especificidad: como comparar versiones de software.** `(1,0,0)` le gana a `(0,9,9)` igual
  que la versión 1.0.0 es mayor que la 0.9.9: primero se comparan los ids, si empatan las clases,
  y al final las etiquetas. La fase 4 la usará para decidir qué regla gana.
- **Abreviadas: un atajo que se desarma al llegar.** `margin: 0 auto` es azúcar para cuatro
  propiedades. Se desarma al parsear para que el motor de estilos solo vea propiedades largas,
  como un ORM que convierte un atajo en las columnas reales de la tabla. Y lo que el atajo no dice
  vuelve a su valor inicial: `border: 1px solid` también resetea el color.
- **`Option<T>` en Rust:** un valor que puede estar o no estar: `Some(valor)` o `None`. Es como
  `null`, pero el compilador obliga a revisar el caso vacío antes de usarlo. Aquí "este valor no es
  válido" es un `None`, y quien lo recibe tiene que decidir qué hacer.
- **`Result<T, E>`:** como `Option`, pero el caso malo trae una explicación: `Ok(declaraciones)` o
  `Err("falta el :")`. Es de donde salen los motivos de la lista "Descartado".
- **`impl Display`:** enseñarle a un tipo a escribirse como texto, como `toString()` en
  JavaScript. Cada color, valor y selector sabe imprimirse en sintaxis CSS.

## Decisiones

- 3.1 subconjunto medio, delegada a Claude. Detalles de implementación tomados dentro de ella:
  - Validación ligera por tipo (colores y longitudes) para que `color: banana` no pise a un color
    válido anterior, como pasa en un navegador real.
  - Las unidades absolutas se convierten a px al parsear; `rem`, `vh` y `vw` se descartan (deuda).
  - Los valores que Farol no interpreta se guardan como texto en vez de descartarse.
  - Se agregó `parsear_con_avisos` al contrato para que `--css` explique qué descartó y por qué.

## Lo que salió mal o quedó pendiente

- Nada falló al compilar. Una sorpresa al probar `04-roto.css`: la basura suelta (`}}} ;;;`)
  entre reglas se pega al selector de la regla siguiente y la invalida. Se revisó la
  especificación: es lo correcto y Chrome hace lo mismo. Se dejó así y se explicó en el archivo.
- Seis deudas anotadas en `DECISIONES.md`: unidades relativas modernas, validación ligera,
  valores como texto, `background` solo color, selectores y at-rules fuera del subconjunto, y
  `var()`, `calc()`, `hsl()`.
