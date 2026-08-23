# ORIGIN-Ω ZERO — ROADMAP T001–T100

## Zero-Training Verifiable Epistemic Computer — Post-Frontier Production Roadmap

> **Contrato:** exactamente 100 tasks. Cero entrenamiento, cero parámetros aprendibles, cero backprop, cero checkpoints y cero pesos preentrenados en el producto. Rust posee el estado autoritativo; JAX/XLA se usa únicamente como coprocesador numérico puro; x86-64 Assembly existe solo detrás de una referencia Rust y un gate de performance.

### Convención obligatoria en código

Todo módulo crítico debe incluir comentarios como:

```text
INVARIANT: <propiedad que nunca puede romperse>
KPI: <umbral cuantitativo relevante>
SAFETY: <obligatorio en cada bloque unsafe/FFI/ASM>
```

No se usa el nombre de ninguna persona como supuesta aprobación. Son **lentes de diseño** derivadas de principios y productos históricos.

## Lentes de auditoría

- **Steve Jobs:** Eliminar complejidad que el usuario/desarrollador no necesita; integrar lo esencial; exigir que el producto final se sienta simple.
- **Linus Torvalds:** Interfaces estables, implementación evolutiva, integración rigurosa, cambios pequeños verificables y sin capas innecesarias.
- **Ada Lovelace:** Separar operaciones de los objetos sobre los que operan y permitir composición formal.
- **Alan Turing:** Definir la máquina y su procedimiento independientemente de aprendizaje paramétrico; todo comportamiento debe reducirse a mecanismos ejecutables.
- **Grace Hopper:** No repetir trabajo estable: compilar, reutilizar y automatizar la fábrica de software.
- **Dennis Ritchie:** Abstracciones suficientes para productividad, pero con representación y coste cercanos al hardware.
- **Ken Thompson:** No confiar solo en código fuente o provenance superficial; auditar toolchain, build y autoridad.
- **Donald Knuth:** Algoritmos, invariantes, complejidad, pruebas y benchmarks antes de claims.
- **Bjarne Stroustrup:** Zero-overhead: lo que no usas no debe costar; lo que usas debe competir con una implementación manual razonable.
- **Guido van Rossum:** Estados y errores explícitos; evitar magia, silencios y ambigüedad.
- **Tim Berners-Lee:** Identidad global, interoperabilidad, modularidad y referencias verificables.
- **Bill Gates:** Convertir la arquitectura en plataforma: SDK, tooling, CI, distribución y developer ergonomics.
- **Niklaus Wirth:** Simplicidad estructural radical; si una primitiva no es necesaria, eliminarla.
- **John Carmack:** Profile first; optimizar el hot path demostrado, no el imaginado.
- **Steve Wozniak:** Lograr más con menos hardware/memoria; explotar diseño antes que fuerza bruta.
- **Elon Musk (lente de ingeniería Tesla):** Tratar producción, latencia, throughput, determinismo, recovery y coste como parte de la arquitectura.

## Gates globales no negociables

- **ZT-0:** `trainable_parameter_count == 0`; ningún API de training entra a main/release.
- **EI-0:** promociones ilegales a `VERIFIED` = 0.
- **CI-0:** promociones causales ilegales = 0.
- **AE-0:** efectos externos sin capability = 0.
- **SA-0:** ejecución de artifact stale = 0.
- **RP-100:** replay/rollback exacto = 100% para operaciones deterministas.
- **ASM-0:** mismatches Rust↔ASM = 0; todo Assembly tiene fallback Rust.
- **GH-PROD:** merge a `main` solo vía PR + checks requeridos + CODEOWNERS + ruleset.
- **SUPPLY:** release solo con SBOM + artifact attestation verificable.
- **CLAIM:** ningún claim “post-frontier” se publica si no pasa T100.

## Grafo de fases

```text
P00 Truth/GitHub
  ↓
P01 Formal Semantics
  ↓
P02 Rust Kernel/Store
  ↓
P03 Evidence/Logic
  ↓
P04 Zero-Train Reasoning
  ↓
P05 Causality/Planning
  ↓
P06 JAX Numerical Coprocessor
  ↓
P07 OIR/Certified Compiler
  ↓
P08 SIMD/Assembly/Two-Speed Runtime
  ↓
P09 Production/Security/GitHub Release/Falsification
```


# P00 — Truth, Toolchain & GitHub Constitution


## T001 — Zero-Training Constitution

**Objetivo falsable.** Convertir “cero entrenamiento” en una propiedad verificable del repositorio: sin parámetros aprendibles, optimizers, gradient updates, checkpoints ni pesos preentrenados.

**Artefactos exactos.** `spec/zero_training.md`, `tools/zero_train_guard.py`, `.github/workflows/zero-training.yml`.

**Conexiones.** Upstream: **Ninguna; raíz del roadmap**. Downstream directo: **T002, T003, T010**.

**KPIs PASS/FAIL.**

- 0 imports prohibidos en main/release: jax.grad, jax.value_and_grad, optax, flax.training, torch, tensorflow, checkpoints/weights.
- Guard ejecutado en 100% de PRs y releases.
- 0 bypasses no documentados; cualquier excepción rompe G00.


**Ejemplo mínimo de implementación/configuración.**

```python
# INVARIANT: TRAINABLE_PARAMETER_COUNT == 0
FORBIDDEN = ("jax.grad", "jax.value_and_grad", "optax", "torch", "tensorflow")
assert scan_repo(FORBIDDEN) == []
```

**GitHub/CI.** Required check `zero-training`; bloquea merge y release.


## T002 — Claim Ledger & Kill Criteria

**Objetivo falsable.** Registrar cada claim técnico con métrica, baseline, owner, evidencia y condición explícita de abandono.

**Artefactos exactos.** `spec/claims.yaml`, `spec/kill_criteria.md`, `tools/claims_lint.rs`.

**Conexiones.** Upstream: **T001**. Downstream directo: **T003, T009, T010**.

**KPIs PASS/FAIL.**

- 100% de claims cuantitativos tienen métrica+baseline+gate.
- 0 claims `post-frontier` sin benchmark asociado.
- Toda regresión crítica abre issue automáticamente.


**Ejemplo mínimo de implementación/configuración.**

```rust
// KPI: every quantitative claim must resolve to a benchmark id.
assert!(claim.metric.is_some() && claim.baseline.is_some() && claim.gate.is_some());
```

**GitHub/CI.** CI `claims-lint`; GitHub issue template referencia Claim-ID.


## T003 — Monorepo Canonical Layout

**Objetivo falsable.** Crear un monorepo mínimo donde Rust sea dueño del estado, JAX sea coprocesador numérico y Assembly esté aislado.

**Artefactos exactos.** `Cargo.toml`, `rust-toolchain.toml`, `crates/`, `python/origin_jax/`, `asm/x86_64/`, `spec/`, `bench/`, `tools/`.

**Conexiones.** Upstream: **T001, T002**. Downstream directo: **T004, T005, T010**.

**KPIs PASS/FAIL.**

- `cargo metadata` y import JAX funcionan desde checkout limpio.
- 0 dependencias circulares entre crates.
- `asm/` no puede depender de Python/JAX.


**Ejemplo mínimo de implementación/configuración.**

```text
[workspace]
members = ["crates/*"]
resolver = "2"
# INVARIANT: Rust owns authoritative state; JAX is numerical only.
```

**GitHub/CI.** Repo preparado para GitHub Actions y releases desde el primer commit.


## T004 — Pinned Reproducible Toolchain

**Objetivo falsable.** Fijar Rust, Python/JAX, assembler y herramientas; generar manifest de versiones y hashes reproducibles.

**Artefactos exactos.** `rust-toolchain.toml`, `uv.lock`, `tools/toolchain_manifest.py`, `.github/actions/setup-origin/action.yml`.

**Conexiones.** Upstream: **T003**. Downstream directo: **T005, T006, T009, T010**.

**KPIs PASS/FAIL.**

- Checkout limpio reproduce mismo lock graph 100%.
- Toolchain drift = hard fail.
- CI usa perfil rustup minimal + componentes explícitos.


**Ejemplo mínimo de implementación/configuración.**

```text
# INVARIANT: toolchain versions are exact, never floating.
channel = "1.XX.Y"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

**GitHub/CI.** Reusable local action `setup-origin`; cache key incluye lockfiles + toolchain manifest.


## T005 — GitHub Ruleset & CODEOWNERS Contract

**Objetivo falsable.** Definir política de merge: PR obligatorio, historial lineal, revisiones por ownership, checks requeridos y prohibición de force-push.

**Artefactos exactos.** `.github/CODEOWNERS`, `.github/ruleset.production.json`, `docs/governance/merge-policy.md`.

**Conexiones.** Upstream: **T003, T004**. Downstream directo: **T006, T010, T095**.

**KPIs PASS/FAIL.**

- 0 merges a main sin checks requeridos.
- Cambios en `kernel/`, `asm/`, `security/` requieren code owner.
- Force-push a main/tag release bloqueado.


**Ejemplo mínimo de implementación/configuración.**

```text
/crates/origin-kernel/ @kernel-owners
/asm/                  @perf-owners @security-owners
/.github/              @security-owners
```

**GitHub/CI.** Ruleset importable; required status checks y CODEOWNERS obligatorios.


## T006 — GitHub CI Reusable Workflow

**Objetivo falsable.** Centralizar fmt, clippy, test, doc-test, Python checks, zero-training y build matrix en un workflow reusable determinista.

**Artefactos exactos.** `.github/workflows/_ci.yml`, `.github/workflows/ci.yml`.

**Conexiones.** Upstream: **T004, T005**. Downstream directo: **T007, T008, T010**.

**KPIs PASS/FAIL.**

- PR feedback p50 < 8 min en cache caliente; p95 < 15 min.
- `cargo fmt --check`, `clippy -D warnings`, tests y Python lint = 100% pass.
- Permisos GITHUB_TOKEN mínimos y explícitos.


**Ejemplo mínimo de implementación/configuración.**

```yaml
permissions:
  contents: read
jobs:
  ci:
    uses: ./.github/workflows/_ci.yml
    secrets: inherit
```

**GitHub/CI.** Reusable workflow; ningún job crítico duplicado entre PR/release.


## T007 — CodeQL + Dependency Security

**Objetivo falsable.** Activar análisis estático de Rust, Python/JAX y GitHub Actions, dependency review y actualizaciones Dependabot.

**Artefactos exactos.** `.github/workflows/codeql.yml`, `.github/workflows/dependency-review.yml`, `.github/dependabot.yml`.

**Conexiones.** Upstream: **T006**. Downstream directo: **T008, T010, T095**.

**KPIs PASS/FAIL.**

- 0 alerts Critical/High en merge; Medium requiere issue+owner.
- Dependency Review bloquea nuevas vulnerabilidades High/Critical.
- Dependabot semanal; lockfiles actualizados solo tras CI completo.


**Ejemplo mínimo de implementación/configuración.**

```text
strategy:
  matrix:
    language: [rust, python, actions]
# KPI: merge blocked on required CodeQL/dependency-review checks.
```

**GitHub/CI.** CodeQL, dependency-review y Dependabot son required checks cuando aplique.


## T008 — Artifact Provenance & SLSA Build Path

**Objetivo falsable.** Firmar artefactos de release, SBOM y provenance; separar build workflow reusable del caller.

**Artefactos exactos.** `.github/workflows/_release-build.yml`, `.github/workflows/release.yml`, `tools/verify_attestation.sh`.

**Conexiones.** Upstream: **T006, T007**. Downstream directo: **T010, T094**.

**KPIs PASS/FAIL.**

- 100% binaries publicados tienen attestation verificable.
- SBOM adjunto a 100% releases.
- `gh attestation verify` pasa antes de publicar GitHub Release.


**Ejemplo mínimo de implementación/configuración.**

```yaml
permissions:
  contents: read
  id-token: write
  attestations: write
