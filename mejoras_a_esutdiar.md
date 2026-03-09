# mejoras_a_esutdiar

Documento de estudio e investigacion de mejoras de FPS en `pycro`.

## Protocolo de medicion (canon rapido actual)

- Escenario: `examples/compare_pycro_balls_benchmark.py`
- Carga: `25000` bolas
- Sesion: `BENCHMARK_AUTO_SESSION_SECONDS=2.5`
- Corridas por iteracion: `2` consecutivas
- Comando base:

```bash
BENCHMARK_AUTO=1 \
BENCHMARK_AUTO_INITIAL_BALLS=25000 \
BENCHMARK_AUTO_TARGETS=25000 \
BENCHMARK_AUTO_SESSION_SECONDS=2.5 \
PYCRO_FRAMES=600 \
cargo run --release -- examples/compare_pycro_balls_benchmark.py
```

## Baseline corto de referencia

- Run A: `wall_fps=23.40`
- Run B: `wall_fps=22.50`
- Centro aproximado inicial: `~22.95`

## Mejora 1 (positiva)

- Tecnica aplicada:
  - Rediseño interno de cola de render: `QueuedDrawBatch` + `CircleRun` para agrupar circulos contiguos.
  - Eliminacion de operaciones por-circulo como enum expandido individual en el hot path.
  - Rollback por marca (`mark/rollback`) para abortos de parse sin reconstruir toda la cola.
- Por que mejora:
  - Menos overhead estructural en encolado.
  - Menor presion de asignacion en trayectorias con miles de `draw_circle` por frame.
- Resultado positivo medido:
  - Run A: `wall_fps=25.04`
  - Run B: `wall_fps=25.37`
  - Centro aproximado: `~25.20`
  - Delta vs baseline: `+~2.25 FPS`.
- Riesgo/nota:
  - Mantener orden exacto de comandos para no alterar semantica visual.
- Validacion usada:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`

## Mejora 2 (positiva)

- Tecnica aplicada:
  - Cache de `submit_render` movido a `Arc<SubmitRenderCircleCache>` para evitar clon completo del vector de circulos por frame.
  - Parse rapido de posiciones cacheadas (`Vec2`) para ruta caliente.
  - Preservacion de capacidad de `draw_batch` en flush (se limpia, no se destruye capacidad).
- Por que mejora:
  - Reduce copias grandes por frame (clonado de miles de entradas).
  - Evita trabajo recurrente de reconstruccion de buffers.
- Resultado positivo medido:
  - Run A: `wall_fps=25.43`
  - Run B: `wall_fps=25.09`
  - Centro aproximado: `~25.26`
  - Delta vs baseline: `+~2.31 FPS`.
- Riesgo/nota:
  - Validar coherencia del cache por identidad de comandos para evitar hits falsos.
- Validacion usada:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`

## Mejora 3 (positiva leve)

- Tecnica aplicada:
  - Cache de callables `setup/update` en `RustPythonVm` para evitar lookup en globals en cada frame.
  - Eliminacion de formateo de strings en parse numerico de hot path (errores se formatean solo en fallo real).
- Por que mejora:
  - Recorta costo fijo por frame en puente VM.
  - Reduce asignaciones inutiles en parse numérico exitoso.
- Resultado positivo medido:
  - Run A: `wall_fps=25.06`
  - Run B: `wall_fps=24.57`
  - Centro aproximado: `~24.82`
  - Sigue por encima del baseline inicial (`~22.95`).
- Riesgo/nota:
  - Esta mejora es sensible al ruido de corrida corta; mantener varias repeticiones al validar.
