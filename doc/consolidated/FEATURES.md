# `token-optimizer`: Catálogo de Características

Este documento detalla las funcionalidades clave de la suite `token-optimizer`, diseñadas para potenciar el desarrollo de software asistido por agentes de IA.

## 1. Análisis y Comprensión de Código

### `analyze`
-   **Propósito:** Realiza un análisis exhaustivo de todos los archivos relevantes en un proyecto.
-   **Funcionamiento:** Recorre el árbol de directorios, y para cada archivo, calcula su hash, extrae metadatos AST y lo guarda en el cache (`.cache/analysis-cache.json`).
-   **Uso:** `token-optimizer analyze --path <ruta_del_proyecto>`

### `overview`
-   **Propósito:** Proporciona una vista de alto nivel de la arquitectura del proyecto.
-   **Funcionamiento:** Sintetiza la información del cache para generar un resumen que incluye componentes, servicios, rutas, interceptors, manejo de estado y métricas de salud.
-   **Uso:** `token-optimizer overview --format json` (el formato JSON es ideal para agentes).

### `changes`
-   **Propósito:** Identifica qué archivos han cambiado desde el último análisis.
-   **Funcionamiento:** Se integra con Git para detectar archivos modificados, añadidos o eliminados.
-   **Uso:** `token-optimizer changes --modified-only`

## 2. La Suite de Agentes de IA

Estos comandos representan las capacidades de alto nivel que transforman la herramienta en una suite proactiva.

### `oracle`
-   **Propósito:** Responder a preguntas de desarrollo complejas con soluciones idiomáticas y contextualizadas.
-   **Funcionamiento:**
    1.  Analiza el contexto local del proyecto (dependencias, versiones).
    2.  Realiza una búsqueda web inteligente y enriquecida.
    3.  Filtra y clasifica los resultados por relevancia y calidad.
    4.  Sintetiza una respuesta única y de alta calidad.
-   **Uso:** `token-optimizer oracle --query "cómo implementar X con Y"`

### `refactor`
-   **Propósito:** Aplicar refactorizaciones complejas de forma automática y segura.
-   **Funcionamiento:** Utiliza el `PatternDetectionService` para identificar oportunidades (ej: código duplicado) y genera las operaciones de modificación de archivos necesarias para aplicar el cambio.
-   **Uso:** `token-optimizer refactor --target "archivo.ts" --suggestion "extraer lógica"`

### `test`
-   **Propósito:** Acelerar el ciclo de TDD generando automáticamente tests unitarios.
-   **Funcionamiento:**
    1.  Analiza los métodos públicos y dependencias de un archivo.
    2.  Identifica el framework de testing del proyecto.
    3.  Genera un archivo `*.spec.ts` con mocks para las dependencias y casos de prueba para cada método.
-   **Uso:** `token-optimizer test --file "archivo.ts"`

### `doc`
-   **Propósito:** Generar documentación de código automáticamente.
-   **Funcionamiento:** Utiliza el `ReasoningService` (DeepSeek) para analizar cada método público y generar comentarios en formato JSDoc/TSDoc, explicando su propósito, parámetros y valor de retorno.
-   **Uso:** `token-optimizer doc --file "archivo.ts"`

## 3. Gestión de la Herramienta

### `cache`
-   **Propósito:** Gestionar el ciclo de vida del cache local.
-   **Subcomandos:**
    -   `status`: Muestra estadísticas del cache.
    -   `clean`: Elimina entradas de archivos que ya no existen.
    -   `rebuild`: Borra y reconstruye el cache desde cero.
    -   `clear`: Vacía completamente el cache.
-   **Uso:** `token-optimizer cache <subcomando>`

### `ml`
-   **Propósito:** Gestionar los modelos de Machine Learning.
-   **Subcomandos:**
    -   `models download --all`: Descarga todos los modelos recomendados.
    -   `models list`: Muestra los modelos disponibles.
    -   `models status`: Verifica el estado del cache de modelos.
-   **Uso:** `token-optimizer ml models <subcomando>`