# INVARIANT: unsigned release artifacts are never published.
```

**GitHub/CI.** Artifact attestations y reusable build workflow; release job verifica antes de publicar.


## T009 — Benchmark Constitution

**Objetivo falsable.** Congelar hardware classes, datasets sintéticos, seeds, métricas, warmup y reglas de comparación antes de optimizar.

**Artefactos exactos.** `bench/CONSTITUTION.md`, `bench/manifest.toml`, `crates/origin-bench/`.

**Conexiones.** Upstream: **T002, T004**. Downstream directo: **T010, T098**.

**KPIs PASS/FAIL.**

- 100% benchmarks tienen baseline, seed policy y unidades.
- CV < 3% en microbench estables; si >3% se marca noisy.
- No benchmark cambia después de ver resultados sin nueva versión.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub struct BenchSpec {
    pub warmups: u32,
    pub samples: u32,
    pub max_cv: f64, // <= 0.03
}
```

**GitHub/CI.** CI smoke benchmarks; nightly full benchmarks; PR muestra delta vs main.


## T010 — G00 Truth + Repository Gate

**Objetivo falsable.** Cerrar P00 solo si el repo es reproducible, seguro, zero-training y gobernado automáticamente.

**Artefactos exactos.** `reports/gates/G00.md`, `tools/gate_g00.rs`.

**Conexiones.** Upstream: **T001, T002, T003, T004, T005, T006, T007, T008, T009**. Downstream directo: **T011**.

**KPIs PASS/FAIL.**

- 10/10 checks P00 verdes.
- 0 bypasses de ruleset en release candidate.
- Build limpio desde runner nuevo; hashes/reportes archivados.


**Ejemplo mínimo de implementación/configuración.**

```text
gate.require("zero-training")?;
gate.require("reproducible-build")?;
gate.require("security")?;
gate.require("bench-constitution")?;
```

**GitHub/CI.** `G00` es required check y bloquea todas las tasks posteriores en release branch.


# P01 — Formal Semantics & Type System


## T011 — Authoritative State Algebra

**Objetivo falsable.** Especificar el único estado autoritativo S=(G,C,E,U,O,B,Z), sus dominios y qué componente puede mutar cada parte.

**Artefactos exactos.** `spec/state_algebra.md`, `crates/origin-core/src/state.rs`.

**Conexiones.** Upstream: **T010**. Downstream directo: **T012, T013, T014, T019, T020**.

**KPIs PASS/FAIL.**

- 0 campos autoritativos duplicados en otros subsistemas.
- Todas las transiciones pasan por `StateTxn`.
- State schema versionado desde v0.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub struct State {
    pub graph: GraphRoot,
    pub constraints: ConstraintRoot,
    pub evidence: EvidenceRoot,
    pub operators: OperatorRoot,
    pub obligations: ObligationRoot,
    pub budget: Budget,
    pub artifacts: ArtifactRoot,
}
```

**GitHub/CI.** Schema/API changes disparan `api-diff` check.


## T012 — Epistemic Status Lattice

**Objetivo falsable.** Definir estados UNKNOWN/HYPOTHESIS/SUPPORTED/VERIFIED/CONTESTED/REFUTED como lattice parcial, no enum promocionable arbitrariamente.

**Artefactos exactos.** `spec/status_lattice.md`, `crates/origin-core/src/status.rs`.

**Conexiones.** Upstream: **T011**. Downstream directo: **T013, T017, T018, T019, T020, T036, T072**.

**KPIs PASS/FAIL.**

- 100% transiciones ilegales rechazadas en property tests ≥1e6 casos.
- `CONTESTED` nunca colapsa silenciosamente a boolean.
- No `unwrap()` en transición de status.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub fn promote(s: Status, proof: &Proof) -> Result<Status, EpistemicError> {
    match (s, proof.kind()) {
        (Status::Supported, ProofKind::Verified) => Ok(Status::Verified),
        _ => Err(EpistemicError::IllegalPromotion),
    }
}
```

**GitHub/CI.** Mutation tests aseguran que quitar guards rompe CI.


## T013 — Canonical Object Model

**Objetivo falsable.** Formalizar representación canónica de Entity, Observation, Claim, Evidence, Operator, Obligation y Artifact.

**Artefactos exactos.** `spec/object_model.md`, `crates/origin-core/src/object.rs`.

**Conexiones.** Upstream: **T011, T012**. Downstream directo: **T014, T016, T020, T022**.

**KPIs PASS/FAIL.**

- Round-trip canónico 100% en ≥1e6 property cases.
- Mismo objeto semántico => mismos bytes.
- Campos no canónicos prohibidos en hash identity.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub trait Canonical {
    fn encode_canonical(&self, out: &mut Vec<u8>);
}
```

**GitHub/CI.** Cross-platform golden vectors en CI Linux x86_64 y macOS ARM cuando disponible.


## T014 — Distinction Semantics

**Objetivo falsable.** Definir una Distinction como predicado/observable que separa hipótesis solo respecto a un dominio y decisión explícitos.

**Artefactos exactos.** `spec/distinction.md`, `crates/origin-core/src/distinction.rs`.

**Conexiones.** Upstream: **T011, T013**. Downstream directo: **T015, T019, T020**.

**KPIs PASS/FAIL.**

- Toda distinción declara Domain+Predicate+Cost.
- 0 distinciones globales implícitas.
- Equivalencia decision-relative demostrada en suite formal.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub struct Distinction {
    pub domain: DomainId,
    pub predicate: PredicateId,
    pub cost: Cost,
}
```

**GitHub/CI.** Spec-test generado desde ejemplos del documento.


## T015 — Lazy Relevance Quotient

**Objetivo falsable.** Formalizar cuándo dos estados son equivalentes para una decisión sin enumerar Ω y definir extracción local del cociente activo.

**Artefactos exactos.** `spec/lazy_quotient.md`, `crates/origin-core/src/quotient.rs`.

**Conexiones.** Upstream: **T014**. Downstream directo: **T020, T049, T059**.

**KPIs PASS/FAIL.**

- Nunca se materializa Ω.
- En test worlds, quotient conserva 100% de decisiones observables.
- Active quotient reduce ≥10× estados en ≥70% de escenarios benchmark diseñados para redundancia.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub fn equivalent(a: &WorldSig, b: &WorldSig, r: &RelevantSet) -> bool {
    r.iter().all(|q| q.eval(a) == q.eval(b))
}
```

**GitHub/CI.** `quotient-correctness` required test; benchmark delta publicado.


## T016 — Evidence vs Derived Information Semantics

**Objetivo falsable.** Separar observación primaria, información derivada y confianza; prohibir doble conteo por lineage.

**Artefactos exactos.** `spec/evidence_semantics.md`, `crates/origin-core/src/evidence.rs`.

**Conexiones.** Upstream: **T013**. Downstream directo: **T017, T018, T020, T031**.

**KPIs PASS/FAIL.**

- 100% derivaciones conservan roots de provenance.
- 100 fuentes copiadas de 1 raíz cuentan como 1 dominio dependiente en benchmark.
- 0 `Verified` sin path permitido.


**Ejemplo mínimo de implementación/configuración.**

```text
enum Support {
    Primary(EvidenceId),
    Derived { rule: RuleId, parents: Vec<ObjectId> },
}
```

**GitHub/CI.** Adversarial evidence fixtures ejecutados en PR.


## T017 — Verification Obligation Algebra

**Objetivo falsable.** Reemplazar deuda escalar por obligaciones tipadas con precondiciones, expiración y método de descarga.

**Artefactos exactos.** `spec/obligations.md`, `crates/origin-core/src/obligation.rs`.

**Conexiones.** Upstream: **T012, T016**. Downstream directo: **T019, T020, T035**.

**KPIs PASS/FAIL.**

- Toda promoción crítica resuelve set de obligaciones explícito.
- Obligación expirada invalida `freshness` 100%.
- 0 obligación puede auto-satisfacerse por el claim que protege.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub enum ObligationKind {
    SourceRequired, IndependentSource, Execution, Proof, Intervention, Freshness, HumanApproval
}
```

**GitHub/CI.** `obligation-cycle` detector obligatorio en CI.


## T018 — Causal Status Type Algebra

**Objetivo falsable.** Formalizar OBSERVATIONAL/ASSUMED_CAUSAL/INTERVENTIONAL/MECHANISTIC/VERIFIED_CAUSAL y promociones permitidas.

**Artefactos exactos.** `spec/causal_types.md`, `crates/origin-core/src/causal_status.rs`.

**Conexiones.** Upstream: **T012, T016**. Downstream directo: **T019, T020, T051, T052, T072**.

**KPIs PASS/FAIL.**

- 0 observational→verified causal sin witness.
- 100% promociones incluyen provenance+assumptions.
- False causal promotion = 0 en suite sintética known-truth.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub fn causal_promote(from: CausalStatus, w: &CausalWitness)
    -> Result<CausalStatus, CausalError> { /* explicit matrix */ }