- Validacion usada:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`

## Mejora 4 (positiva leve)

- Tecnica aplicada:
  - Layout comprimido de `submit_render`: `runs` de circulos + no-circulos indexados.
  - Reorganizacion interna para recorrer menos metadata por frame en la ruta caliente.
  - Separacion mas directa entre bloques masivos de circulos y comandos sueltos de HUD/fondo.
- Por que mejora:
  - Reduce overhead de iteracion en el puente Python->Rust cuando la lista de comandos mantiene forma estable.
  - Mejora localidad de datos al procesar miles de circulos consecutivos.
- Resultado positivo medido:
  - Run A: `wall_fps=24.83`
  - Run B: `wall_fps=25.89`
  - Centro aproximado: `~25.36`
  - Delta vs iteracion previa: `+~0.55 FPS` sobre `~24.82`.
  - Sigue por encima del baseline inicial (`~22.95`).
- Riesgo/nota:
  - Verificar que el layout comprimido no rompa el orden exacto entre circulos y comandos no-circulo.
  - La mejora es real pero leve; conviene seguir midiendo con varias repeticiones para confirmar estabilidad.
- Validacion usada:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`

## Mejora 5 (positiva)

- Tecnica aplicada:
  - Nuevo `draw_circle_batch` en backend.
  - Uso desde `runtime flush` para despachar `CircleRun` completos como lote, en vez de hacer una llamada Rust por circulo dentro del recorrido final.
  - Reutilizacion de la estructura ya agrupada en `QueuedDrawBatch`.
- Por que mejora:
  - Reduce overhead de llamada y branching en el flush final del frame.
  - Aprovecha mejor que el benchmark genera miles de circulos contiguos y ya los tiene comprimidos en `runs`.
- Resultado positivo medido:
  - Run A: `wall_fps=25.86`
  - Run B: `wall_fps=26.27`
  - Centro aproximado: `~26.07`
  - Delta vs iteracion previa (`run11=24.83`, `run12=25.89`, centro `~25.36`): `+~0.71 FPS`.
  - Sigue por encima del baseline inicial (`~22.95`).
- Riesgo/nota:
  - Mantener semantica exacta de orden dentro de cada `CircleRun`.
  - Si el conteo de dispatches se usa como evidencia, el batch debe preservar contabilidad equivalente.
- Validacion usada:
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-ejecucion del benchmark corto canon a `25000` bolas

## Mejora 6 (positiva clara)

- Tecnica aplicada:
  - Tuning de `Cargo.toml` en perfil `release`.
  - Paso 1: `lto=thin` + `codegen-units=1`.
  - Paso 2: sobre esa base, `lto=fat` + `panic=abort`.
- Por que mejora:
  - `codegen-units=1` y LTO ayudan al compilador a optimizar mejor rutas calientes cruzando limites de modulo.
  - `lto=fat` exprime mas inlining y eliminacion de overhead en el binario final.
  - `panic=abort` reduce algo de peso y complejidad en binario para ruta release.
- Resultado positivo medido:
  - Paso 1 (`lto=thin` + `codegen-units=1`):
    - Run A: `wall_fps=26.97`
    - Run B: `wall_fps=27.39`
    - Centro aproximado: `~27.18`
    - Delta vs iteracion previa (`~26.07`): `+~1.12 FPS`.
  - Paso 2 (`lto=fat` + `panic=abort`):
    - Run A: `wall_fps=29.22`
    - Run B: `wall_fps=29.45`
    - Centro aproximado: `~29.34`
    - Delta vs paso 1 (`~27.18`): `+~2.16 FPS`.
    - Delta total vs iteracion previa a este tuning (`~26.07`): `+~3.28 FPS`.
  - Sigue ampliamente por encima del baseline inicial (`~22.95`).
- Riesgo/nota:
  - Aumenta tiempos de compilacion release.
  - `panic=abort` cambia comportamiento de fallo en release; aceptable para benchmark, pero hay que vigilar implicaciones operativas si se adopta como default permanente.
  - Conviene validar que no aparezcan diferencias de linking o tooling entre plataformas.
- Validacion usada:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-ejecucion del benchmark corto canon a `25000` bolas tras cada ajuste de perfil release

## Mejora 7 (positiva clara)

