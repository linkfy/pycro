# Perf Phase 3 Closeout

## Estado actual consolidado
- Rama de trabajo: `codex/perf-phase3-clean-start`.
- Objetivo de referencia de usuario: acercar pycro al rango observado de pygame-ce (`41-43 FPS`) en `25000` bolas.
- Mejor configuración validada hasta ahora (sin regresión visual grave):
  - `PYCRO_APPLE_GFX_API=opengl`
  - `PYCRO_CIRCLE_SPRITE=1`
  - `PYCRO_CIRCLE_SPRITE_SIZE=768`
  - sin `PYCRO_SUBMIT_RENDER_DIRECT`
- Resultado observado más alto reciente en esta línea: ~`39 FPS` en pantalla (con variación por corrida breve).

## Cambios implementados en esta fase
- Optimización del bridge/runtime y batching de círculos en Rust.
- Caché de parsing para `submit_render` con rutas rápidas para posiciones.
- Sprite de círculo de alta resolución con borde suavizado.
- Control por comando de modo vector/sprite mediante `options`:
  - `draw_circle(..., options={"as_sprite": True|False})`
  - `submit_circle_batch(..., options={"as_sprite": True|False})`
  - `submit_render` acepta `draw_circle` con quinto argumento `options`.
- Ejemplo actualizado para toggle con `Space` y HUD de estado.
- Fix de cache dinámico para que el toggle de `options["as_sprite"]` se aplique en tiempo real.

## Pendientes (todo lo que falta)
1. Alcanzar y sostener `>=41 FPS` en protocolo corto `25000 / 2.5s / 2 runs` con evidencia reproducible.
2. Cerrar guardrails de regresión en `3000` y `6000` bolas (sin caída >5% contra baseline aceptado).
3. Congelar benchmark protocol en script/documentación para evitar drift:
   - misma duración, mismo run count, mismo stack/runtime metadata.
4. Ejecutar comparación final pycro vs pygame-ce con stack moderno fijo y tabla before/after final.
5. Consolidar checklist de aceptación de fase en tracker/state y generar checkpoint commit de rollback.
6. Completar `ci-visual-payload-smoke` (hoy sigue planned).
7. Completar gate de evidencia de playtest manual (hoy sigue planned).
8. Formalizar policy de selección backend por plataforma con ADR (OpenGL/Metal en macOS ARM, etc.).

## Propuestas a futuro (priorizadas)
1. Generalizar optimización vector->sprite a más primitivas geométricas.
2. Añadir `options` homogéneo para figuras no circulares (por ejemplo `draw_rect`, polígonos, líneas, shapes futuras).
3. Crear cachés de sprite por primitiva y estilo (relleno/borde/radio/AA) con política de invalidación.
4. Implementar batching unificado para primitivas vectoriales mixtas (minimizar draw calls por material/texture key).
5. Exponer knobs de calidad/rendimiento por primitiva (`aa`, `filter`, `sprite_size`, `as_sprite`) de forma consistente.
6. Añadir benchmarks dedicados por tipo de primitiva para medir impacto aislado de cada optimización.
7. Evaluar ruta de instancing/miniquad para lotes masivos cuando aplique.

## Recordatorio explícito
- Aplicar las mismas optimizaciones de figuras geométricas usadas en círculos al resto de elementos vectoriales es una línea de trabajo pendiente y prioritaria.
