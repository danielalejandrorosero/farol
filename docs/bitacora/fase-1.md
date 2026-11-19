# Bitácora — Fase 1: ventana y pixeles

Fecha: 2026-11-18. Estado: cerrada.

## Qué se construyó

- `src/pintado/mod.rs`: el `Lienzo`, una lista de `ancho × alto` colores, y la función que
  rellena rectángulos recortando lo que quede fuera. Tiene 4 pruebas automáticas.
- `src/ui/mod.rs`: abre la ventana con `winit`, atiende sus eventos y copia el lienzo a la
  pantalla con `softbuffer`. Es el único módulo que conoce esas dos librerías.
- `src/main.rs`: una escena de prueba (barra superior, tres columnas, franja inferior) cuyas
  medidas dependen del tamaño de la ventana.

## Hito

La ventana "Farol 0.1.0" muestra la escena y se redibuja bien al cambiarle el tamaño. Verificado
con capturas a 900×600 y a 600×750: las columnas se reparten el ancho nuevo y la franja amarilla
sigue pegada abajo.

## Conceptos nuevos

- **Framebuffer:** la pantalla es una lista de números. El pixel `(x, y)` está en la posición
  `y * ancho + x`. Cada número es un color `0x00RRGGBB`.
- **Bucle de eventos:** el programa no "corre" de arriba abajo; se queda esperando a que Windows le
  avise de algo (cerrar, redimensionar, repintar) y reacciona. Es como un servidor web esperando
  peticiones, pero las peticiones son eventos de la ventana.
- **`impl ApplicationHandler for App`:** en Rust, un *trait* es un contrato, parecido a una
  interfaz. `winit` exige que la aplicación cumpla ese contrato para poder avisarle de los eventos.
- **`Option` y `let ... else`:** Rust no tiene `null`. Un valor que puede faltar es un `Option`, y
  el compilador obliga a decidir qué hacer cuando falta (por ejemplo, la ventana minimizada mide 0).

## Decisiones

- 0.2 pasó de "cero absoluto" a "mínimas y declaradas".
- 1.1: `winit` + `softbuffer`. 1.2: pintado por CPU.

## Lo que salió mal

- La primera compilación falló: `error calling dlltool 'dlltool.exe': program not found`. El
  toolchain GNU de Rust trae su propio `dlltool`, pero apuntarle directamente tampoco bastó porque
  le faltaba el ensamblador. La solución fue instalar MinGW-w64 completo.
- Se avanzó a la fase 1 sin el commit de la fase 0, porque git aún no tiene nombre ni correo.
