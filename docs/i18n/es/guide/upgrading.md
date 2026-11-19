# Actualización

<!-- languages:begin -->
[English](../../../guide/upgrading.md) · [Русский](../../ru/guide/upgrading.md) · [中文](../../zh/guide/upgrading.md) · **Español**
<!-- languages:end -->
<!-- translation of docs/guide/upgrading.md: f487335e37bf -->

Cómo pasar un proyecto de la versión de libphonenumber de la que depende a una más reciente. Busque la sección de cada
versión posterior a la suya y aplíquelas de la más antigua a la más reciente: pasar de 0.1.0 a una versión posterior
significa aplicar la sección de cada versión intermedia y luego la de la versión que quiere. Una versión que no
requiere cambios lo dice. Después, [actualice](#cómo-actualizar) a la versión más reciente en un solo paso: nunca hace
falta instalar antes una versión intermedia.

Cada sección enumera lo que la versión pide a su código: en **Action required**, un cambio que hay que hacer antes de
la actualización o junto con ella; en **Deprecated**, algo que todavía funciona y dejará de hacerlo en una versión
posterior; en **Removed**, algo que ya ha dejado de funcionar. El
[registro de cambios](https://github.com/oxidestack/libphonenumber/blob/main/CHANGELOG.md) recoge todo lo demás que cambió
cada versión. Solo las versiones tienen sección: una versión candidata (`-rc.N`) se describe en su release de GitHub, y
lo que cambia se escribe aquí bajo la versión que lo incluye.

La documentación de la versión que usa también está en este sitio: el selector de versión de la parte superior de
cada página cambia de una versión a otra, y una página de una versión antigua enlaza la misma página en la más reciente.

## Cómo actualizar

Las versiones siguen el [versionado semántico](https://semver.org/lang/es/spec/v2.0.0.html). Antes de 1.0, una nueva
versión menor (de 0.1 a 0.2) puede cambiar la API; una nueva versión de parche (de 0.1.0 a 0.1.1), no.

1. Lea las secciones siguientes de cada versión posterior a la suya y aplique sus cambios a su código.
2. Cambie la versión en su `Cargo.toml` (`libphonenumber = "0.2"`) o, para una versión de parche, ejecute
   `cargo update -p libphonenumber`.
3. Ejecute `cargo build` y sus pruebas. Un cambio de **Action required** que el compilador puede detectar aparece
   aquí; los demás son de comportamiento, y la sección dice cómo comprobarlos.
4. Una versión que pasa a metadatos más recientes de libphonenumber lo dice en **Action required**: la validez, el tipo o el formato de un número pueden cambiar con los metadatos, como en Java, así que las pruebas que fijan ese resultado pueden necesitar el nuevo.

## Actualizar a X.Y.Z (primera versión, número y fecha aún sin fijar)

<!-- Placeholder: replace "X.Y.Z" in this heading with the first release's
     number when it is cut (expected 0.1.0). Every later release gets its own section above this one. -->

La primera versión de libphonenumber: las partes de libphonenumber 9.0.41 (Java) que usa Datafaker 2.7.0, con los metadatos de libphonenumber 9.0.41. No hay nada desde lo que actualizar.

### Action required

Nada.

### Deprecated

Nada.

### Removed

Nada.
