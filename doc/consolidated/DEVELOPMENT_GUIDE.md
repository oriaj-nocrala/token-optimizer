# Guía de Desarrollo y Arquitectura de `token-optimizer`

Este documento sirve como la guía central para desarrolladores (tanto humanos como agentes de IA) que deseen contribuir, extender o entender la arquitectura interna y la filosofía de `token-optimizer`.

## 1. Filosofía y Paradigmas de Desarrollo

El desarrollo de esta herramienta se guía por principios que maximizan la colaboración efectiva con agentes de IA.

### 1.1. Desarrollo Guiado por Pruebas (TDD)

Los tests son la forma más clara y menos ambigua de definir los requisitos para un agente de IA. El ciclo es:
1.  **Red**: Escribir un test que falla y que define el comportamiento esperado.
2.  **Green**: Implementar el código mínimo necesario para que el test pase.
3.  **Refactor**: Mejorar el código manteniendo los tests en verde.

### 1.2. Desarrollo Guiado por Contratos (Contract-First)

Se definen `traits` de Rust y estructuras de datos claras antes de la implementación. Esto es crucial para sistemas modulares como el `PluginManager`, donde cada plugin debe adherirse a una interfaz estricta.

### 1.3. Desarrollo Guiado por Documentación (DDD)

La documentación en el código, especialmente los comentarios de Rust (`///`), se utiliza para guiar la implementación. La IA es particularmente buena para generar código que cumple con una especificación bien documentada.

## 2. Arquitectura General

La herramienta está diseñada de forma modular para ser extensible y mantenible.

```
src/
├── analyzers/      # Lógica de análisis de código (AST, semántico)
├── cache/          # Sistema de cache inteligente
├── cli/            # Interfaz de línea de comandos (Clap)
├── generators/     # Generación de reportes (JSON, Markdown, texto)
├── ml/             # Infraestructura de Machine Learning
│   ├── plugins/    # Implementaciones de modelos (DeepSeek, Qwen)
│   └── services/   # Servicios de alto nivel (Contexto, Impacto, etc.)
├── types/          # Definiciones de estructuras de datos (structs, enums)
└── utils/          # Utilidades (ficheros, git, hash)
```

## 3. Estrategia de Fiabilidad para la Capa de ML

Para combatir la inconsistencia y los bloqueos de los modelos de lenguaje locales, se implementa una estrategia de "defensa en profundidad".

### Capa 1: Control Externo (Supervisor)
-   **Problema:** El modelo se queda "pensando" indefinidamente.
-   **Solución:** Cada llamada a `llama-cli` se envuelve en un `timeout` configurable (ej: 120-300 segundos). Esto garantiza que la aplicación nunca se bloquee.

### Capa 2: Optimización del Prompt (Guía)
-   **Problema:** Preguntas abiertas pueden confundir al modelo.
-   **Solución:** Usar plantillas de prompts muy específicas y proporcionar ejemplos de la salida deseada (few-shot learning), especialmente para JSON.

### Capa 3: Caching Inteligente (Memoria Persistente)
-   **Problema:** El agente "olvida" análisis previos (memoria de pez).
-   **Solución:** Extender el `SmartCache` para almacenar los resultados de las inferencias de ML. La clave del cache se basa en un hash del prompt. Se incluye un comando `token-optimizer ml pre-cache` para "calentar" el cache de forma proactiva.

### Capa 4: Arquitectura de Modelos por Capas (Equipo de Especialistas)
-   **Problema:** No todas las tareas requieren el mismo nivel de potencia (y latencia).
-   **Solución:** Usar el modelo adecuado para cada tarea:
    1.  **Análisis AST:** Para hechos estructurales (instantáneo).
    2.  **Embeddings/Reranking (Qwen):** Para búsqueda de similitud (rápido).
    3.  **Razonamiento Profundo (DeepSeek):** Para análisis complejos, siempre con un timeout (lento y supervisado).

## 4. Hoja de Ruta de Implementación de ML (Histórico)

El desarrollo de la capa de ML se dividió en fases para asegurar una base sólida.

-   **Fase 1 (Completada):** Se construyó el sistema de plugins de ML. Se crearon los plugins para DeepSeek y Qwen, se implementó la gestión de memoria y se aseguró la compatibilidad con la aceleración de GPU.
-   **Fase 2 (Completada):** Se implementaron los 5 servicios de ML de alto nivel (`SmartContextService`, `ImpactAnalysisService`, etc.), integrando el análisis AST con las capacidades de los plugins de ML y asegurando que existan fallbacks robustos.

## 5. Plan de Soporte para Nuevos Lenguajes (Ej: Rust)

La arquitectura modular facilita la extensión a nuevos lenguajes.

1.  **Extender Tipos:** Añadir nuevos `enum` a `FileType` en `src/types/mod.rs` (ej: `RustModule`, `RustCrate`).
2.  **Detección de Archivos:** Actualizar `detect_file_type()` en `src/utils/file_utils.rs` para reconocer nuevos tipos de archivo (ej: `.rs`, `Cargo.toml`).
3.  **Análisis Específico:** Añadir la lógica de análisis para el nuevo lenguaje en `src/analyzers/`, potencialmente usando un nuevo parser de `tree-sitter`.
4.  **Generadores de Reportes:** Actualizar los generadores en `src/generators/` para incluir la nueva información en los `overview`.
