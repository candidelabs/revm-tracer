import 'src/rust/api/tracer.dart';

class RevmTracer {
  static revmTrace({
    required BigInt chainId,
    required String from,
    required BigInt fromNonce,
    required String to,
    required String data,
    required BigInt gasLimit,
    required BigInt gasPrice,
    required BigInt gasPriorityFee,
    required String latestBlockEnv,
    required String prestateTracerResult,
    required bool isOpStack
  }) => formatAndTraceTransaction(
    chainId: chainId,
    from: from,
    fromNonce: fromNonce,
    to: to,
    data: data,
    gasLimit: gasLimit,
    gasPrice: gasPrice,
    gasPriorityFee: gasPriorityFee,
    latestBlockEnv: latestBlockEnv,
    prestateTracerResult: prestateTracerResult,
    isOpStack: isOpStack,
  );
}