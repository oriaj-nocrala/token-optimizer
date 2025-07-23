# 🚀 Instalación de cuDNN en Ubuntu 25.04

## 📋 **Requisitos previos**
- Ubuntu 25.04 (Plucky Pangolin)
- NVIDIA RTX 3050 8GB con driver 570.133.07
- CUDA 12.8 instalado
- Privilegios de administrador (sudo)

## 🔧 **Método 1: Instalación via repositorio NVIDIA (Recomendado)**

### 1. Descargar e instalar keyring CUDA
```bash
# Descargar keyring para Ubuntu 22.04 (compatible con 25.04)
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb

# Instalar keyring
sudo dpkg -i cuda-keyring_1.0-1_all.deb

# Actualizar repositorios
sudo apt update
```

### 2. Instalar cuDNN
```bash
# Instalar cuDNN 8 (compatible con CUDA 12.8)
sudo apt install libcudnn8 libcudnn8-dev libcudnn8-doc

# Verificar instalación
ldconfig -p | grep cudnn
```

### 3. Verificar instalación
```bash
# Debería mostrar algo como:
# libcudnn.so.8 (libc6,x86-64) => /usr/lib/x86_64-linux-gnu/libcudnn.so.8
# libcudnn_adv_infer.so.8 (libc6,x86-64) => /usr/lib/x86_64-linux-gnu/libcudnn_adv_infer.so.8
# libcudnn_adv_train.so.8 (libc6,x86-64) => /usr/lib/x86_64-linux-gnu/libcudnn_adv_train.so.8
```

## 🔧 **Método 2: Instalación via Snap (Alternativa)**

```bash
# Instalar cuDNN con snap
sudo snap install cudnn --classic

# Verificar instalación
snap list cudnn
```

## 🔧 **Método 3: Instalación manual (Para casos específicos)**

### 1. Descargar desde NVIDIA Developer
- Ir a https://developer.nvidia.com/cudnn
- Registrarse/iniciar sesión
- Descargar cuDNN 8.9.x para CUDA 12.x
- Descargar el archivo .deb para Ubuntu 22.04

### 2. Instalar manualmente
```bash
# Instalar archivo .deb descargado
sudo dpkg -i cudnn-local-repo-ubuntu2204-8.9.x.x_1.0-1_amd64.deb

# Agregar la clave del repositorio
sudo cp /var/cudnn-local-repo-ubuntu2204-8.9.x.x/cudnn-local-*-keyring.gpg /usr/share/keyrings/

# Actualizar e instalar
sudo apt update
sudo apt install libcudnn8 libcudnn8-dev
```

## ✅ **Verificación de la instalación**

### 1. Verificar archivos cuDNN
```bash
# Verificar bibliotecas instaladas
ldconfig -p | grep cudnn

# Verificar archivos de desarrollo
ls -la /usr/include/cudnn*
ls -la /usr/lib/x86_64-linux-gnu/libcudnn*
```

### 2. Verificar versión
```bash
# Verificar versión de cuDNN
cat /usr/include/cudnn_version.h | grep CUDNN_MAJOR -A 2
```

### 3. Probar compilación Rust
```bash
# Compilar proyecto con cuDNN
cargo check --tests

# Ejecutar tests VRAM
cargo test test_real_vram_loading_deepseek -- --test-threads=1
```

## 🔧 **Solución de problemas**

### Error: "unable to find library -lcudnn"
```bash
# Verificar ubicación de bibliotecas
sudo find /usr -name "libcudnn*" 2>/dev/null

# Agregar al path si es necesario
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH

# Actualizar cache de bibliotecas
sudo ldconfig
```

### Error: "CUDNN_STATUS_NOT_INITIALIZED"
```bash
# Verificar permisos
sudo chmod +r /usr/include/cudnn*
sudo chmod +r /usr/lib/x86_64-linux-gnu/libcudnn*
```

## 📊 **Configuración del sistema**

### Variables de entorno (opcional)
```bash
# Agregar a ~/.bashrc
export CUDNN_PATH=/usr/lib/x86_64-linux-gnu
export LD_LIBRARY_PATH=$CUDNN_PATH:$LD_LIBRARY_PATH
export LIBRARY_PATH=$CUDNN_PATH:$LIBRARY_PATH
export CPATH=/usr/include:$CPATH
```

### Verificar configuración CUDA
```bash
# Verificar instalación CUDA
nvidia-smi
nvcc --version

# Verificar dispositivos CUDA
nvidia-smi -L
```

## 🚀 **Después de la instalación**

### 1. Limpiar cache de Cargo
```bash
cargo clean
```

### 2. Recompilar proyecto
```bash
cargo build --release
```

### 3. Ejecutar tests VRAM
```bash
# Test individual
cargo test test_real_vram_loading_deepseek -- --test-threads=1 --nocapture

# Todos los tests VRAM
cargo test real_vram_test -- --test-threads=1 --nocapture
```

## 📈 **Rendimiento esperado**

Con cuDNN instalado, deberías ver:
- ✅ Uso real de VRAM en `nvidia-smi`
- ✅ Aceleración de operaciones tensoriales
- ✅ Mejor rendimiento en inferencia ML
- ✅ Aprovechamiento completo de RTX 3050 8GB

## 🔍 **Monitoring VRAM**

```bash
# Monitorear uso de VRAM en tiempo real
watch -n 1 nvidia-smi

# Durante la ejecución de tests
cargo test test_gpu_memory_monitoring -- --test-threads=1 --nocapture
```

---

**Hardware Target**: NVIDIA RTX 3050 8GB  
**CUDA Version**: 12.8  
**cuDNN Version**: 8.9.x  
**Framework**: Candle 0.9.1 con cuda+cudnn features  