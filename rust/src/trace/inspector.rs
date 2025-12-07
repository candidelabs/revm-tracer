use revm::{
    context::ContextTr,
    interpreter::{CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter, InterpreterTypes},
};
use revm::Inspector;
use revm::primitives::{Address, U256, Bytes, Log, B256};
use serde::{Deserialize, Serialize};

// Constants for repeated strings
const ERROR_EXECUTION_REVERTED: &str = "execution reverted";
const HEX_PREFIX: &str = "0x";

/// Represents a log entry emitted during contract execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
}

impl From<Log> for LogEntry {
    fn from(log: Log) -> Self {
        LogEntry {
            address: log.address,
            topics: log.data.topics().to_vec(),
            data: log.data.data.clone(),
        }
    }
}

/// Represents a single call or contract creation in the execution trace.
/// This structure captures all relevant information about a call including
/// inputs, outputs, gas usage, logs, and any subcalls made during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallFrame {
    #[serde(rename = "type")]
    pub call_type: String,
    pub from: Address,
    pub to: Option<Address>,
    #[serde(with = "hex_u256")]
    pub value: U256,
    pub gas: U256,
    pub gas_used: U256,
    pub input: Bytes,
    pub output: Option<Bytes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revert_reason: Option<String>,
    pub logs: Vec<LogEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub calls: Vec<CallFrame>,
}

/// Inspector that traces all calls and contract creations during EVM execution.
/// Maintains a stack of call frames to properly track nested calls.
#[derive(Debug, Default)]
pub struct CallTracer {
    call_stack: Vec<CallFrame>,
}

impl CallTracer {
    /// Creates a new CallTracer instance.
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
        }
    }

    /// Consumes the tracer and returns the root call frame, if any.
    pub fn into_result(mut self) -> Option<CallFrame> {
        self.call_stack.pop()
    }

    /// Converts a call scheme byte to its string representation.
    fn call_type_from_scheme(scheme: u8) -> &'static str {
        match scheme {
            0 => "CALL",
            1 => "CALLCODE",
            2 => "DELEGATECALL",
            3 => "STATICCALL",
            _ => "UNKNOWN",
        }
    }

    /// Common logic for finalizing a frame after execution completes.
    /// Updates gas usage, sets output/error info, and adds to parent frame or root.
    fn finalize_frame(
        &mut self,
        gas_spent: u64,
        is_success: bool,
        output: Bytes,
        created_address: Option<Address>,
    ) {
        if let Some(mut frame) = self.call_stack.pop() {
            frame.gas_used = U256::from(gas_spent);

            if is_success {
                // For contract creation, set the created address as output
                if let Some(address) = created_address {
                    frame.to = Some(address);
                    frame.output = Some(Bytes::from(address.into_array()));
                } else {
                    frame.output = Some(output);
                }
            } else {
                frame.error = Some(ERROR_EXECUTION_REVERTED.to_string());
                if !output.is_empty() {
                    frame.revert_reason = Some(format!("{}{}", HEX_PREFIX, hex::encode(&output)));
                }
            }

            // Add this frame as a subcall to the parent frame, or push it back if it's the root
            if let Some(parent) = self.call_stack.last_mut() {
                parent.calls.push(frame);
            } else {
                self.call_stack.push(frame);
            }
        }
    }
}

impl<CTX: ContextTr, INTR: InterpreterTypes> Inspector<CTX, INTR> for CallTracer {
    fn initialize_interp(&mut self, _interp: &mut Interpreter<INTR>, _context: &mut CTX) {}

    fn call(
        &mut self,
        context: &mut CTX,
        inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
        let call_type = Self::call_type_from_scheme(inputs.scheme as u8).to_string();
        // Get transfer value, defaulting to zero if not available
        let value = inputs.transfer_value().unwrap_or(U256::ZERO);
        let mut from = inputs.caller;
        let mut to = Some(inputs.target_address);
        if call_type == "DELEGATECALL" {
            from = inputs.target_address;
            to = Some(inputs.bytecode_address);
        }

        let frame = CallFrame {
            call_type,
            from,
            to,
            value,
            gas: U256::from(inputs.gas_limit),
            gas_used: U256::ZERO, // Will be updated in call_end
            input: inputs.input.bytes(context),
            output: None,
            error: None,
            revert_reason: None,
            logs: Vec::new(),
            calls: Vec::new(),
        };

        self.call_stack.push(frame);
        None
    }

    fn call_end(
        &mut self,
        _context: &mut CTX,
        _inputs: &CallInputs,
        outcome: &mut CallOutcome,
    ) {
        self.finalize_frame(
            outcome.result.gas.spent(),
            outcome.result.is_ok(),
            outcome.result.output.clone(),
            None,
        );
    }

    fn create(
        &mut self,
        _context: &mut CTX,
        inputs: &mut CreateInputs,
    ) -> Option<CreateOutcome> {
        let call_type = if inputs.scheme == revm::interpreter::CreateScheme::Create {
            "CREATE"
        } else {
            "CREATE2"
        }.to_string();

        let frame = CallFrame {
            call_type,
            from: inputs.caller,
            to: None,
            value: inputs.value,
            gas: U256::from(inputs.gas_limit),
            gas_used: U256::ZERO,
            input: inputs.init_code.clone(),
            output: None,
            error: None,
            revert_reason: None,
            logs: Vec::new(),
            calls: Vec::new(),
        };

        self.call_stack.push(frame);
        None
    }

    fn create_end(
        &mut self,
        _context: &mut CTX,
        _inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        self.finalize_frame(
            outcome.result.gas.spent(),
            outcome.result.is_ok(),
            outcome.result.output.clone(),
            outcome.address,
        );
    }

    fn step(&mut self, _interp: &mut Interpreter<INTR>, _context: &mut CTX) {}

    fn step_end(&mut self, _interp: &mut Interpreter<INTR>, _context: &mut CTX) {}

    fn log(&mut self, _interp: &mut Interpreter<INTR>, _context: &mut CTX, log: Log) {
        // Add the log to the current frame (top of the stack)
        if let Some(frame) = self.call_stack.last_mut() {
            frame.logs.push(LogEntry::from(log));
        }
    }
}

// Custom serialization for U256 to hex string
mod hex_u256 {
    use super::*;
    use serde::{Serializer, Deserializer};

    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{:x}", value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_start_matches("0x");
        U256::from_str_radix(s, 16).map_err(serde::de::Error::custom)
    }
}