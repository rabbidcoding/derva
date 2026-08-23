# ORIGIN-Ω ZERO — Zero-Training Constitution (T001)

> **INVARIANT:** `trainable_parameter_count == 0`  
> **KPI:** 0 forbidden neural training imports, 0 gradient update loops, 0 unbacked model checkpoints.

---

## 1. Declaración de Principios

ORIGIN-Ω ZERO es una computadora epistemológica producida para entornos **Post-Frontier**. Todo comportamiento emergente, inferencia, planificación y justificación de conocimiento debe ocurrir mediante:

1. **Sistemas Dinámicos Simbólicos y Estructurados**: Álgebra de estados autoritativos en Rust.
2. **Razonamiento Cero-Entrenamiento**: Satisfacción de restricciones, E-graphs, reglas de Horn y grafos causales.
3. **Coprocesamiento Numérico Puro en JAX**: Evaluación vectorial sin parámetros entrenables (`jax.grad` y `optax` están prohibidos).
4. **Verificabilidad Causal y Epistémica**: Transiciones de estado auditables por ORIDs (Origin Resource Identifiers) inmutables.

## 2. Invariantes Restringidos (Bypass Hard-Fail)

Está strictly prohibido incluir en cualquier parte del código autoritativo, coprocesador o pipeline de compilación:
- `jax.grad`, `jax.value_and_grad`, `jax.custom_vjp` (salvo que sea para derivadas simbólicas deterministas sin backpropagation de pesos).
- Módulos de optimizadores de deep learning: `optax`, `torch.optim`, `tf.keras.optimizers`.
- Carga de checkpoints de pesos neurales preentrenados o fine-tuned (`.bin`, `.safetensors`, `.ckpt`, `.h5`, `.pt`, `.onnx`).
- Actualización de parámetros en tiempo de ejecución (`param += lr * grad`).

## 3. Mecanismo de Verificación Guard (CI/CD)

El script `tools/zero_train_guard.py` escanea el 100% de la base de código y dependencias en cada Pull Request y Release. Cualquier violación cancela inmediatamente el pipeline de compilación.
