# Propuesta: La Suite `token-optimizer` para Agentes de IA

Esta propuesta describe la evolución de `token-optimizer` desde una herramienta de análisis a una suite de desarrollo integral y de alto nivel para agentes de IA.

## 1. El Oráculo: `token-optimizer oracle`

Este comando implementa la idea de un "oráculo de código", combinando el análisis de código local con búsquedas web inteligentes para proporcionar soluciones idiomáticas y contextualizadas.

**Comando:**
```bash
token-optimizer oracle --query "cómo implementar wgpu en Android con winit"
```

**Concepto:**
Responde a una pregunta de desarrollo de alto nivel proporcionando un ejemplo de código completo y validado, adaptado específicamente al contexto del proyecto actual (versiones de librerías, patrones de código, etc.).

**Workflow de la Herramienta:**
1.  **Análisis Local (Grounding):** Ejecuta `overview` para extraer dependencias, versiones y patrones del proyecto.
2.  **Búsqueda Inteligente:** Formula una consulta de búsqueda web enriquecida con el contexto local (ej: `"wgpu 0.17.0" "winit 0.28.6" tutorial`).
3.  **Filtrado y Clasificación:** Los resultados se clasifican por relevancia, coincidencia de dependencias, calidad de la fuente (estrellas de GitHub, fecha) y similitud de patrones.
4.  **Síntesis de la Respuesta:** Procesa las mejores fuentes para generar una única respuesta de alta calidad, incluyendo fragmentos de código y explicaciones.

## 2. El Refactorizador: `token-optimizer refactor`

**Comando:**
```bash
token-optimizer refactor --target "src/services/auth.service.ts" --suggestion "extraer lógica de validación a un servicio separado"
```

**Concepto:**
Aplica refactorizaciones sugeridas por el `PatternDetectionService` de forma automática y segura.

**Workflow de la Herramienta:**
1.  Ejecuta `token-optimizer patterns` para obtener una lista de sugerencias de refactorización.
2.  El usuario (o el agente) selecciona una sugerencia.
3.  La herramienta lee los archivos afectados.
4.  Genera y aplica los cambios necesarios, creando nuevos archivos si es necesario.
5.  Presenta los `tool_code` calls para aprobación final.

## 3. El Tester: `token-optimizer test`

**Comando:**
```bash
token-optimizer test --file "src/app/services/auth.service.ts"
```

**Concepto:**
Genera automáticamente tests unitarios para un archivo específico, respetando el framework y los patrones de testing del proyecto.

**Workflow de la Herramienta:**
1.  Analiza el archivo objetivo, sus métodos públicos y sus dependencias.
2.  Identifica el framework de testing (ej: Jasmine, Jest) y los patrones existentes.
3.  Genera mocks para las dependencias inyectadas.
4.  Escribe casos de prueba para cada método público.
5.  Crea el archivo `*.spec.ts` completo.

## 4. El Documentador: `token-optimizer doc`

**Comando:**
```bash
token-optimizer doc --file "src/app/services/auth.service.ts"
```

**Concepto:**
Genera documentación JSDoc/TSDoc para todos los métodos públicos de un archivo, utilizando la capa de razonamiento de ML.

**Workflow de la Herramienta:**
1.  Analiza el archivo y sus métodos públicos.
2.  Para cada método, el `ReasoningService` (DeepSeek) genera un comentario explicando su propósito, parámetros (`@param`) y valor de retorno (`@returns`).
3.  Genera un comentario a nivel de clase/módulo explicando el propósito general.
4.  Añade la documentación al archivo fuente.

---

### Conclusión

La implementación de esta suite transformaría a `token-optimizer` en una plataforma indispensable para el desarrollo de software asistido por IA, cubriendo el ciclo completo: **comprender, actuar, verificar y documentar**.
