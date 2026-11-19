# Arquitectura

El motor es una tubería. Cada etapa recibe una cosa y entrega otra; ninguna conoce a las que
vienen después.

```
URL → red → bytes → parsers → estilos → layout → display list → pintado → ventana
```

## Módulos

| Módulo | Recibe | Entrega | Fase |
|---|---|---|---|
| `red` | Una URL | Bytes de HTML, CSS e imágenes | 9, 10, 12 |
| `html` | Texto HTML | Árbol DOM | 2 |
| `css` | Texto CSS | Lista de reglas | 3 |
| `estilo` | DOM y reglas | Árbol de estilo: cada nodo con sus propiedades finales | 4 |
| `layout` | Árbol de estilo y ancho de ventana | Árbol de cajas con posición y tamaño | 5, 8 |
| `pintado` | Árbol de cajas | Display list y pixeles en el framebuffer | 6, 7 |
| `ui` | Eventos de ventana, mouse y teclado | Scroll, clics, navegación | 1, 11 |

## Reglas de dependencia entre módulos

- Las flechas van en un solo sentido: `layout` usa `estilo`, nunca al revés.
- `red` no sabe nada de HTML; solo entrega bytes y cabeceras.
- `ui` es el único módulo que conoce la ventana del sistema operativo.

## Por dentro de `html`

- `tokenizador.rs`: recorre el texto carácter por carácter y lo corta en tokens (abre, cierra,
  texto, comentario). El contenido de `style`, `script`, `title` y `textarea` se toma crudo.
- `constructor.rs`: arma el árbol con una pila de elementos abiertos y aplica las reglas de
  tolerancia (decisión 2.1).
- `dom.rs`: la arena (decisión 2.2). `NodoId` es la posición de un nodo en la lista.

Este documento se actualiza cuando una decisión cambia la forma de un módulo.