```

**GitHub/CI.** Causal transition matrix snapshot testeado.


## T019 — Nine-Instruction Micro-ISA

**Objetivo falsable.** Cerrar semántica de OBSERVE, PROPOSE, RELATE, REFINE, QUERY, INTERVENE, VERIFY, COMMIT y COMPILE sin añadir primitivas.

**Artefactos exactos.** `spec/micro_isa.md`, `crates/origin-core/src/opcode.rs`.

**Conexiones.** Upstream: **T011, T012, T014, T017, T018**. Downstream directo: **T020, T029, T037, T071**.

**KPIs PASS/FAIL.**

- Exactamente 9 opcodes v1.
- 100% operaciones de referencia expresables por composición.
- Añadir opcode requiere ADR + benchmark + aprobación de arquitectura.


**Ejemplo mínimo de implementación/configuración.**

```rust
#[repr(u8)]
pub enum OpCode { Observe, Propose, Relate, Refine, Query, Intervene, Verify, Commit, Compile }
```

**GitHub/CI.** CI cuenta opcodes y falla si !=9 en v1.


## T020 — G01 Formal Consistency Gate

**Objetivo falsable.** Demostrar consistencia operativa mínima mediante modelos pequeños exhaustivos y tests de propiedades antes de storage/runtime.

**Artefactos exactos.** `reports/gates/G01.md`, `crates/origin-modelcheck/`.

**Conexiones.** Upstream: **T011, T012, T013, T014, T015, T016, T017, T018, T019**. Downstream directo: **T021**.

**KPIs PASS/FAIL.**

- Exploración exhaustiva de todos estados ≤12 objetos sin violar invariantes.
- ≥1e7 transiciones property-tested nightly.
- 0 counterexample unresolved para invariantes A0–A12.


**Ejemplo mínimo de implementación/configuración.**

```text
for s in enumerate_states(MAX_SMALL_STATE) {
    for op in legal_ops(&s) {
        assert_invariants(step(s.clone(), op)?);
    }
}
```

**GitHub/CI.** `G01-formal` required check para merge de crates posteriores.


# P02 — Rust Authoritative Kernel & Immutable Store


## T021 — Core Rust Crate Boundaries

**Objetivo falsable.** Crear crates `origin-core`, `origin-kernel`, `origin-store`, `origin-verify` con API unidireccional y sin dependencias cíclicas.

**Artefactos exactos.** `crates/origin-core/`, `crates/origin-kernel/`, `crates/origin-store/`, `crates/origin-verify/`.

**Conexiones.** Upstream: **T020**. Downstream directo: **T022, T028, T030**.

**KPIs PASS/FAIL.**

- 0 dependency cycles.
- `origin-core` sin I/O ni async runtime.
- Unsafe Rust = 0 en core/kernel/store/verify.


**Ejemplo mínimo de implementación/configuración.**

```rust
// INVARIANT: origin-core is pure data + semantics; no filesystem/network imports.
pub use origin_core::{Claim, Evidence, Operator};
```

**GitHub/CI.** `cargo deny`/dependency graph check en CI.


## T022 — Canonical Binary Codec

**Objetivo falsable.** Implementar codec binario propio mínimo para objetos canónicos con orden fijo, longitudes acotadas y rejection de non-canonical encodings.

**Artefactos exactos.** `crates/origin-core/src/codec.rs`, `spec/encoding.md`, `tests/golden/codec/`.

**Conexiones.** Upstream: **T013, T021**. Downstream directo: **T023, T030, T092**.

**KPIs PASS/FAIL.**

- Byte-for-byte determinism 100% cross-platform.
- Decoder rechaza 100% fixtures malleables.
- Throughput ≥500 MB/s en payloads grandes o ≥2M small objects/s en benchmark release, lo que ocurra primero según clase.


**Ejemplo mínimo de implementación/configuración.**

```text
fn put_u64(out: &mut Vec<u8>, x: u64) { encode_varint_canonical(out, x); }
// INVARIANT: decoder rejects overlong encodings.
```

**GitHub/CI.** Golden vectors versionados; fuzz decoder job.


## T023 — ORID Content Addressing

**Objetivo falsable.** Implementar ORID como hash versionado sobre canonical bytes y domain separation por tipo.

**Artefactos exactos.** `crates/origin-core/src/orid.rs`, `spec/orid.md`.

**Conexiones.** Upstream: **T022**. Downstream directo: **T024, T030, T086**.

**KPIs PASS/FAIL.**

- 0 colisiones en 1e8 objetos nightly sintéticos; cryptographic collision claim no se hace.
- Type-domain separation 100%.
- ORID parse/format round-trip 100%.


**Ejemplo mínimo de implementación/configuración.**

```rust
let digest = hash(b"origin:claim:v1\0", canonical_bytes);
ORID::new(ObjectKind::Claim, digest)
```

**GitHub/CI.** ORID golden vectors públicos en release.


## T024 — Append-Only Object Store

**Objetivo falsable.** Persistir objetos por ORID, verificar hash al read y no permitir mutación in-place.

**Artefactos exactos.** `crates/origin-store/src/object_store.rs`.

**Conexiones.** Upstream: **T023**. Downstream directo: **T025, T027, T030**.

**KPIs PASS/FAIL.**

- Bitflip detectado 100% en corruption tests.
- Write-after-read identity 100%.
- Crash durante append nunca publica objeto parcial como válido.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub fn get(&self, id: ORID) -> Result<Vec<u8>> {
    let bytes = self.read_raw(id)?;
    ensure!(ORID::of(&bytes) == id, Corruption);
    Ok(bytes)
}
```

**GitHub/CI.** Crash/corruption tests en nightly y release.


## T025 — Merkle Commit DAG

**Objetivo falsable.** Crear commits inmutables que referencien parent roots, delta canonical y policy root.

**Artefactos exactos.** `crates/origin-store/src/commit.rs`, `spec/commit_dag.md`.

**Conexiones.** Upstream: **T024**. Downstream directo: **T026, T027, T030, T053, T054, T079**.

**KPIs PASS/FAIL.**

- Replay commit root exacto 100%.
- Branch/merge no reescribe historia.
- Commit hash cambia ante cualquier delta/policy change.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub struct Commit {
    parents: SmallVec<[ORID; 2]>,
    delta: ORID,
    policy_root: ORID,
}
```

**GitHub/CI.** Commit root se adjunta a benchmark/release metadata.


## T026 — Atomic Transaction Engine

**Objetivo falsable.** Implementar prepare→validate→fsync→publish-root con recuperación idempotente.

**Artefactos exactos.** `crates/origin-kernel/src/txn.rs`, `crates/origin-store/src/wal.rs`.

**Conexiones.** Upstream: **T025**. Downstream directo: **T028, T029, T030, T091**.

**KPIs PASS/FAIL.**

- Kill injection en cada syscall boundary: 0 roots corruptos en ≥1e5 runs.
- Commit visibility atómica 100%.
- Recovery idempotente 100%.


**Ejemplo mínimo de implementación/configuración.**

```text
wal.append(&prepared)?;
wal.fsync()?;
validate(&prepared)?;
root.publish_atomic(prepared.root)?;
```

**GitHub/CI.** Fault-injection workflow nightly; release bloqueado si falla.


## T027 — Graph Relation Indexes

**Objetivo falsable.** Crear índices compactos por relation kind, source, target y provenance ancestor sin duplicar autoridad.

**Artefactos exactos.** `crates/origin-store/src/index/`.

**Conexiones.** Upstream: **T024, T025**. Downstream directo: **T030, T032, T038, T049, T085, T097**.

**KPIs PASS/FAIL.**

- Index rebuild desde object store produce resultado idéntico 100%.
- Index corruption nunca altera truth state; solo fuerza rebuild.
- Lookup p99 <200µs para 1M relaciones en benchmark host release.


**Ejemplo mínimo de implementación/configuración.**

```rust
// Indexes are derived caches, never authoritative.
pub fn rebuild(root: CommitRoot) -> Result<IndexSet> { /* deterministic scan */ }
```

**GitHub/CI.** Benchmark regression >10% bloquea merge en paths marcados performance-critical.


## T028 — Budget & Resource Accounting

**Objetivo falsable.** Hacer CPU-time, wall-time, allocations, queries e interventions presupuestables por operación.

**Artefactos exactos.** `crates/origin-kernel/src/budget.rs`.

**Conexiones.** Upstream: **T021, T026**. Downstream directo: **T030, T045, T047, T058, T087**.

**KPIs PASS/FAIL.**

- 100% loops de búsqueda consultan budget.
- Budget exhaustion devuelve `UNKNOWN/TIMEOUT`, nunca guess.
- Accounting overhead <3% p50 en benchmark micro.


**Ejemplo mínimo de implementación/configuración.**

```text
budget.charge(StepCost::ConstraintCheck)?;
if budget.exhausted() { return Err(Stop::BudgetExceeded); }
```

**GitHub/CI.** `budget-lint` busca loops de search sin charge/check.


## T029 — Capability & Effect Gate

**Objetivo falsable.** Aplicar capability tokens tipados a QUERY_EXTERNAL, INTERVENE y COMMIT; PURE/READ no pueden escalar efectos.

**Artefactos exactos.** `crates/origin-kernel/src/capability.rs`, `crates/origin-kernel/src/effects.rs`.

**Conexiones.** Upstream: **T019, T026**. Downstream directo: **T030, T053, T073, T092, T093**.

**KPIs PASS/FAIL.**

- Unauthorized side effects = 0 en ≥1e6 adversarial sequences.
- Capability non-forgeability dentro del process model testeado.
- Every effecting opcode logs principal+capability+commit.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub fn intervene(cap: Capability<Intervene>, op: OperatorId) -> Result<Receipt> {
    policy::authorize(&cap, op)?;
    execute(op)
}
```

**GitHub/CI.** Security tests required; changes en capability code requieren CODEOWNER security.


## T030 — G02 Authoritative Kernel Gate

**Objetivo falsable.** Validar que el kernel Rust puede persistir, ramificar, recuperar, presupuestar y autorizar sin JAX ni Assembly.

**Artefactos exactos.** `reports/gates/G02.md`.

**Conexiones.** Upstream: **T021, T022, T023, T024, T025, T026, T027, T028, T029**. Downstream directo: **T031, T037, T039, T044, T081**.

**KPIs PASS/FAIL.**

- 100% core invariants verdes.
- 0 unsafe en authoritative crates.
- Line coverage core/kernel ≥95%, branch ≥90%; mutation score ≥85%.
- 1M-object store/replay sin divergencia.


**Ejemplo mínimo de implementación/configuración.**

```text
assert_eq!(replay(root)?.root(), root);
assert_eq!(unsafe_count("authoritative-crates"), 0);
```

**GitHub/CI.** `G02` required; coverage/mutation reports adjuntos a PR.


# P03 — Evidence, Logic & Constraint Proof Engine


## T031 — Evidence Object Runtime

**Objetivo falsable.** Implementar Evidence con source identity, acquisition method, timestamp, correlation domain, trust domain y raw object ORID.

**Artefactos exactos.** `crates/origin-evidence/src/evidence.rs`.

**Conexiones.** Upstream: **T016, T030**. Downstream directo: **T032, T034, T035, T040**.

**KPIs PASS/FAIL.**

- Campos obligatorios presentes 100%.
- Evidence sin raw ORID no puede soportar VERIFIED.
- Timestamp/method malformed = reject.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub struct Evidence {
    raw: ORID, source: SourceId, method: MethodId,
    correlation: CorrelationDomain, trust: TrustDomain, observed_at: Timestamp,
}
```

**GitHub/CI.** Schema evolution check y fixtures versionados.


## T032 — Provenance Hypergraph

**Objetivo falsable.** Representar derivaciones N-arias y reconstruir roots, reglas y transformaciones exactas de cualquier claim.

**Artefactos exactos.** `crates/origin-evidence/src/provenance.rs`.

**Conexiones.** Upstream: **T031, T027**. Downstream directo: **T033, T036, T040, T093, T096**.

**KPIs PASS/FAIL.**

- `why(claim)` reconstruye 100% roots/rules.
- Cycle insertion rechazado 100%.
- Traversal de 100k-edge lineage p99 <50ms release.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub struct Derivation {
    rule: RuleId,
    parents: SmallVec<[ORID; 4]>,
    child: ORID,
}
```

**GitHub/CI.** `provenance-cycle` and perf checks.


## T033 — Correlation-Domain Deduplicator

**Objetivo falsable.** Evitar que evidencia derivada/copiada se trate como independiente mediante ancestry y dominios declarados.

**Artefactos exactos.** `crates/origin-evidence/src/correlation.rs`, `bench/scenarios/copied_sources.yaml`.

**Conexiones.** Upstream: **T032**. Downstream directo: **T034, T040**.

**KPIs PASS/FAIL.**

- 100 copias/1 root => independent_root_count=1.
- False merge <1% en synthetic domains con ground truth.
- Nunca aumenta status solo por multiplicidad dentro del mismo root domain.


**Ejemplo mínimo de implementación/configuración.**

```rust
let roots = provenance.independent_roots(evidence_set)?;
score.independent_support = roots.len();
```

**GitHub/CI.** Adversarial misinformation benchmark en PR.


## T034 — Trust Policy Engine

**Objetivo falsable.** Separar reliability histórica de source, integrity de transporte y correctness del claim; trust modifica prioridad, no prueba lógica.

**Artefactos exactos.** `crates/origin-evidence/src/trust.rs`, `spec/trust_policy.md`.

**Conexiones.** Upstream: **T031, T033**. Downstream directo: **T040**.

**KPIs PASS/FAIL.**

