# Bitácora — Fase 2: parser de HTML

Fecha: 2026-11-18. Estado: cerrada.

## Qué se construyó

- `src/html/tokenizador.rs`: convierte el texto en tokens. Entiende atributos con y sin comillas,
  mayúsculas, entidades (`&amp;`, `&#241;`), comentarios, `<!DOCTYPE>`, el contenido crudo de
  `<style>` y `<script>`, y un `<` suelto que no abre etiqueta.
- `src/html/constructor.rs`: arma el árbol con una pila y las reglas de tolerancia.
- `src/html/dom.rs`: la arena de nodos y la impresión del árbol con líneas.
- `farol --dom <archivo>`: imprime el árbol de una página.
- `pruebas/02-mal-escrito.html`: una página con errores a propósito.
- 20 pruebas automáticas nuevas (24 en total).

## Hito

`cargo run -- --dom pruebas/01-hola.html` imprime el árbol completo. La página mal escrita también
produce un árbol sensato, sin fallar.

## Conceptos nuevos

- **Bytes y caracteres:** el archivo es una fila de bytes. `ñ` ocupa dos bytes en UTF-8, por eso
  el tokenizador trabaja con caracteres y no con bytes sueltos.
- **Tokenizador:** una máquina que lee de izquierda a derecha y cambia de modo según lo que ve
  (`<` empieza etiqueta, `/` dice que es de cierre, `=` separa clave y valor).
- **Pila:** la torre de platos. El de arriba es el padre de lo que llegue.
- **Arena:** todos los nodos en una lista; cada uno conoce a sus parientes por número de posición.
- **`enum` en Rust:** un tipo que puede ser una cosa u otra (`Abre`, `Cierra`, `Texto`). El
  compilador obliga a manejar todos los casos en un `match`; si se olvida uno, no compila.
- **Los espacios entre etiquetas existen:** en `<b>a</b> <i>b</i>` el espacio del medio es un
  nodo de texto. No se imprime, pero se guarda porque separa palabras en la fase 8.

## Decisiones

- 2.1 tolerante simple y 2.2 arena con índices, delegadas a Claude por el arquitecto.

## Lo que salió mal o quedó pendiente

- Nada falló al compilar.
- Tres deudas anotadas en `DECISIONES.md`: etiquetas cruzadas, `<html>/<head>/<body>` implícitos
  y entidades con nombre poco comunes.
