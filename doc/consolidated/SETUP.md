# Guía de Configuración e Instalación de `token-optimizer`

Este documento proporciona instrucciones detalladas para instalar la herramienta y configurar su entorno, incluyendo las dependencias para la capa de Machine Learning.

## 1. Requisitos Previos

-   **Rust:** Versión 1.70 o superior. Verifica con `rustc --version`.
-   **Git:** Necesario para la detección de cambios. Verifica con `git --version`.
-   **(Opcional) GPU NVIDIA:** Para la aceleración de ML, se requiere una GPU compatible con CUDA 12.8+.

## 2. Instalación de la Herramienta

```bash
# 1. Clonar el repositorio
git clone <url_del_repositorio>
cd token-optimizer

# 2. Compilar en modo de producción (optimizado)
cargo build --release

# 3. (Opcional) Instalar globalmente para acceso desde cualquier lugar
cargo install --path .

# 4. Verificar la instalación
token-optimizer --version
```

## 3. Configuración de la Capa de Machine Learning (Opcional)

Para habilitar las capacidades de análisis semántico, necesitas una GPU NVIDIA y los drivers correspondientes, CUDA y cuDNN.

### 3.1. Instalación de CUDA y cuDNN en Ubuntu

Esta guía está optimizada para Ubuntu y una RTX 3050, pero los pasos son similares para otras distribuciones y tarjetas.

**Paso 1: Instalar Drivers de NVIDIA y CUDA Toolkit**

Asegúrate de tener los drivers propietarios de NVIDIA y el CUDA Toolkit 12.8 o superior instalados. La forma más fácil suele ser a través del gestor de paquetes de tu distribución.

```bash
# Verificar instalación de drivers y CUDA
nvidia-smi
nvcc --version
```

**Paso 2: Instalar cuDNN (Recomendado: vía Repositorio NVIDIA)**

1.  **Añadir el keyring de NVIDIA:**
    ```bash
    wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb
    sudo dpkg -i cuda-keyring_1.0-1_all.deb
    sudo apt update
    ```

2.  **Instalar las librerías de cuDNN:**
    ```bash
    sudo apt install libcudnn8 libcudnn8-dev libcudnn8-doc
    ```

3.  **Verificar la instalación:**
    ```bash
    ldconfig -p | grep cudnn
    ```
    Deberías ver las librerías `libcudnn.so.8` listadas.

### 3.2. Descargar los Modelos de Lenguaje

Una vez configurado el entorno de GPU, descarga los modelos que la herramienta utilizará.

```bash
# Descargar todos los modelos recomendados (~18GB)
# Esto puede tardar un tiempo considerable.
token-optimizer ml models download --all

# Verificar que los modelos están en el cache
token-optimizer ml models status
```

La herramienta está diseñada con **degradación elegante**: si los modelos de ML o la configuración de GPU no están disponibles, continuará funcionando utilizando únicamente su robusto análisis AST.

## 4. Soporte para Nuevos Lenguajes (Ej: Rust)

La arquitectura modular de `token-optimizer` permite extender su soporte a nuevos lenguajes. El plan general es:

1.  **Extender Tipos:** Añadir nuevos `enum` a `FileType` en `src/types/mod.rs` (ej: `RustModule`, `RustCrate`).
2.  **Detección de Archivos:** Actualizar `detect_file_type()` en `src/utils/file_utils.rs` para reconocer las nuevas extensiones (ej: `.rs`, `Cargo.toml`).
3.  **Análisis Específico:** Implementar un nuevo analizador en `src/analyzers/` utilizando el parser de `tree-sitter` correspondiente (ej: `tree-sitter-rust`).
4.  **Actualizar Generadores:** Modificar los generadores de reportes en `src/generators/` para incluir la nueva información en los `overview`.