- `trust=1.0` nunca crea VERIFIED sin derivación.
- Policy changes versionadas por ORID.
- Same evidence under different trust policy no cambia raw provenance.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub fn priority(e: &Evidence, p: &TrustPolicy) -> Priority {
    p.rank(e.source, e.method) // never mutates epistemic status
}
```

**GitHub/CI.** Policy root incluido en commit/release metadata.


## T035 — Verification Obligation Runtime

**Objetivo falsable.** Instanciar, resolver, expirar y reabrir obligations; mantener witness específico.

**Artefactos exactos.** `crates/origin-verify/src/obligations.rs`.

**Conexiones.** Upstream: **T017, T031**. Downstream directo: **T036, T040, T052, T074, T078, T096**.

**KPIs PASS/FAIL.**

- Resolution witness obligatorio 100%.
- Expired freshness reabre obligation en ≤1 transaction.
- Self-witness/cycle rejected 100%.


**Ejemplo mínimo de implementación/configuración.**

```text
obligation.resolve(Resolution {
    witness: evidence_id,
    verifier: verifier_id,
    at: clock.now(),
})?;
```

**GitHub/CI.** Time/freshness property tests con reloj simulado.


## T036 — Contradiction Preservation Engine

**Objetivo falsable.** Detectar incompatibilidad y marcar CONTESTED sin borrar ramas ni seleccionar ganador automáticamente.

**Artefactos exactos.** `crates/origin-verify/src/contradiction.rs`.

**Conexiones.** Upstream: **T012, T032, T035**. Downstream directo: **T040, T097**.

**KPIs PASS/FAIL.**

- 100% contradictory fixtures preservan ambos claims.
- 0 silent overwrite.
- Conflict query devuelve minimal conflicting set cuando solver lo permite.


**Ejemplo mínimo de implementación/configuración.**

```text
if solver.incompatible(a, b)? {
    graph.relate(a, Relation::Contradicts, b)?;
    status.set_pair(a, b, Status::Contested)?;
}
```

**GitHub/CI.** Conflict tests required.


## T037 — Horn-Rule IR

**Objetivo falsable.** Definir subset seguro de reglas Horn con variables tipadas, negación estratificada opcional y sin funciones recursivas que destruyan decidibilidad del v1.

**Artefactos exactos.** `crates/origin-logic/src/horn.rs`, `spec/horn_subset.md`.

**Conexiones.** Upstream: **T019, T030**. Downstream directo: **T038, T039, T040, T047**.

**KPIs PASS/FAIL.**

- Parser/validator rechaza reglas fuera del subset 100%.
- Rule evaluation determinista.
- Rule IDs content-addressed.


**Ejemplo mínimo de implementación/configuración.**

```text
grandparent(X,Z) :- parent(X,Y), parent(Y,Z).
// INVARIANT: every variable in head appears in body.
```

**GitHub/CI.** Grammar con snapshot tests.


## T038 — Semi-Naive Fixed-Point Engine

**Objetivo falsable.** Evaluar reglas incrementalmente usando deltas para evitar recomputación completa.

**Artefactos exactos.** `crates/origin-logic/src/fixpoint.rs`.

**Conexiones.** Upstream: **T037, T027**. Downstream directo: **T040, T046**.

**KPIs PASS/FAIL.**

- Mismo fixed point que naive en 100% differential tests.
- ≥5× speedup vs naive en benchmark recursivo 1M facts.
- No allocations no acotadas por iteration.


**Ejemplo mínimo de implementación/configuración.**

```rust
while !delta.is_empty() {
    let next = eval_rules_against_delta(&rules, &facts, &delta)?;
    delta = facts.insert_new(next);
}
```

**GitHub/CI.** Differential + perf benchmark required.


## T039 — Constraint Solver ABI

**Objetivo falsable.** Definir interfaz SAT/finite-domain/linear arithmetic con proof/witness hooks; backend reference propio y oracle Z3 solo para test diferencial.

**Artefactos exactos.** `crates/origin-constraints/src/lib.rs`, `crates/origin-constraints/src/reference.rs`.

**Conexiones.** Upstream: **T030, T037**. Downstream directo: **T040, T043, T047, T055, T064**.

**KPIs PASS/FAIL.**

- Reference solver exacto en domains pequeños exhaustivos.
- Differential agreement con oracle ≥99.999% en casos soportados; cualquier mismatch triageado.
- Timeout nunca se convierte en UNSAT.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub enum SolveResult<W> { Sat(W), Unsat(Proof), Unknown(BudgetStop) }
```

**GitHub/CI.** Oracle no se enlaza en release binary; CI differential feature-only.


## T040 — G03 Epistemic Proof Gate

**Objetivo falsable.** Integrar evidence→provenance→logic→constraints→obligations y demostrar promociones justificadas.

**Artefactos exactos.** `reports/gates/G03.md`, `bench/scenarios/epistemic_adversarial/`.

**Conexiones.** Upstream: **T031, T032, T033, T034, T035, T036, T037, T038, T039**. Downstream directo: **T041, T046, T088**.

**KPIs PASS/FAIL.**

- Illegal VERIFIED promotions = 0 en ≥1e6 adversarial cases.
- Circular provenance acceptance = 0.
- Copied-source overcount = 0 en ground-truth suite.
- Proof replay = 100%.


**Ejemplo mínimo de implementación/configuración.**

```rust
for case in adversarial_suite() {
    let out = engine.run(case)?;
    assert!(out.no_illegal_verified());
}
```

**GitHub/CI.** `G03` required check y report artifact.


# P04 — Zero-Train Hypothesis, Equality & Search Intelligence


## T041 — Typed Hypothesis Grammar

**Objetivo falsable.** Definir gramática finita extensible por dominio con tipos, cost model y canonical form; no generar términos imposibles.

**Artefactos exactos.** `crates/origin-search/src/grammar.rs`, `spec/hypothesis_grammar.md`.

**Conexiones.** Upstream: **T040**. Downstream directo: **T042, T044, T050**.

**KPIs PASS/FAIL.**

- 100% generated ASTs type-correct.
- Canonical grammar hash versionado.
- Branching factor reportado por nonterminal; sin expansión ilimitada.


**Ejemplo mínimo de implementación/configuración.**

```text
Expr := Var(T) | Const(T) | Apply(Op<T...->T>, [Expr...])
// KPI: generate() never returns ill-typed AST.
```

**GitHub/CI.** Grammar compatibility snapshots.


## T042 — Cost-Ordered Enumerator

**Objetivo falsable.** Enumerar hipótesis por complejidad/coste creciente con deduplicación canónica y resumable frontier.

**Artefactos exactos.** `crates/origin-search/src/enumerate.rs`.

**Conexiones.** Upstream: **T041**. Downstream directo: **T043, T048, T050, T063**.

**KPIs PASS/FAIL.**

- No candidate con coste k+1 antes de agotar k salvo bound explícito.
- Resume produce misma secuencia 100%.
- ≥1M candidates/s en grammar microbench simple.


**Ejemplo mínimo de implementación/configuración.**

```rust
while let Some(node) = frontier.pop_min() {
    yield_if_novel(node)?;
    expand(node, &mut frontier)?;
}
```

**GitHub/CI.** Sequence golden tests y perf regression.


## T043 — Constraint-Guided Pruning

**Objetivo falsable.** Consultar constraint engine antes de materializar ramas costosas; cachear UNSAT cores por grammar context.

**Artefactos exactos.** `crates/origin-search/src/prune.rs`.

**Conexiones.** Upstream: **T039, T042**. Downstream directo: **T048, T050**.

**KPIs PASS/FAIL.**

- Soundness: 0 soluciones válidas podadas en small exhaustive worlds.
- ≥20× reducción de candidates en benchmark constrained.
- Pruning overhead <15% cuando no ayuda; adaptive disable.


**Ejemplo mínimo de implementación/configuración.**

```text
match constraints.check(prefix)? {
    SolveResult::Unsat(_) => continue,
    _ => frontier.push(prefix),
}
```

**GitHub/CI.** Ablation benchmark pruning on/off.


## T044 — E-Graph Core

**Objetivo falsable.** Implementar e-graph mínimo en Rust con hash-consing, union-find, rebuild y typed enodes.

**Artefactos exactos.** `crates/origin-egraph/src/lib.rs`.

**Conexiones.** Upstream: **T041, T030**. Downstream directo: **T045, T050**.

**KPIs PASS/FAIL.**

- Congruence closure correcto en exhaustive algebra tests.
- 0 type-violating unions.
- 1M enodes dentro de memory budget <1.5GB release benchmark.


**Ejemplo mínimo de implementación/configuración.**

```rust
let a = eg.add(expr_a)?;
let b = eg.add(expr_b)?;
eg.union_typed(a, b)?;
eg.rebuild()?;
```

**GitHub/CI.** Memory/perf nightly.


## T045 — Equality Saturation & Rewrite Safety

**Objetivo falsable.** Aplicar reglas de equivalencia con conditions y budgets; rechazar rewrites no demostrados.

**Artefactos exactos.** `crates/origin-egraph/src/rewrite.rs`, `spec/rewrite_rules.md`.

**Conexiones.** Upstream: **T044, T028**. Downstream directo: **T050, T075**.

**KPIs PASS/FAIL.**

- 100% rewrites tienen proof/axiom ID.
- Saturation siempre respeta budget.
- Extractor devuelve expresión equivalente en differential oracle tests.


**Ejemplo mínimo de implementación/configuración.**

```text
rewrite!("add-zero"; "(+ ?a 0)" => "?a"; proof = AXIOM_ADD_ZERO);
```

**GitHub/CI.** Rule changes requieren proof-id and CODEOWNER logic.


## T046 — Forward-Chaining Reasoner

**Objetivo falsable.** Construir consecuencias nuevas desde facts/deltas y registrar proof trace por cada claim derivado.

**Artefactos exactos.** `crates/origin-reason/src/forward.rs`.

**Conexiones.** Upstream: **T038, T040**. Downstream directo: **T050**.

**KPIs PASS/FAIL.**

- Completeness respecto al Horn subset soportado en small models =100%.
- Cada derivación tiene trace replayable.
- Incremental update no recalcula >20% facts en benchmark local-change.


**Ejemplo mínimo de implementación/configuración.**

```text
for consequence in fixpoint.delta(facts, rules)? {
    commit_derived(consequence.claim, consequence.proof)?;
}
```

**GitHub/CI.** Proof replay check.


## T047 — Backward-Chaining Goal Resolver

**Objetivo falsable.** Descomponer un goal en subgoals con memoization, cycle detection y budgets explícitos.

**Artefactos exactos.** `crates/origin-reason/src/backward.rs`.

**Conexiones.** Upstream: **T037, T039, T028**. Downstream directo: **T048, T049, T050**.

**KPIs PASS/FAIL.**

- Cycle detection =100%.
- Solved/unsolved exacto en finite reference suite.
- Subgoal cache hit ≥60% en recursive benchmark o se elimina cache.


**Ejemplo mínimo de implementación/configuración.**

```text
fn prove(goal: Goal, ctx: &mut Ctx) -> SolveResult<Proof> {
    ctx.budget.charge(StepCost::GoalExpand)?;
    ctx.memo_or_expand(goal)
}
```

**GitHub/CI.** Cache ablation required.


## T048 — Abductive Minimal-Explanation Search

**Objetivo falsable.** Encontrar conjuntos mínimos de hipótesis que vuelven observaciones consistentes, sin promoverlas a verdad.

**Artefactos exactos.** `crates/origin-reason/src/abduce.rs`.

**Conexiones.** Upstream: **T042, T043, T047**. Downstream directo: **T050**.

**KPIs PASS/FAIL.**

- Returned explanation satisface observations 100%.
- Minimality exacta en small exhaustive set; bounded approximation labeled en large.
- Output status siempre HYPOTHESIS hasta verify.


**Ejemplo mínimo de implementación/configuración.**

```rust
let h = search_min_explanation(obs, budget)?;
assert_eq!(h.status, Status::Hypothesis);
```

**GitHub/CI.** No `abduce` API puede emitir Verified type.


## T049 — Active Slice Retriever

**Objetivo falsable.** Extraer solo claims/evidence/operators/obligations que pueden afectar un goal mediante backward dependency + relevance quotient.

**Artefactos exactos.** `crates/origin-reason/src/active_slice.rs`.

