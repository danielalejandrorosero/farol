# Estado de las fases

Fase actual: **8**

| Fase | Nombre | Estado | Hito |
|---|---|---|---|
| 0 | Entorno y esqueleto | Cerrada | `cargo run` imprime nombre y versión; primer commit |
| 1 | Ventana y pixeles | Cerrada | Ventana con rectángulos de colores que se redibuja al redimensionar |
| 2 | Parser de HTML | Cerrada | `--dom pagina.html` imprime el árbol con sangría |
| 3 | Parser de CSS | Cerrada | `--css estilos.css` imprime las reglas; un CSS roto no lo tumba |
| 4 | Motor de estilos | Cerrada | Propiedades finales de cualquier nodo y qué regla ganó |
| 5 | Layout de bloques | Cerrada | Árbol de cajas con coordenadas que coinciden con Chrome |
| 6 | Pintado y display list | Cerrada | Primera página HTML con CSS dibujada por el motor |
| 7 | Texto y fuentes | Cerrada | "Hola, mundo" con tildes en varios tamaños y colores |
| 8 | Layout en línea | Pendiente | Un artículo se lee bien y se reacomoda al redimensionar |
| 9 | Redes: URL, DNS, TCP, HTTP | Pendiente | Carga `http://example.com` y `http://info.cern.ch`; captura de Wireshark |
| 10 | Recursos e imágenes | Pendiente | Página con CSS externo e imágenes completa |
| 11 | Interacción | Pendiente | Dirección, clic en enlace, atrás, scroll |
| 12 | HTTPS | Pendiente | Carga una página HTTPS real |

## Fase 0: lista de cierre

- [x] Nombre del navegador: Farol (decisión G.1)
- [x] Política de dependencias: mínimas y declaradas (decisión 0.2)
- [x] Proyecto creado con sus siete módulos vacíos; `cargo run` imprime `Farol 0.1.0`
- [x] Git configurado y primer commit
- [ ] Confirmar toolchain GNU (decisión 0.3) y un solo crate (decisión 0.4): siguen provisionales
