module @jit_dummy_scoring_kernel attributes {mhlo.num_partitions = 1 : i32, mhlo.num_replicas = 1 : i32} {
  func.func public @main(%arg0: tensor<64x32xf32>) -> (tensor<64x32xf32> {jax.result_info = "result"}) {
    %0 = stablehlo.sine %arg0 : tensor<64x32xf32>
    %1 = stablehlo.cosine %arg0 : tensor<64x32xf32>
    %2 = stablehlo.add %0, %1 : tensor<64x32xf32>
    return %2 : tensor<64x32xf32>
  }
}