**Conexiones.** Upstream: **T015, T027, T047**. Downstream directo: **T050, T057, T059, T078, T083, T097**.

**KPIs PASS/FAIL.**

- Decision equivalence full-graph vs active-slice =100% en test suite.
- Median slice ≤10% del graph en long-horizon benchmark.
- p99 extraction <20ms para 1M-edge graph.


**Ejemplo mínimo de implementación/configuración.**

```rust
let slice = backward_reachable(goal, relevant_relations)?;
debug_assert_eq!(decide(&slice)?, decide(&full_graph)?);
```

**GitHub/CI.** Full-vs-slice differential test required.


## T050 — G04 Zero-Train Reasoning Gate

**Objetivo falsable.** Demostrar generación→pruning→equivalence→forward/backward→abduction→active slice sin pesos ni gradientes.

**Artefactos exactos.** `reports/gates/G04.md`, `bench/zero_train_reasoning/`.

**Conexiones.** Upstream: **T041, T042, T043, T044, T045, T046, T047, T048, T049**. Downstream directo: **T051, T061, T071, T088, T098**.

**KPIs PASS/FAIL.**

- Zero-training guard green.
- ≥95% solve rate en benchmark formal interno pre-registered.
- 0 invalid proofs.
- Median active slice ≤10%; candidate pruning ≥20× en constrained suite.


**Ejemplo mínimo de implementación/configuración.**

```rust
assert_eq!(trainable_parameter_count(), 0);
assert!(report.invalid_proofs == 0);
assert!(report.solve_rate >= 0.95);
```

**GitHub/CI.** `G04` required; benchmark manifest frozen from T009.


# P05 — Causal Operators, Counterfactuals & Planning


## T051 — Operator Schema & Preconditions

**Objetivo falsable.** Implementar Operator con domain, codomain, preconditions, effect relation, evidence, causal status, cost y risk.

**Artefactos exactos.** `crates/origin-causal/src/operator.rs`.

**Conexiones.** Upstream: **T018, T050**. Downstream directo: **T052, T053, T054, T055, T057, T060**.

**KPIs PASS/FAIL.**

- 100% operators type-check domains/codomains.
- Missing precondition => operator not executable.
- Risk/cost mandatory for INTERVENE.


**Ejemplo mínimo de implementación/configuración.**

```rust
pub struct Operator {
    domain: SchemaId, codomain: SchemaId, pre: PredicateId,
    effect: EffectId, status: CausalStatus, cost: Cost, risk: Risk,
}
```

**GitHub/CI.** Schema change API diff.


## T052 — Causal Promotion Validator

**Objetivo falsable.** Implementar matriz de promoción con witnesses, assumptions y provenance obligatorios.

**Artefactos exactos.** `crates/origin-causal/src/promotion.rs`.

**Conexiones.** Upstream: **T018, T035, T051**. Downstream directo: **T060**.

**KPIs PASS/FAIL.**

- False promotion =0 en known-truth causal suite.
- Every verified-causal operator has intervention/mechanism witness.
- Assumption removal invalidates dependent status 100%.


**Ejemplo mínimo de implementación/configuración.**

```text
ensure!(witness.supports_transition(from, to), CausalError::MissingWitness);
```

**GitHub/CI.** Causal safety required check.


## T053 — Intervention Journal

**Objetivo falsable.** Registrar before-state root, action ORID, capability, environment receipt, after-observation root y timestamps.

**Artefactos exactos.** `crates/origin-causal/src/journal.rs`.

**Conexiones.** Upstream: **T025, T029, T051**. Downstream directo: **T060, T096**.

**KPIs PASS/FAIL.**

- 100% real interventions generan receipt+journal entry.
- Missing after-observation marca outcome UNKNOWN, no success.
- Journal append atomic con commit.


**Ejemplo mínimo de implementación/configuración.**

```text
journal.append(InterventionRecord { before, action, cap, receipt, after })?;
```

**GitHub/CI.** Security owner review para cambios en journal.


## T054 — Counterfactual Fork Engine

**Objetivo falsable.** Crear branch hipotética copy-on-write sobre commit DAG, sin capacidades de efecto real por defecto.

**Artefactos exactos.** `crates/origin-causal/src/counterfactual.rs`.

**Conexiones.** Upstream: **T025, T051**. Downstream directo: **T056, T060, T065**.

**KPIs PASS/FAIL.**

- Counterfactual branch no modifica real root 100%.
- External effect capabilities absent by construction.
- Fork creation p99 <1ms for metadata-only fork.


**Ejemplo mínimo de implementación/configuración.**

```rust
let cf = state.fork_counterfactual()?;
assert!(!cf.capabilities().contains(Effect::Intervene));
```

**GitHub/CI.** Capability isolation test required.


## T055 — Operator Composition Checker

**Objetivo falsable.** Componer operadores solo si postconditions de A satisfacen preconditions/domain de B y efectos no violan policy.

**Artefactos exactos.** `crates/origin-causal/src/compose.rs`.

**Conexiones.** Upstream: **T039, T051**. Downstream directo: **T056, T060**.

**KPIs PASS/FAIL.**

- Invalid compositions rejected 100%.
- Composition proof emitted for every accepted chain.
- Associativity only claimed where semantics prove it.


**Ejemplo mínimo de implementación/configuración.**

```text
ensure!(solver.entails(a.post(), b.pre())?, ComposeError::Precondition);
```

**GitHub/CI.** Composition proof replay.


## T056 — Interaction/Order Diagnostic

**Objetivo falsable.** Calcular diferencias de `U_a∘U_b` vs `U_b∘U_a` en domains finitos/simulables y etiquetarlo diagnóstico, no prueba causal.

**Artefactos exactos.** `crates/origin-causal/src/interaction.rs`.

**Conexiones.** Upstream: **T054, T055**. Downstream directo: **T060**.

**KPIs PASS/FAIL.**

- 0 diagnostic promoted automatically to causal status.
- Exact agreement in finite worlds.
- Budget-bounded for large domains.


**Ejemplo mínimo de implementación/configuración.**

```rust
let delta = compare(apply(a, apply(b, s)?), apply(b, apply(a, s)?));
return Diagnostic::OrderInteraction(delta);
```

**GitHub/CI.** Static check prohibits promotion call in diagnostic module.


## T057 — A* Deterministic Planner

**Objetivo falsable.** Implementar A* con cost/risk/uncertainty term, canonical state IDs y admissible heuristic interface.

**Artefactos exactos.** `crates/origin-plan/src/astar.rs`.

**Conexiones.** Upstream: **T051, T049**. Downstream directo: **T058, T059, T060**.

**KPIs PASS/FAIL.**

- Optimality 100% when heuristic declared admissible on finite reference suite.
- Deterministic tie-breaking.
- No unbounded allocation; budget stops explicit.


**Ejemplo mínimo de implementación/configuración.**

```text
f(n) = g_cost(n) + lambda_r*risk(n) + lambda_u*uncertainty(n) + h(n)
```

**GitHub/CI.** Optimality differential tests.


## T058 — IDA*/AO* Memory-Bounded Alternatives

**Objetivo falsable.** Añadir planners memory-bounded y AND/OR solo detrás de interfaz común; selección por problem signature, no ML.

**Artefactos exactos.** `crates/origin-plan/src/ida.rs`, `crates/origin-plan/src/ao.rs`, `crates/origin-plan/src/select.rs`.

**Conexiones.** Upstream: **T057, T028**. Downstream directo: **T060**.

**KPIs PASS/FAIL.**

- Selector deterministic.
- IDA* memory ≤25% de A* en deep benchmark con ≤20% cost overhead objetivo.
- AO* matches reference on AND/OR worlds.


**Ejemplo mínimo de implementación/configuración.**

```text
match sig.memory_pressure {
    High => PlannerKind::IdaStar,
    _ if sig.and_or => PlannerKind::AoStar,
    _ => PlannerKind::AStar,
}
```

**GitHub/CI.** Planner selection telemetry in benchmarks.


## T059 — Epistemic Query Planner

**Objetivo falsable.** Elegir QUERY/OBSERVE que minimiza worst-case residual ambiguity por coste, usando exact enumeration en small sets y bounds en large.

**Artefactos exactos.** `crates/origin-plan/src/query.rs`.

**Conexiones.** Upstream: **T015, T049, T057**. Downstream directo: **T060, T066**.

**KPIs PASS/FAIL.**

- Optimal query =100% vs exhaustive oracle en worlds pequeños.
- ≥50% fewer external queries than fixed-order baseline en benchmark active-info.
- Nunca excede query budget.


**Ejemplo mínimo de implementación/configuración.**

```rust
score(q) = worst_case_remaining_classes(q) * q.cost();
let best = queries.min_by_key(score);
```

**GitHub/CI.** Active-info benchmark required.


## T060 — G05 Causal Planning Gate

**Objetivo falsable.** Integrar causal types, journal, counterfactuals, composition, planning y query selection.

**Artefactos exactos.** `reports/gates/G05.md`, `bench/causal_worlds/`.

**Conexiones.** Upstream: **T051, T052, T053, T054, T055, T056, T057, T058, T059**. Downstream directo: **T061, T088, T098**.

**KPIs PASS/FAIL.**

- False verified-causal promotion =0.
- Plan optimality 100% en finite admissible suite.
- Counterfactual real-effect leaks =0.
- External query count ≥50% mejor que fixed-order baseline.


**Ejemplo mínimo de implementación/configuración.**

```rust
assert_eq!(report.false_causal_promotions, 0);
assert_eq!(report.effect_leaks, 0);
assert!(report.query_reduction >= 0.50);
```

**GitHub/CI.** `G05` required; causal benchmark report uploaded.


# P06 — JAX Numerical Coprocessor (No Training)


## T061 — JAX Zero-Train Numerical Boundary

**Objetivo falsable.** Definir que JAX recibe arrays/structures derivados y devuelve scores/simulations; nunca muta state autoritativo ni contiene trainable parameters.

**Artefactos exactos.** `python/origin_jax/boundary.py`, `spec/jax_boundary.md`.

**Conexiones.** Upstream: **T050, T060**. Downstream directo: **T062, T070**.

**KPIs PASS/FAIL.**

- Trainable params =0.
- No filesystem/network writes desde kernels JIT.
- Todas salidas revalidadas por Rust boundary.


**Ejemplo mínimo de implementación/configuración.**

```python
# INVARIANT: pure numerical coprocessor; no authoritative writes.
def evaluate(batch):
    return jax.jit(_evaluate)(batch)
```

**GitHub/CI.** Zero-training scanner inspecciona `python/origin_jax/`.


## T062 — Static PyTree Schemas

**Objetivo falsable.** Definir PyTrees/shapes dtypes versionados para CandidateBatch, IntervalBatch y OperatorBatch; evitar recompiles accidentales.

**Artefactos exactos.** `python/origin_jax/schema.py`, `spec/jax_shapes.md`.

**Conexiones.** Upstream: **T061**. Downstream directo: **T063, T064, T065, T066, T067, T070**.

**KPIs PASS/FAIL.**

- 0 dynamic Python objects cruzan JIT boundary.
- Recompile rate <1% steady-state benchmark.
- Schema hash incluido en compiled artifact metadata.


**Ejemplo mínimo de implementación/configuración.**

```python
@jax.tree_util.register_dataclass
@dataclass(frozen=True)
class CandidateBatch:
    values: jax.Array   # [N,D], fixed dtype
    masks: jax.Array    # [N]
```

**GitHub/CI.** JAX compile-cache stats artifact.


## T063 — Vectorized Hypothesis Scoring

**Objetivo falsable.** Evaluar lotes de hipótesis con `vmap`/pure functions y deterministic tie-breaking.

**Artefactos exactos.** `python/origin_jax/hypothesis.py`.

