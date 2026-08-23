# ORIGIN-Ω ZERO — Merge & Governance Policy (T005)

> **INVARIANT:** No direct commits to `main`. Linear history strictly enforced; zero force-pushes allowed.  
> **KPI:** 0 merges to `main` without 100% green required status checks + CODEOWNERS approval.

---

## 1. Contrato de Integración y Merge

1. **Pull Request Obligatorio**: Queda prohibido cualquier commit directo sobre `main` o tags de release.
2. **Historial Lineal Obligatorio (Non-Fast-Forward Protection)**:
   - Todo PR debe integrarse mediante rebase o squash merge. Se impone veto a commits de merge no lineales.
3. **Required Status Checks**:
   - `zero-training` (Task T001)
   - `ci` (Task T006)
   - Audit Gates de Fase (`G00` a `G09`)
4. **Revisión por Ownership (CODEOWNERS)**:
   - Modificaciones a `/crates/origin-kernel/`, `/crates/origin-core/`, `/asm/` o `/.github/` exigen la aprobación explícita de `@kernel-owners`, `@security-owners` o `@architecture-team`.
5. **Desestimación Automática de Aprobaciones**:
   - Ante la incorporación de nuevos commits a la rama del PR, las revisiones previas quedan desestimadas automáticamente (`dismiss_stale_reviews_on_push: true`).
6. **Bloqueo Rígido de Force-Push y Eliminación de Ramas**:
   - Bloqueo por regla de protección en GitHub Rulesets (`deletion` and `non_fast_forward` rules active).
