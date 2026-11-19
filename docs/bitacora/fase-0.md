# Bitácora — Fase 0: entorno y esqueleto

Fecha: 2026-11-18. Estado: cerrada. Quedan dos decisiones provisionales por confirmar.

## Qué se construyó

- Rust 1.100.0 instalado con el toolchain GNU.
- El proyecto `farol`: `Cargo.toml` sin dependencias y `src/` con siete módulos vacíos, uno por
  etapa de la tubería.
- `cargo run` imprime `Farol 0.1.0`.
- Los documentos del proyecto: `CLAUDE.md`, `DECISIONES.md`, `docs/FASES.md`,
  `docs/ARQUITECTURA.md` y la guía en PDF.

## Conceptos nuevos

- **Crate:** una unidad de código compilable de Rust; equivale a un paquete de npm.
- **`Cargo.toml`:** el `package.json` de Rust. El nuestro tiene la sección `[dependencies]` vacía.
- **Módulo (`mod`):** una carpeta o archivo que agrupa código. `mod.rs` es su puerta de entrada,
  como un `index.js`.
- **Tubería:** los datos fluyen en un solo sentido, de `red` a `ui`; ningún módulo conoce al
  que viene después.

## Decisiones tomadas

- Nombre: Farol.
- Lenguaje: Rust.
- Dependencias mínimas y declaradas: cada librería la aprueba el arquitecto.
- Provisionales: toolchain GNU y un solo crate.

## Lo que salió distinto a lo planeado

- El PC no tenía Visual Studio, así que se usó el toolchain GNU en vez del MSVC habitual.
- La política de dependencias se registró primero como "cero absoluto" por un malentendido y se
  corrigió el mismo día a "mínimas y declaradas".
- `.claude/settings.json` (permisos preaprobados para `cargo`) no se creó: el modo automático
  lo bloqueó.
- Git no tiene nombre ni correo configurados, por eso aún no hay commit.