**Conexiones.** Upstream: **T042, T062**. Downstream directo: **T068, T069, T070**.

**KPIs PASS/FAIL.**

- Rust scalar vs JAX vector exact for integer kernels; float ≤2 ULP o tolerance spec.
- ≥20× throughput vs Python scalar at N≥4096.
- No ranking instability under repeated runs same backend.


**Ejemplo mínimo de implementación/configuración.**

```python
score_many = jax.jit(jax.vmap(score_one))
scores = score_many(candidate_batch)
```

**GitHub/CI.** Differential + perf check.


## T064 — Interval Arithmetic Kernels

**Objetivo falsable.** Implementar intervalos outward-rounded/validated para bounds numéricos relevantes a verificación.

**Artefactos exactos.** `python/origin_jax/interval.py`, `crates/origin-numeric/src/interval_ref.rs`.

**Conexiones.** Upstream: **T062, T039**. Downstream directo: **T068, T069, T070**.

**KPIs PASS/FAIL.**

- True value contenido en interval 100% en oracle high-precision suite.
- No NaN silently accepted.
- Width inflation benchmark reportado; pathological cases fallback Rust/high-precision.


**Ejemplo mínimo de implementación/configuración.**

```python
def add_interval(a_lo,a_hi,b_lo,b_hi):
    return lower_round(a_lo+b_lo), upper_round(a_hi+b_hi)
```

**GitHub/CI.** High-precision oracle job nightly.


## T065 — Batched Counterfactual Simulation

**Objetivo falsable.** Simular miles de candidate states/operators en paralelo sin side effects y devolver summaries verificables.

**Artefactos exactos.** `python/origin_jax/counterfactual.py`.

**Conexiones.** Upstream: **T054, T062**. Downstream directo: **T068, T069, T070**.

**KPIs PASS/FAIL.**

- No external effects by construction.
- Reference Rust match within spec.
- ≥10× throughput at batch≥2048 vs Rust scalar baseline, o JAX path se desactiva.


**Ejemplo mínimo de implementación/configuración.**

```python
simulate_many = jax.jit(jax.vmap(simulate_one))
# PURE: counterfactual arrays only; no capabilities cross this boundary.
```

**GitHub/CI.** Performance feature gate enabled only when threshold passes.


## T066 — Vectorized Query Scoring

**Objetivo falsable.** Calcular information gain/worst-case class reduction para miles de queries candidatas.

**Artefactos exactos.** `python/origin_jax/query_score.py`.

**Conexiones.** Upstream: **T059, T062**. Downstream directo: **T068, T069, T070**.

**KPIs PASS/FAIL.**

- Exact match con exhaustive Rust small-world oracle.
- ≥15× throughput at 4096 queries.
- Ties resolved canonically by ORID in Rust.


**Ejemplo mínimo de implementación/configuración.**

```python
scores = jax.jit(jax.vmap(worst_case_score))(queries, world_signatures)
```

**GitHub/CI.** Result ORIDs and metrics attached to benchmark.


## T067 — JAX Control-Flow Kernels

**Objetivo falsable.** Usar `lax.scan/while_loop/cond` solo para loops numéricos acotados y evitar unroll/recompile patológico.

**Artefactos exactos.** `python/origin_jax/control.py`.

**Conexiones.** Upstream: **T062**. Downstream directo: **T068, T069, T070**.

**KPIs PASS/FAIL.**

- HLO size grows O(1) w.r.t. scan length where applicable.
- Compile time <5s p95 for production signatures target.
- Runtime parity con reference loops.


**Ejemplo mínimo de implementación/configuración.**

```python
carry, ys = jax.lax.scan(step, init, xs)
```

**GitHub/CI.** Compile-time benchmark tracked.


## T068 — AOT/StableHLO Export

**Objetivo falsable.** Exportar kernels JAX maduros con signature/schema hash y manifest de backend; no exportar kernels no estables.

**Artefactos exactos.** `python/origin_jax/export.py`, `artifacts/stablehlo/`, `spec/numeric_artifact.md`.

**Conexiones.** Upstream: **T063, T064, T065, T066, T067**. Downstream directo: **T069, T070, T079**.

**KPIs PASS/FAIL.**

- 100% exported artifacts include schema+source hash+JAX version.
- Load/reexecute matches source kernel.
- Stale schema prevents load 100%.


**Ejemplo mínimo de implementación/configuración.**

```python
exported = jax.export.export(jax.jit(kernel))(*abstract_args)
write_manifest(exported, schema_hash, source_orid)
```

**GitHub/CI.** Export artifacts built only in attested release workflow.


## T069 — Rust↔JAX Differential Harness

**Objetivo falsable.** Generar mismos inputs y comparar resultados, exceptions, bounds y determinism en todos kernels JAX.

**Artefactos exactos.** `crates/origin-numeric-test/`, `python/tests/differential/`.

**Conexiones.** Upstream: **T063, T064, T065, T066, T067, T068**. Downstream directo: **T070**.

**KPIs PASS/FAIL.**

- Integer/bit kernels exact 100%.
- Float: ≤2 ULP o task-specific proven interval.
- ≥1e7 randomized cases nightly; 0 unexplained mismatches.


**Ejemplo mínimo de implementación/configuración.**

```text
rust = rust_reference(case)
jaxv = jax_kernel(case)
assert within_contract(rust, jaxv)
```

**GitHub/CI.** Nightly differential matrix CPU/GPU where runner available.


## T070 — G06 Numerical Coprocessor Gate

**Objetivo falsable.** Aceptar JAX solo donde preserva contratos y demuestra throughput; resto permanece Rust.

**Artefactos exactos.** `reports/gates/G06.md`.

**Conexiones.** Upstream: **T061, T062, T063, T064, T065, T066, T067, T068, T069**. Downstream directo: **T077, T088**.

**KPIs PASS/FAIL.**

- Trainable params=0.
- 0 unexplained differential mismatches.
- Every enabled JAX fast path ≥10× throughput or ≥30% end-to-end win en su workload.
- No authoritative state mutation.


**Ejemplo mínimo de implementación/configuración.**

```text
assert trainable_parameter_count() == 0
assert unexplained_mismatches == 0
assert all(path.justified for path in enabled_jax_paths)
```

**GitHub/CI.** `G06` required; disabled paths no se empaquetan por defecto.


# P07 — OIR & Certified Cognitive Compiler


## T071 — OIR Core IR

**Objetivo falsable.** Definir SSA-like OIR para 9 opcodes, values tipados, effects y source ORIDs; textual + binary form canónica.

**Artefactos exactos.** `crates/origin-oir/src/ir.rs`, `spec/oir.md`.

**Conexiones.** Upstream: **T019, T050**. Downstream directo: **T072, T073, T078, T080**.

**KPIs PASS/FAIL.**

- 100% OIR nodes source-map a ORID/spec origin.
- Binary/text round-trip 100%.
- Invalid opcode/type combinations unrepresentable o rejected.


**Ejemplo mínimo de implementación/configuración.**

```text
%h1 = propose.claim %obs0 : !origin.claim
%v2 = verify %h1 with %proof : !origin.verified_claim
```

**GitHub/CI.** OIR grammar/codegen checked in CI.


## T072 — OIR Type Checker

**Objetivo falsable.** Validar schemas, statuses, domains/codomains, claim kinds y operator signatures antes de ejecución.

**Artefactos exactos.** `crates/origin-oir/src/typecheck.rs`.

**Conexiones.** Upstream: **T071, T012, T018**. Downstream directo: **T074, T080**.

**KPIs PASS/FAIL.**

- Reject 100% corpus invalid IR.
- Accept 100% canonical valid corpus.
- Typecheck throughput ≥1M simple ops/s.


**Ejemplo mínimo de implementación/configuración.**

```text
ensure_eq!(value.ty(), op.expected_input_ty(), TypeError::Mismatch);
```

**GitHub/CI.** Negative corpus required.


## T073 — OIR Effect Checker

**Objetivo falsable.** Probar que PURE/READ regions no contienen QUERY_EXTERNAL, INTERVENE, COMMIT indirectos.

**Artefactos exactos.** `crates/origin-oir/src/effectcheck.rs`.

**Conexiones.** Upstream: **T071, T029**. Downstream directo: **T074, T077, T080, T093**.

**KPIs PASS/FAIL.**

- Effect escalation acceptance=0 en ≥1e6 generated programs.
- Call graph transitive effects resolved 100%.
- Unknown external call defaults deny.


**Ejemplo mínimo de implementación/configuración.**

```text
if caller.effect < callee.effect { return Err(EffectError::Escalation); }
```

**GitHub/CI.** Security CODEOWNER required.


## T074 — OIR Verifier & Invariant Pass

**Objetivo falsable.** Validar status promotions, obligation witnesses, provenance references, budgets y artifact freshness en IR.

**Artefactos exactos.** `crates/origin-oir/src/verify.rs`.

**Conexiones.** Upstream: **T072, T073, T035**. Downstream directo: **T075, T076, T077, T080, T092**.

**KPIs PASS/FAIL.**

- Malformed semantic IR accepted=0.
- Verifier deterministic 100%.
- Verifier p99 <5ms for 100k-op module target.


**Ejemplo mínimo de implementación/configuración.**

```text
verify_types(m)?;
verify_effects(m)?;
verify_epistemic_invariants(m)?;
verify_budgets(m)?;
```

**GitHub/CI.** Verifier fuzz target mandatory.


## T075 — E-Graph OIR Optimizer

**Objetivo falsable.** Optimizar OIR con rewrites proof-tagged sin cruzar effect/status boundaries.

**Artefactos exactos.** `crates/origin-oir/src/opt.rs`.

**Conexiones.** Upstream: **T045, T074**. Downstream directo: **T076, T080**.

**KPIs PASS/FAIL.**

- Semantic equivalence 100% on bounded exhaustive programs.
- No effect reordering across barriers.
- Optimization accepted only if estimated+measured cost improves ≥5%.


**Ejemplo mínimo de implementación/configuración.**

```text
rewrite!("fold-pure-and"; "(and true ?x)" => "?x"; proof = AX_BOOL_IDENTITY);
```

**GitHub/CI.** Optimizer ablation and differential check.


## T076 — OIR→Rust Lowering

**Objetivo falsable.** Generar Rust para graph/control paths con explicit guards, budget charges y typed error propagation.

**Artefactos exactos.** `crates/origin-codegen-rust/`.

**Conexiones.** Upstream: **T074, T075**. Downstream directo: **T080**.

**KPIs PASS/FAIL.**

- Generated Rust passes same verifier tests.
- No `unsafe` emitted.
- Slow→generated speedup ≥1.5× on accepted artifacts.


**Ejemplo mínimo de implementación/configuración.**

```text
// generated from OIR; invariant guards are mandatory.
budget.charge(...)?;
if !guard(ctx) { return Err(Fallback::SlowPath); }
```

**GitHub/CI.** Generated source archived with artifact attestation.


## T077 — OIR→JAX Lowering

**Objetivo falsable.** Bajar regiones PURE numéricas a JAX functions/StableHLO usando schemas de T062; prohibir effects.

**Artefactos exactos.** `crates/origin-codegen-jax/`, `python/origin_jax/generated/`.

**Conexiones.** Upstream: **T070, T073, T074**. Downstream directo: **T080**.

**KPIs PASS/FAIL.**

- Only PURE regions lower.
- Rust reference parity per T069.
- Accepted lowered region meets T070 performance criterion.


**Ejemplo mínimo de implementación/configuración.**

```python
# generated only from PURE OIR region
@jax.jit
def kernel(inputs):
    return lowered_ops(inputs)
```

**GitHub/CI.** Generated JAX rechecked by zero-training guard.


## T078 — Stable-Region Detector

