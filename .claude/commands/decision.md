---
description: Registra una decisión de arquitectura en DECISIONES.md
argument-hint: <código o tema> <elección>
---

Registra en `DECISIONES.md` la decisión que el usuario acaba de tomar: $ARGUMENTS

1. Si ya existe la fila en la tabla, actualiza elección, estado (**Decidida**) y fecha de hoy.
   Si no existe, agrégala con un código nuevo que siga la numeración de su fase.
2. Añade o actualiza su entrada en "Detalle" con opciones, motivo y costo.
3. Si la decisión aprueba una dependencia, agrégala a "Dependencias aprobadas".
4. Si cambia la forma de un módulo, actualiza `docs/ARQUITECTURA.md`.
5. Confirma al usuario en una línea qué quedó registrado.