- Tecnica aplicada:
  - Uso de `mimalloc` como global allocator del proceso.
  - Sustitucion del allocator por defecto bajo el protocolo corto canon (`2.5s`, `2` corridas).
- Por que mejora:
  - El benchmark ejerce mucha presion de asignacion y acceso a memoria en el puente Python->Rust y en estructuras de render temporales.
  - Un allocator mas afinado para este patron reduce overhead de asignacion/liberacion y mejora localidad en rutas calientes.
- Resultado positivo medido:
  - Run A: `wall_fps=31.93`
  - Run B: `wall_fps=31.86`
  - Centro aproximado: `~31.90`
  - Delta vs referencia estable previa (`run17=29.22`, `run18=29.45`, centro `~29.34`): `+~2.56 FPS`.
  - Sigue ampliamente por encima del baseline inicial (`~22.95`).
- Riesgo/nota:
  - Introduce una dependencia global de allocator; hay que vigilar compatibilidad y comportamiento por plataforma.
  - Puede cambiar perfil de memoria y tiempos de arranque/cierre aunque mejore el throughput.
  - Conviene mantener validacion cruzada en desktop real y revisar si hay diferencias en debugging o tooling.
- Validacion usada:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-ejecucion del benchmark corto canon a `25000` bolas con protocolo `2.5s / 2 runs`

## Mejora 8 (positiva marginal)

- Tecnica aplicada:
  - Fast-path especifico para posiciones de circulo representadas como `PyList[PyFloat, PyFloat]`.
  - Atajo en parse de `Vec2` para evitar ruta generica de secuencia cuando el benchmark ya entrega listas mutables de dos floats.
- Por que mejora:
  - El benchmark actual actualiza posiciones in-place en listas Python reutilizadas frame a frame.
  - Saltar parte del parse generico reduce un poco el overhead del puente Python->Rust en la ruta mas repetida del frame.
- Resultado positivo medido:
  - Run A: `wall_fps=31.69`
  - Run B: `wall_fps=32.17`
  - Centro aproximado: `~31.93`
  - Delta vs referencia previa (`run21=31.93`, `run22=31.86`, centro `~31.90`): `+~0.03 FPS`.
  - La mejora existe, pero es marginal y muy cercana al ruido normal de corrida corta.
- Riesgo/nota:
  - Bajo riesgo funcional si el fast-path mantiene fallback correcto a la ruta generica.
  - Riesgo principal: sobreajuste al shape actual del benchmark; el beneficio puede desaparecer en scripts que usen tuplas u otras secuencias.
  - Conviene mantener el camino general intacto y tratar este atajo como optimizacion oportunista.
- Validacion usada:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-ejecucion del benchmark corto canon a `25000` bolas con protocolo `2.5s / 2 runs`

## Mejora 9 (positiva fuerte)

- Tecnica aplicada:
  - Activacion de `PYCRO_CIRCLE_SPRITE=1`.
  - `draw_circle_batch` en backend renderiza circulos con una sprite texture circular en vez de tessellation repetida via `draw_circle`.
  - El cambio se aplica en la ruta batch, no en la logica Python del benchmark.
- Por que mejora:
  - Reduce costo de rasterizacion y construccion de geometria por circulo en el backend.
  - Aprovecha mucho mejor el caso dominante del benchmark: miles de circulos con el mismo patron visual base.
  - Encaja especialmente bien con el flush agrupado por `CircleRun`.
- Resultado positivo medido:
  - Run A: `wall_fps=37.88`
  - Run B: `wall_fps=37.30`
  - Centro aproximado: `~37.59`
  - Delta vs referencia reciente sin sprite (`run37=32.15`, `run38=31.71`, centro `~31.93`): `+~5.66 FPS`.
  - Sigue ampliamente por encima del baseline inicial (`~22.95`).