**Objetivo falsable.** Marcar candidatos a compilación solo si domain guard estable, hit count alto, no unresolved obligations y churn bajo.

**Artefactos exactos.** `crates/origin-compiler/src/stability.rs`.

**Conexiones.** Upstream: **T035, T049, T071**. Downstream directo: **T079, T080**.

**KPIs PASS/FAIL.**

- 0 artifact compiled con unresolved critical obligation.
- False-stable rate <0.1% en churn benchmark.
- Minimum hit threshold configurable y versionado.


**Ejemplo mínimo de implementación/configuración.**

```text
eligible = hits >= H_MIN
    && churn <= CHURN_MAX
    && unresolved_critical == 0;
```

**GitHub/CI.** Compilation decision logged to benchmark trace.


## T079 — Artifact Guard + Dependency Invalidation

**Objetivo falsable.** Generar guard de dominio y dependency root; cualquier cambio upstream o schema mismatch marca STALE antes de execute.

**Artefactos exactos.** `crates/origin-compiler/src/artifact.rs`.

**Conexiones.** Upstream: **T025, T068, T078**. Downstream directo: **T080, T087, T093**.

**KPIs PASS/FAIL.**

- Stale artifact execution=0 en ≥1e6 mutation scenarios.
- Guard false-negative=0 en bounded reference domains.
- Guard overhead <5% p50 fast path.


**Ejemplo mínimo de implementación/configuración.**

```text
ensure!(artifact.dep_root == current.dep_root, ArtifactError::Stale);
ensure!(artifact.guard.accepts(input), Fallback::SlowPath);
```

**GitHub/CI.** Artifact invalidation required check.


## T080 — G07 Certified Compilation Gate

**Objetivo falsable.** Aceptar cognitive compilation solo si equivalencia, guard, provenance y speedup están demostrados.

**Artefactos exactos.** `reports/gates/G07.md`.

**Conexiones.** Upstream: **T071, T072, T073, T074, T075, T076, T077, T078, T079**. Downstream directo: **T081, T088, T098**.

**KPIs PASS/FAIL.**

- Semantic divergence=0 in bounded exhaustive suite.
- Stale executions=0.
- Fast artifact speedup ≥3× vs slow path median AND ≥2× p99, o artifact se rechaza.
- 100% artifacts poseen source/dependency/build identity.


**Ejemplo mínimo de implementación/configuración.**

```rust
assert_eq!(report.semantic_divergence, 0);
assert_eq!(report.stale_exec, 0);
assert!(report.speedup_median >= 3.0 && report.speedup_p99 >= 2.0);
```

**GitHub/CI.** `G07` required; rejected artifacts never ship.


# P08 — Rust SIMD, x86-64 Assembly & Two-Speed Runtime


## T081 — Profiler-First Runtime

**Objetivo falsable.** Instrumentar cycles, allocations, cache misses estimados, branch behavior, graph hot edges y JAX compile/runtime before hand-written Assembly.

**Artefactos exactos.** `crates/origin-profiler/`, `tools/profile.sh`, `bench/profiles/`.

**Conexiones.** Upstream: **T030, T080**. Downstream directo: **T082, T085, T086, T090, T096**.

**KPIs PASS/FAIL.**

- ≥95% CPU time atribuible a named spans en benchmark.
- No Assembly task puede activarse sin profile artifact.
- Profiler disabled overhead <1%; enabled <5%.


**Ejemplo mínimo de implementación/configuración.**

```rust
let _span = profiler::span("graph.scan");
scan_relations(...);
```

**GitHub/CI.** Profile artifacts uploaded nightly.


## T082 — CPU Feature Dispatch

**Objetivo falsable.** Detectar AVX2/BMI2/POPCNT/SHA extensions y seleccionar implementation con pure-Rust reference fallback.

**Artefactos exactos.** `crates/origin-fast/src/dispatch.rs`.

**Conexiones.** Upstream: **T081**. Downstream directo: **T083, T084, T086, T087, T090**.

**KPIs PASS/FAIL.**

- Fallback correctness 100%.
- Unknown CPU feature set never executes unsupported instruction.
- Dispatch overhead <20ns p50 cached.


**Ejemplo mínimo de implementación/configuración.**

```text
if is_x86_feature_detected!("avx2") { Impl::Avx2 } else { Impl::Scalar }
```

**GitHub/CI.** CI scalar forced path + AVX2 runner when available.


## T083 — Packed Bitset Reference + AVX2 Path

**Objetivo falsable.** Implementar intersection/union/difference first in safe Rust then AVX2; Assembly only if intrinsics fail performance gate.

**Artefactos exactos.** `crates/origin-fast/src/bitset.rs`, `asm/x86_64/bitset.S`.

**Conexiones.** Upstream: **T082, T049**. Downstream directo: **T084, T090, T092**.

**KPIs PASS/FAIL.**

- Bitwise identity 100% across ≥1e8 random words nightly.
- AVX2 ≥3× scalar for ≥64KiB bitsets.
- Hand ASM retained only if ≥1.15× intrinsics and no p99 regression.


**Ejemplo mínimo de implementación/configuración.**

```text
// SAFETY: len is multiple of 32; pointers non-overlapping and aligned/unaligned-safe variant chosen.
unsafe { bitset_and_avx2(dst, a, b, len) };
```

**GitHub/CI.** ASM diff-fuzz required; `SAFETY:` comment lint.


## T084 — POPCNT/Cardinality Fast Path

**Objetivo falsable.** Optimizar cardinalidad de relevance/evidence masks con portable fallback y chunked POPCNT.

**Artefactos exactos.** `crates/origin-fast/src/cardinality.rs`, `asm/x86_64/popcnt.S`.

**Conexiones.** Upstream: **T082, T083**. Downstream directo: **T090**.

**KPIs PASS/FAIL.**

- Exact match 100%.
- ≥4× scalar naive on ≥1MiB masks.
- ASM retained only if ≥1.10× compiler/intrinsics.


**Ejemplo mínimo de implementación/configuración.**

```text
// KPI: exact cardinality; no approximate count.
sum += chunk.count_ones() as u64;
```

**GitHub/CI.** Perf gate decides feature inclusion.


## T085 — Packed Status/Relation Scanner

**Objetivo falsable.** Definir SoA packed layout para status+relation filters y vectorizar scans calientes.

**Artefactos exactos.** `crates/origin-fast/src/scan.rs`, `bench/layout/`.

**Conexiones.** Upstream: **T027, T081**. Downstream directo: **T090**.

**KPIs PASS/FAIL.**

- ≥2× baseline AoS throughput en chosen hot workload.
- Memory/edge ≤24 bytes target for packed index metadata.
- No authoritative data stored only in packed cache.


**Ejemplo mínimo de implementación/configuración.**

```text
struct PackedIndex {
    kinds: Vec<u8>, src: Vec<u32>, dst: Vec<u32>, status: Vec<u8>
}
```

**GitHub/CI.** Layout benchmark report required before merge.


## T086 — Fast ORID Hash Batch

**Objetivo falsable.** Batch hash/verification de small canonical objects usando hardware acceleration only after profiling.

**Artefactos exactos.** `crates/origin-fast/src/hash.rs`.

**Conexiones.** Upstream: **T023, T081, T082**. Downstream directo: **T090**.

**KPIs PASS/FAIL.**

- Hash identity exact 100%.
- ≥1.5× baseline hasher throughput to enable.
- No alternate digest semantics; same ORID bytes.


**Ejemplo mínimo de implementación/configuración.**

```rust
let digest = fast_hash(domain_sep, canonical);
debug_assert_eq!(digest, reference_hash(domain_sep, canonical));
```

**GitHub/CI.** Cross-impl hash golden vectors.


## T087 — Fast Artifact Executor

**Objetivo falsable.** Ejecutar compiled artifacts con guard→dependency check→budget→kernel; fallback inmediato al slow path.

**Artefactos exactos.** `crates/origin-runtime/src/fast.rs`.

**Conexiones.** Upstream: **T079, T082, T028**. Downstream directo: **T089, T090**.

**KPIs PASS/FAIL.**

- Guard+freshness path p99 <50µs target for tiny artifact.
- Fallback correctness 100%.
- Fast result semantic parity 100% bounded suite.


**Ejemplo mínimo de implementación/configuración.**

```text
if artifact.is_fresh(ctx) && artifact.guard(input) {
    return artifact.execute(input);
}
slow_path(input)
```

**GitHub/CI.** Fast/slow differential check required.


## T088 — Slow Deliberative Runtime

**Objetivo falsable.** Orquestar active slice→reason→query/plan→verify→commit con deadline/budget y deterministic event log.

**Artefactos exactos.** `crates/origin-runtime/src/slow.rs`.

**Conexiones.** Upstream: **T040, T050, T060, T070, T080**. Downstream directo: **T089, T090, T096**.

**KPIs PASS/FAIL.**

- Every slow step emits deterministic event record.
- Timeout returns typed Stop, no silent partial commit.
- No action executes before verification/capability gates.


**Ejemplo mínimo de implementación/configuración.**

```rust
let slice = active_slice(goal)?;
let proposal = reason(slice, budget)?;
let verified = verify(proposal)?;
commit(verified)
```

**GitHub/CI.** Replay event log in CI.


## T089 — Unified Two-Speed Scheduler

**Objetivo falsable.** Elegir fast artifact only on exact guard hit; otherwise slow path; collect hit/miss/churn stats without ML.

**Artefactos exactos.** `crates/origin-runtime/src/scheduler.rs`.

**Conexiones.** Upstream: **T087, T088**. Downstream directo: **T090, T097**.

**KPIs PASS/FAIL.**

- Wrong-fast-path selection=0.
- Scheduler overhead <2% end-to-end.
- Fast hit rate ≥70% en mature-workload benchmark para justificar compilation strategy.


**Ejemplo mínimo de implementación/configuración.**

```text
match cache.lookup(input.signature()) {
    Some(a) if a.valid_for(input) => run_fast(a, input),
    _ => run_slow(input),
}
```

**GitHub/CI.** Scheduler KPI reported in PR benchmark comment.


## T090 — G08 Fast Runtime & Assembly Gate

**Objetivo falsable.** Aceptar SIMD/Assembly solo si exacto, seguro, portable por fallback y útil end-to-end.

**Artefactos exactos.** `reports/gates/G08.md`.

**Conexiones.** Upstream: **T081, T082, T083, T084, T085, T086, T087, T088, T089**. Downstream directo: **T091, T094, T098**.

**KPIs PASS/FAIL.**

- ASM/Rust unexplained mismatch=0.
- Every unsafe block has `SAFETY:` + owner.
- No SIGILL on unsupported CPU matrix.
- End-to-end mature workload ≥2× vs no-fast build; otherwise feature removed/reworked.


**Ejemplo mínimo de implementación/configuración.**

```rust
assert_eq!(diff_mismatches, 0);
assert_eq!(unsupported_cpu_faults, 0);
assert!(e2e_speedup >= 2.0);
```

**GitHub/CI.** `G08` required on release branch; scalar build always tested.


# P09 — Post-Frontier Production, Security, GitHub Release & Falsification


## T091 — Crash/Power-Loss Recovery Campaign

**Objetivo falsable.** Inyectar kills y write truncation en object store/WAL/commit/root publication y demostrar recovery.

**Artefactos exactos.** `crates/origin-chaos/`, `bench/chaos/crash_matrix.toml`.

**Conexiones.** Upstream: **T026, T090**. Downstream directo: **T093, T094**.

**KPIs PASS/FAIL.**

- ≥1e6 injected crash points release campaign.
- Invalid published roots=0.
- Recovery returns last committed root 100%.


**Ejemplo mínimo de implementación/configuración.**

