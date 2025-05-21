# JS + Rust + WebAssembly

Este proyecto combina **JavaScript**, **Rust** y **WebAssembly (Wasm)**. Usa [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) para compilar código Rust a WebAssembly, que luego puede ser usado desde JavaScript.

## Requisitos

Antes de empezar, necesitás tener instaladas algunas herramientas básicas:

### 1. Instalar Rust

Seguí las instrucciones oficiales en:

👉 [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

Después de instalar, verificá que Rust esté disponible:

```bash
rustc --version
```

### 2. Instalar wasm-pack

`wasm-pack` es una herramienta oficial para compilar proyectos Rust a WebAssembly.

Seguí las instrucciones oficiales en:

👉 [https://rustwasm.github.io/wasm-pack/installer](https://rustwasm.github.io/wasm-pack/installer)

Después de instalar, verificá que esté wasm-pack esté disponible:

```bash
wasm-pack --version
```

### 3. Instalar Node.js y npm

Este proyecto usa JavaScript, así que necesitás tener Node.js y su gestor de paquetes npm:

* Recomendado: instalar desde [https://nodejs.org/](https://nodejs.org/)
* O usando un gestor de versiones como [nvm](https://github.com/nvm-sh/nvm):

Verificá que estén instalados:

```bash
node -v
npm -v
```

---

## Estructura del Proyecto

```
.
├── rust/               # Código fuente en Rust
├── js/                 # Proyecto JavaScript
│   └── src/
│       └── wasm/       # Paquete generado por wasm-pack
├── build.sh            # Script para compilar Rust a Wasm
```

---

## Cómo ejecutar el Proyecto

### Linux & macOS

1. Asegurate de que tenés todos los requisitos instalados.
2. Abrí una terminal en la raíz del proyecto.
3. Ejecutá el script `build.sh`:

    ```bash
    sh build.sh
    ```

    El script `build.sh`:

    * Compila el código Rust ubicado en la carpeta `rust/`.
    * Genera un package WebAssembly.
    * Coloca los archivos generados en `js/src/wasm/`.

4. Compilá y ejecutá el proyecto JavaScript:

```bash
cd js
npm install
npm run dev
```
---

## Licencia

MIT