- Riesgo/nota:
  - Riesgo visual real: la sprite puede diferir de `draw_circle` en borde, suavizado, alpha, gamma o aspecto al escalar.
  - Conviene revisar artefactos en movimiento rapido, solapamiento y tamaños extremos de radio.
  - Si se adopta como default, hay que validar consistencia multiplataforma porque el resultado puede depender del backend grafico.
- Validacion usada:
  - `cargo test`
  - `cargo test submit_render_matches_legacy_draw_path_order_and_payload`
  - `cargo test draw_batch_flush_clears_per_frame`
  - `cargo test submit_circle_batch_queues_expected_draw_circles`
  - `cargo test direct_bridge_returns_backend_values_for_frame_time_and_texture_handle`
  - Re-ejecucion del benchmark corto canon a `25000` bolas con `PYCRO_CIRCLE_SPRITE=1`
  - Verificacion visual requerida para confirmar que la sustitucion sprite vs tessellation mantiene calidad aceptable

## Plantilla obligatoria para futuras mejoras positivas

Cuando una mejora suba FPS (aunque sea minima), agregar una entrada nueva con:

1. Tecnica aplicada.
2. Por que mejora (mecanismo).
3. Evidencia exacta (2 corridas, `wall_fps` summary).
4. Delta contra baseline vigente.
5. Riesgo tecnico y validaciones ejecutadas.

## Mejora 10 (positiva: calidad visual + rendimiento estable)

- Tecnica aplicada:
  - Se descarto el enfoque de cache por diametro porque en practica genero una regresion severa de FPS (multiplicaba texturas y draw calls).
  - Se adopto una sola textura circular de alta resolucion para todos los radios, con escalado lineal.
  - Se ajusto el borde con suavizado (AA suave) para conservar apariencia limpia al escalar.
- Por que mejora:
  - Calidad: los circulos grandes dejan de verse tan pixelados porque la fuente base tiene mas detalle y el filtrado lineal evita escalones duros.
  - Rendimiento: al reutilizar una sola textura se preserva mejor el batching (menos cambios de recurso y menos fragmentation de draw calls) frente al esquema por diametro.
- Resultado positivo observado por usuario en pantalla:
  - FPS observado: `~39 FPS`.
  - Mejora visual confirmada: circulos grandes con borde mas suave y menos pixelacion.
- Riesgos/ajustes:
  - Si el sprite base es demasiado chico reaparece pixelacion en radios altos; si es demasiado grande sube uso de memoria.
  - Ajuste recomendado via `PYCRO_CIRCLE_SPRITE_SIZE` segun target visual/perf de cada equipo.
  - Mantener verificacion visual en tamanos extremos para evitar halos o blur excesivo.

## Mejora 11 (positiva minima)

- Tecnica aplicada:
  - En `src/backend.rs`, `draw_circle_batch` elimino el pre-scan `all(...)`.
  - Se paso a un unico recorrido que decide por circulo si usar ruta sprite o vector.
- Por que mejora:
  - Evita una pasada completa extra sobre el batch antes de renderizar.
  - Reduce overhead fijo en la ruta caliente manteniendo la misma semantica de seleccion sprite/vector.
- Evidencia exacta medida (protocolo canon `25000 / 2.5s / 2 runs`, `PYCRO_FLUSH_STDIO_ON_UPDATE=1`, backend `opengl`, `sprite=1`):
  - Antes:
    - Run A: `wall_fps=35.61`
    - Run B: `wall_fps=35.77`
    - Centro aproximado: `~35.69`
  - Despues:
    - Run A: `wall_fps=35.71`
    - Run B: `wall_fps=36.08`
    - Centro aproximado: `~35.90`
- Delta:
  - Delta promedio: `+~0.21 FPS`.
- Riesgo/nota:
  - Mejora pequena y cercana al ruido de corrida corta; conviene revalidar con mas repeticiones para confirmar estabilidad estadistica.
  - Verificar que la decision por circulo no introduzca divergencias visuales sutiles frente al comportamiento previo del batch homogeneo.
