# Raytracing: zbuffer

Implementación modular de la actividad de profundidad en Raytracing.

## Ejecutar

```bash
cargo run --release
```

El programa genera `zbuffer.ppm` con esferas superpuestas. La esfera cercana aparece primero en el arreglo para demostrar que el resultado depende de la distancia del impacto y no del orden de los objetos.

## Estructura

- `main.rs`: configura la escena y ejecuta el render.
- `color.rs`: define `Color`.
- `math.rs`: define `Vec3`.
- `material.rs`: define `Material`, `Intersect` y `RayIntersect`.
- `sphere.rs`: calcula la intersección rayo-esfera y su distancia.
- `scene.rs`: crea la escena con objetos superpuestos.
- `renderer.rs`: utiliza el zbuffer para seleccionar el impacto más cercano.
- `output.rs`: escribe el framebuffer en formato PPM.

## Zbuffer

Para cada rayo se inicializa:

```rust
let mut zbuffer = f32::INFINITY;
```

Cada intersección válida se compara con esa distancia. Si es menor, se convierte en el impacto seleccionado y su material determina el color del píxel.
