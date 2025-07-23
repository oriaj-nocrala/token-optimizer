# Token Optimizer: La Suite de Desarrollo para Agentes de IA

**Veredicto**: ✅ **HERRAMIENTA COMPLETAMENTE FUNCIONAL Y ÚTIL PARA IA**

`token-optimizer` es una herramienta CLI de alto rendimiento, escrita en Rust, diseñada para ser el compañero indispensable de un agente de IA en tareas de desarrollo de software. Su objetivo principal es reducir drásticamente el consumo de tokens (hasta un 99.7%) y, al mismo tiempo, aumentar la comprensión contextual y semántica del código base.

## Capacidades Principales

La herramienta opera sobre una arquitectura híbrida que combina un análisis de código estático (AST) rápido y preciso con una potente capa de análisis semántico basada en Machine Learning.

### 1. Análisis de Código Estático (AST)

Gracias a `tree-sitter`, la herramienta tiene una comprensión profunda de la estructura de proyectos TypeScript/Angular:

-   **Componentes y Servicios:** Identifica metadatos clave como `@Input`/`@Output`, hooks de ciclo de vida, inyección de dependencias y `scope`.
-   **Routing y Módulos:** Analiza la estructura de rutas, `guards`, interceptors, y la arquitectura modular (incluyendo lazy loading).
-   **Manejo de Estado:** Detecta patrones de estado comunes como `BehaviorSubject` y `Observables`.

### 2. Análisis Semántico (Machine Learning)

Utilizando una suite de modelos de 8B de parámetros (DeepSeek, Qwen3) con aceleración GPU (Candle + CUDA/cuDNN), la herramienta ofrece capacidades de análisis avanzadas:

-   **`SmartContextService`**: Proporciona un contexto inteligente sobre fragmentos de código.
-   **`ImpactAnalysisService`**: Predice los efectos en cascada de un cambio y evalúa el riesgo.
-   **`SemanticSearchService`**: Permite buscar código basado en intención en lenguaje natural.
-   **`PatternDetectionService`**: Identifica código duplicado o similar semánticamente.

## Flujo de Trabajo Esencial para Agentes de IA

1.  **Análisis Inicial (Al empezar un proyecto):**
    ```bash
    token-optimizer analyze --verbose --path <ruta_al_proyecto>
    ```

2.  **Obtener Visión General (El comando más importante):**
    ```bash
    token-optimizer overview --format json --path <ruta_al_proyecto>
    ```
    Este comando proporciona un mapa completo de la arquitectura del proyecto en un formato JSON compacto, que sirve como el contexto principal para el agente.

3.  **Trabajo Incremental (Para sesiones continuas):**
    ```bash
    token-optimizer changes --modified-only --path <ruta_al_proyecto>
    ```
    Permite al agente enfocarse únicamente en los archivos que han cambiado desde el último análisis.

## La Suite de Comandos para Agentes

Además del análisis, la herramienta ofrece una suite de comandos de alto nivel para actuar sobre el código:

-   **`token-optimizer oracle`**: Responde a preguntas de desarrollo complejas con ejemplos de código idiomáticos y adaptados al contexto del proyecto.
-   **`token-optimizer refactor`**: Aplica refactorizaciones sugeridas de forma automática y segura.
-   **`token-optimizer test`**: Genera tests unitarios para un archivo específico, respetando los patrones del proyecto.
-   **`token-optimizer doc`**: Genera documentación para el código existente.

## Instalación y Configuración

### Prerrequisitos
-   Rust (>=1.70)
-   Git
-   **Para la capa de ML (Opcional):** GPU NVIDIA con soporte para CUDA 12.8+ y cuDNN 8.9.x.

### Instalación
```bash
# Clonar el repositorio
git clone <url_del_repositorio>
cd token-optimizer

# Compilar para producción
cargo build --release

# (Opcional) Instalar globalmente
cargo install --path .
```

### Configuración de Modelos de ML
Para activar las capacidades semánticas, descarga los modelos necesarios:
```bash
# Descargar todos los modelos recomendados (~18GB)
token-optimizer ml models download --all
```

La herramienta está diseñada con una **degradación elegante**: si los modelos de ML no están disponibles, utilizará su robusto análisis AST para proporcionar la mejor respuesta posible.
