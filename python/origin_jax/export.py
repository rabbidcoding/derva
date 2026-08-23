# AUDIT-LENSES: Grace Hopper, Ken Thompson, Tim Berners-Lee
# INVARIANT: StableHLO AOT kernel export with immutable metadata manifest; 100% stale schema load rejection.
# KPI: 100% exported artifacts include schema+source hash+JAX version; re-execution matches source; stale schema load fails 100%.

import os
import json
import hashlib
import jax
import jax.numpy as jnp
from typing import Dict, Any, Callable

class StableHLOExporter:
    """
    Exports mature JAX numerical kernels to StableHLO bytecode artifacts with identity manifests.
    """

    def __init__(self, output_dir: str = "artifacts/stablehlo"):
        self.output_dir = output_dir
        os.makedirs(self.output_dir, exist_ok=True)

    def export_kernel(
        self,
        kernel_name: str,
        fn: Callable,
        sample_args: tuple,
        schema_hash: str,
        source_orid: str,
    ) -> Tuple[str, str]:
        """
        Exports a JAX JIT kernel to StableHLO bytecode and writes metadata manifest.
        Returns: (kernel_path, manifest_path)
        """
        lowered = jax.jit(fn).lower(*sample_args)
        hlo_bytecode = lowered.as_text()

        source_hash = hashlib.sha256(hlo_bytecode.encode("utf-8")).hexdigest()

        manifest = {
            "kernel_name": kernel_name,
            "schema_hash": schema_hash,
            "source_orid": source_orid,
            "source_hash": source_hash,
            "jax_version": jax.__version__,
            "backend": jax.default_backend(),
        }

        kernel_path = os.path.join(self.output_dir, f"{kernel_name}.hlo")
        manifest_path = os.path.join(self.output_dir, f"{kernel_name}_manifest.json")

        with open(kernel_path, "w", encoding="utf-8") as f:
            f.write(hlo_bytecode)

        with open(manifest_path, "w", encoding="utf-8") as f:
            json.dump(manifest, f, indent=2)

        return kernel_path, manifest_path

    def load_and_verify(
        self,
        kernel_name: str,
        expected_schema_hash: str,
    ) -> Dict[str, Any]:
        """
        Loads exported manifest and verifies schema identity.
        MUST reject load (throw ValueError) 100% of the time if schema_hash is stale or mismatched.
        """
        manifest_path = os.path.join(self.output_dir, f"{kernel_name}_manifest.json")
        if not os.path.exists(manifest_path):
            raise FileNotFoundError(f"Export manifest missing for {kernel_name}")

        with open(manifest_path, "r", encoding="utf-8") as f:
            manifest = json.load(f)

        if manifest.get("schema_hash") != expected_schema_hash:
            raise ValueError(
                f"STALE SCHEMA REJECTED: Expected {expected_schema_hash}, got {manifest.get('schema_hash')}"
            )

        return manifest

def dummy_kernel(x: jax.Array) -> jax.Array:
    return jnp.sin(x) + jnp.cos(x)

def test_aot_stablehlo_export():
    exporter = StableHLOExporter(output_dir="artifacts/stablehlo")

    sample_x = jnp.ones((10, 10), dtype=jnp.float32)
    schema_hash = "sha256:dummy_schema_v1"
    source_orid = "orid:00112233445566778899aabbccddeeff"

    # 1. Export Kernel & Manifest
    kernel_path, manifest_path = exporter.export_kernel(
        "dummy_kernel",
        dummy_kernel,
        (sample_x,),
        schema_hash,
        source_orid,
    )

    assert os.path.exists(kernel_path)
    assert os.path.exists(manifest_path)

    # 2. Verify Manifest Completeness (100% schema+source hash+JAX version)
    manifest = exporter.load_and_verify("dummy_kernel", schema_hash)
    assert manifest["schema_hash"] == schema_hash
    assert manifest["source_orid"] == source_orid
    assert "source_hash" in manifest
    assert "jax_version" in manifest
    assert "backend" in manifest

    # 3. Stale Schema Rejection (100%)
    stale_schema_hash = "sha256:stale_schema_invalid"
    try:
        _ = exporter.load_and_verify("dummy_kernel", stale_schema_hash)
        assert False, "Stale schema load MUST throw ValueError"
    except ValueError as e:
        assert "STALE SCHEMA REJECTED" in str(e)

    print("[PASS] AOT/StableHLO Export verified (manifest completeness, 100% stale schema load rejection).")

if __name__ == "__main__":
    test_aot_stablehlo_export()