```text
chaos::kill_after(NthIo(n));
restart();
assert_eq!(store.current_root()?, last_committed_root);
```

**GitHub/CI.** Nightly reduced matrix; release full chaos artifact.


## T092 — Fuzzing + Miri + Sanitizer Strategy

**Objetivo falsable.** Fuzz canonical codec/OIR/solver/FFI; Miri en safe/unsafe boundaries compatibles y sanitizers donde toolchain lo permita.

**Artefactos exactos.** `fuzz/`, `tools/miri.sh`, `tools/sanitizers.sh`.

**Conexiones.** Upstream: **T022, T029, T074, T083**. Downstream directo: **T093, T094**.

**KPIs PASS/FAIL.**

- 0 reproducible crashes in 24 CPU-hours/critical target release campaign.
- All found bugs minimized+regression test.
- Unsafe boundary corpus ≥1e7 cases.


**Ejemplo mínimo de implementación/configuración.**

```rust
fuzz_target!(|bytes: &[u8]| {
    let _ = CanonicalDecoder::decode(bytes);
});
```

**GitHub/CI.** OSS-fuzz compatible layout; GitHub scheduled fuzz workflow.


## T093 — Security Red-Team Matrix

**Objetivo falsable.** Atacar forged ORIDs, provenance laundering, capability escalation, prompt/data-as-instruction confusion, stale artifacts y malicious OIR.

**Artefactos exactos.** `security/THREAT_MODEL.md`, `security/redteam/`, `reports/security/`.

**Conexiones.** Upstream: **T029, T032, T073, T079, T091, T092**. Downstream directo: **T094**.

**KPIs PASS/FAIL.**

- Critical bypasses=0.
- 100% previously found exploits have regression tests.
- Time-to-detect simulated forged/stale artifact within same operation.


**Ejemplo mínimo de implementación/configuración.**

```rust
assert!(execute(untrusted_observation("INTERVENE(...)")).is_observation_only());
```

**GitHub/CI.** Security report required for release; CODEOWNER approval.


## T094 — Reproducible Release + SBOM + Attestation

**Objetivo falsable.** Construir release desde reusable workflow, generar SBOM, checksums, attestations y verificar antes de GitHub Release.

**Artefactos exactos.** `.github/workflows/release.yml`, `tools/release_verify.sh`, `dist/`.

**Conexiones.** Upstream: **T008, T090, T091, T092, T093**. Downstream directo: **T095, T099**.

**KPIs PASS/FAIL.**

- 100% release artifacts attested+SBOM.
- Independent rebuild reproducibility: identical source manifest; bit-identical where toolchain supports target, otherwise documented variance.
- Attestation verification mandatory pre-publish.


**Ejemplo mínimo de implementación/configuración.**

```text
gh attestation verify "$ARTIFACT" --repo "$GITHUB_REPOSITORY"
# INVARIANT: publish step depends on successful verification.
```

**GitHub/CI.** GitHub Release job gated by attestation verify.


## T095 — GitHub Merge Queue/Ruleset Production Lock

**Objetivo falsable.** Aplicar ruleset final: PR+CODEOWNERS+required checks+linear history+signed/verified policy si enabled+merge queue para main.

**Artefactos exactos.** `.github/ruleset.production.json`, `docs/governance/main-branch.md`.

**Conexiones.** Upstream: **T005, T007, T094**. Downstream directo: **T099**.

**KPIs PASS/FAIL.**

- 0 direct pushes main.
- All required checks sourced from expected GitHub App/workflow.
- Stale approvals dismissed on code change.
- Release tags protected.


**Ejemplo mínimo de implementación/configuración.**

```yaml
required_checks:
  - G00
  - G01
  - G02
  - G03
  - G04
  - G05
  - G06
  - G07
  - G08
  - security
```

**GitHub/CI.** Ruleset as-code mirrored and audited; actual repo configuration verified by `gh api` script.


## T096 — Observability & Epistemic Debugger

**Objetivo falsable.** Entregar CLI `origin why/why-not/evidence/history/obligations/causal/replay/profile` con machine-readable JSON output.

**Artefactos exactos.** `crates/origin-cli/`, `docs/debugger.md`.

**Conexiones.** Upstream: **T032, T035, T053, T081, T088**. Downstream directo: **T097, T099**.

**KPIs PASS/FAIL.**

- 100% VERIFIED claims explainable to roots.
- Debugger query p99 <100ms on 1M-edge graph for indexed paths.
- JSON schema stable and versioned.


**Ejemplo mínimo de implementación/configuración.**

```text
origin why <ORID> --json
origin replay <COMMIT_ROOT> --verify
```

**GitHub/CI.** CLI smoke tests on every release asset.


## T097 — Long-Horizon & Scale Campaign

**Objetivo falsable.** Ejecutar millones de commits/claims/relations con corrections, contradictions, branches y compaction/rebuild de índices.

**Artefactos exactos.** `bench/scale/`, `reports/scale/`.

**Conexiones.** Upstream: **T027, T036, T049, T089, T096**. Downstream directo: **T098, T099**.

**KPIs PASS/FAIL.**

- ≥10M objects + ≥50M relations benchmark completes without semantic divergence.
- Active-slice median ≤5% graph; p99 ≤20%.
- Index rebuild exact 100%.
- Memory growth per persisted relation within documented bound; no unbounded leak.


**Ejemplo mínimo de implementación/configuración.**

```text
for i in 0..10_000_000 {
    ingest_synthetic_event(i)?;
}
assert_eq!(rebuild_indexes(root)?, live_indexes);
```

**GitHub/CI.** Nightly scaled-down; release full scale on designated runner.


## T098 — Baseline + Ablation Matrix

**Objetivo falsable.** Comparar ORIGIN ZERO contra naive rule engine, SAT-only, KG-only, planner-only y variantes ORIGIN sin quotient/egraph/compiler/active-query; LLM+tools puede figurar solo como referencia externa no dependency.

**Artefactos exactos.** `bench/baselines/`, `reports/ablation/`.

**Conexiones.** Upstream: **T009, T050, T060, T080, T090, T097**. Downstream directo: **T099**.

**KPIs PASS/FAIL.**

- Every claimed component has measured marginal value or is removed.
- No baseline intentionally crippled.
- 95% confidence intervals reported; ≥5 independent runs for noisy benches.
- At least one primary workload must show statistically significant advantage on reliability-per-resource to keep `post-frontier candidate` label.


**Ejemplo mínimo de implementación/configuración.**

```text
for variant in ablations {
    run_same_manifest(variant, seeds)?;
}
require_no_component_without_measured_value();
```

**GitHub/CI.** Benchmark results posted to PR/release, raw JSON archived.


## T099 — Release Candidate Acceptance

**Objetivo falsable.** Congelar RC, rerun all gates from clean checkout, verify docs/API/ABI, supply-chain, security, performance and zero-training.

**Artefactos exactos.** `reports/gates/RC.md`, `CHANGELOG.md`, `RELEASE.md`.

**Conexiones.** Upstream: **T094, T095, T096, T097, T098**. Downstream directo: **T100**.

**KPIs PASS/FAIL.**

- G00–G08 all green.
- 0 Critical/High security findings open.
- 0 undocumented unsafe blocks.
- Zero-training scanner green.
- No primary KPI regression >5% vs previous accepted commit unless waived by architecture ADR.


**Ejemplo mínimo de implementación/configuración.**

```text
for gate in GATES_00_TO_08 { require(gate.status == PASS); }
require(security.high_open == 0);
require(trainable_parameter_count() == 0);
```

**GitHub/CI.** RC tag created only by protected release workflow.


## T100 — G09 Post-Frontier Truth Gate

**Objetivo falsable.** Emitir veredicto final: arquitectura, no slogan. Publicar pass/fail por claim, raw benchmarks, limitations y kill decisions.

**Artefactos exactos.** `reports/gates/G09_POST_FRONTIER.md`, `reports/final_metrics.json`, `docs/LIMITATIONS.md`.

**Conexiones.** Upstream: **T099**. Downstream directo: **T100/release final**.

**KPIs PASS/FAIL.**

- Correctness invariants: 100% on defined suite.
- Illegal VERIFIED/causal promotions: 0.
- Unauthorized effects/stale artifact executions: 0.
- Zero training: 0 learned/trainable params.
- Reproducible provenance: 100% release artifacts.
- At least one preregistered primary benchmark shows ≥2× reliability-per-compute OR ≥2× query-efficiency OR ≥3× mature fast-path speed with no correctness loss vs relevant strongest non-neural baseline; otherwise status remains `research architecture`, not `post-frontier proven`.


**Ejemplo mínimo de implementación/configuración.**

```text
// FINAL INVARIANT: claims are earned by reproducible evidence.
if !primary_advantage_is_proven() {
    label = "research-architecture";
} else {
    label = "post-frontier-candidate";
}
```

**GitHub/CI.** Final report attached to signed/attested GitHub Release; no manual green override.


# GitHub Production Contract

## Required checks propuestos para `main`
`zero-training`, `fmt`, `clippy`, `unit`, `property`, `docs`, `python`, `CodeQL`, `dependency-review`, `G00`, `G01`, `G02`, `G03`, `G04`, `G05`, `G06`, `G07`, `G08`, `security`.

## Workflow topology
```text
pull_request
  ├─ _ci.yml
  ├─ zero-training.yml
  ├─ codeql.yml
  ├─ dependency-review.yml
  └─ benchmark-smoke.yml

schedule/nightly
  ├─ property-heavy.yml
  ├─ differential.yml
  ├─ fuzz.yml
  ├─ chaos.yml
  └─ benchmark-full.yml

tag/v*
  └─ release.yml
      └─ _release-build.yml
          ├─ build
          ├─ SBOM
          ├─ artifact attestation
          ├─ attestation verification
          └─ GitHub Release publish
```

## Pull request policy
- No direct pushes a `main`.
- CODEOWNERS obligatorio en `kernel/`, `security/`, `asm/`, `.github/`.
- Dismiss stale approvals when the diff changes.
- Linear history.
- Force push disabled.
- Dependency review + CodeQL required.
- El check de benchmark puede ser smoke en PR; full en nightly/release.
- Un cambio a un gate requiere ADR y revisión de architecture + security owners.


# Regla final de auditoría de las 16 lentes

Una task **no pasa** por tener código. Pasa solo si:
1. **Jobs/Wirth:** el mecanismo es esencial y el developer surface sigue siendo pequeño.
2. **Torvalds/Gates:** integra, versiona y automatiza correctamente en GitHub.
3. **Lovelace/Turing:** la semántica de operación está explícita y ejecutable sin aprendizaje paramétrico.
4. **Hopper:** trabajo estable puede reutilizarse/compilarse.
5. **Ritchie/Wozniak/Carmack/Stroustrup:** coste de memoria/CPU/latencia se mide y no se paga overhead inútil.
6. **Thompson/Berners-Lee:** identidad, provenance, trust chain y build chain son verificables.
7. **Knuth/Guido:** el claim tiene prueba/KPI y los fallos/unknowns son explícitos.
8. **Musk lens:** deploy, recovery, throughput, determinismo y coste están dentro del Definition of Done.

Si cualquiera de estas reglas relevante a la task falla, el gate falla.


# Definition of Done

La v1 solo puede llamarse **Post-Frontier Candidate** si T100 pasa. Antes de eso el nombre correcto es **research architecture**.

**No se promete** lenguaje/visión humana general sin modelos entrenados. ZERO está diseñado para dominios formalizables, matemática, lógica, planificación, causal worlds, knowledge revision, program synthesis y agentes con interfaces estructuradas. Cualquier frontend perceptual futuro que use pesos aprendidos debe vivir fuera del contrato ZERO y no puede cambiar la autoridad epistemológica del kernel.
