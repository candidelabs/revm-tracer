# REVM Tracer

A high-performance Flutter plugin that provides EVM transaction tracing capabilities using Rust's REVM (Rust Ethereum Virtual Machine) implementation. This library bridges Flutter/Dart with Rust to deliver fast, accurate transaction execution traces for both Ethereum and Optimism (OP Stack) chains.

## Features

- **EVM Transaction Tracing**: Execute and trace Ethereum transactions with detailed call information
- **Optimism Support**: Full support for OP Stack chains with optimized tracing
- **High Performance**: Leverages Rust's REVM for near-native execution speeds
- **Cross-Platform**: Works on Android, iOS, Linux, macOS, and Windows
- **Detailed Call Traces**: Captures complete call frames, state changes, and execution results
- **Type-Safe API**: Generated Flutter/Dart bindings via flutter_rust_bridge

## Installation

Add this to your `pubspec.yaml`:

```yaml
dependencies:
  revm_tracer: ^0.0.1
```

Then run:

```bash
flutter pub get
```

## Usage

### Initialize the Library

Before using the tracer, initialize the Rust library:

```dart
import 'package:revm_tracer/revm_tracer.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(MyApp());
}
```

### Trace a Transaction

```dart
import 'package:revm_tracer/revm_tracer.dart';

// Trace an Ethereum transaction
String traceResult = RevmTracer.revmTrace(
  chainId: BigInt.from(1), // Ethereum mainnet
  from: '0x...', // Sender address
  fromNonce: BigInt.from(1),
  to: '0x...', // Recipient address
  data: '0x...', // Transaction calldata
  gasLimit: BigInt.from(21000),
  gasPrice: BigInt.from(20000000000),
  gasPriorityFee: BigInt.from(1000000000),
  latestBlockEnv: jsonEncode({
    'number': 12345678,
    'timestamp': 1234567890,
    'gasLimit': 30000000,
    'baseFee': '1000000000',
    'difficulty': '0',
    'prevrandao': '0x0000000000000000000000000000000000000000000000000000000000000000',
    'coinbase': '0x0000000000000000000000000000000000000000'
  }),
  prestateTracerResult: jsonEncode({
    '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb': {
      'balance': '1000000000000000000',
      'nonce': 1,
      'code': '0x',
      'storage': {}
    }
  }),
  isOpStack: false, // Set to true for Optimism chains
);

// Parse the JSON result
Map<String, dynamic> result = jsonDecode(traceResult);

if (result.containsKey('error')) {
  print('Error: ${result['message']}');
} else {
  print('Execution result: ${result['executionResult']}');
  print('State diff: ${result['stateDiff']}');
  print('Calls: ${result['calls']}');
}
```

### Trace an Optimism Transaction

For Optimism (OP Stack) chains, simply set `isOpStack: true`:

```dart
String traceResult = RevmTracer.revmTrace(
  chainId: BigInt.from(10), // OP Mainnet
  // ... other parameters
  isOpStack: true, // Enable Optimism-specific tracing
);
```

## API Reference

### `RevmTracer.revmTrace()`

Traces an EVM transaction execution and returns detailed results.

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `chainId` | `BigInt` | Chain ID (e.g., 1 for Ethereum, 10 for OP Mainnet) |
| `from` | `String` | Sender address (hex string with 0x prefix) |
| `fromNonce` | `BigInt` | Sender's nonce |
| `to` | `String` | Recipient address (hex string with 0x prefix) |
| `data` | `String` | Transaction calldata (hex string with 0x prefix) |
| `gasLimit` | `BigInt` | Maximum gas allowed for execution |
| `gasPrice` | `BigInt` | Gas price in wei |
| `gasPriorityFee` | `BigInt` | Priority fee in wei (EIP-1559) |
| `latestBlockEnv` | `String` | Block environment as JSON string |
| `prestateTracerResult` | `String` | Account prestate as JSON string |
| `isOpStack` | `bool` | Use Optimism tracer (true) or Ethereum tracer (false) |

#### Returns

A JSON string containing:

**On Success:**
```json
{
  "executionResult": {
    "success": true,
    "gasUsed": 21000,
    "output": "0x..."
  },
  "stateDiff": {
    "0xAddress": {
      "balance": "...",
      "nonce": 1,
      "code": "0x...",
      "storage": {}
    }
  },
  "calls": {
    "type": "CALL",
    "from": "0x...",
    "to": "0x...",
    "value": "0",
    "gas": 21000,
    "gasUsed": 21000,
    "input": "0x...",
    "output": "0x...",
    "calls": []
  }
}
```

**On Error:**
```json
{
  "error": true,
  "message": "Error description",
  "type": "ErrorType"
}
```

## Block Environment Format

The `latestBlockEnv` parameter expects a JSON string with the following structure:

```json
{
  "number": 12345678,
  "timestamp": 1234567890,
  "gasLimit": 30000000,
  "baseFee": "1000000000",
  "difficulty": "0",
  "prevrandao": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "coinbase": "0x0000000000000000000000000000000000000000"
}
```

## Prestate Format

The `prestateTracerResult` parameter expects a JSON string mapping addresses to their account states:

```json
{
  "0xAddress1": {
    "balance": "1000000000000000000",
    "nonce": 42,
    "code": "0x6060604052...",
    "storage": {
      "0x0000000000000000000000000000000000000000000000000000000000000000": "0x0000000000000000000000000000000000000000000000000000000000000001"
    }
  },
  "0xAddress2": {
    "balance": "5000000000000000000",
    "nonce": 0,
    "code": "0x",
    "storage": {}
  }
}
```

## Requirements

- Flutter SDK: >=3.3.0
- Dart SDK: >=3.4.0 <4.0.0
- Rust toolchain (for building from source)

## Platform Support

| Platform | Supported |
|----------|-----------|
| Android  | ✅ |
| iOS      | ✅ |
| Linux    | ✅ |
| macOS    | ✅ |
| Windows  | ✅ |

## Architecture

This library uses [flutter_rust_bridge](https://pub.dev/packages/flutter_rust_bridge) to bridge Flutter/Dart with Rust. The core tracing logic is implemented in Rust using:

- **REVM** (v29.0.0): High-performance Ethereum Virtual Machine implementation
- **op-revm** (v10.1.0): Optimism-specific EVM extensions for OP Stack chains

## Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd revm_tracer

# Get Flutter dependencies
fvm flutter pub get

# Execute necessary cargo commands and generate bindings (refer to https://github.com/fzyzcjy/flutter_rust_bridge)
flutter_rust_bridge_codegen generate

# Build for your platform
fvm flutter build <platform>
```

## Use Cases

- **Transaction Simulation**: Preview transaction execution before submitting
- **Gas Estimation**: Accurate gas usage calculation with detailed traces
- **Debugging**: Deep inspection of contract calls and state changes
- **Analytics**: Analyze transaction behavior and contract interactions
- **Safe Wallet Verification**: Verify Safe multisig transactions before execution

## License

See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Acknowledgments

- Built with [REVM](https://github.com/bluealloy/revm) - Rust Ethereum Virtual Machine
- Uses [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) for Rust-Flutter interop
