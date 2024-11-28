# OpenRPC-TestGen

`openrpc-testgen` is a Rust library designed to simplify the creation and management of test suites and test cases for OpenRPC endpoints. It enables users to extend its functionality by adding custom test suites and test cases while adhering to specific conventions enforced by the build system.

## Table of Contents

1. [Introduction](#introduction)
2. [Assumptions and Conventions](#assumptions-and-conventions)
   - [Directory and File Naming](#directory-and-file-naming)
   - [Test Suite Structure](#test-suite-structure)
   - [Test Case Structure](#test-case-structure)
3. [Auto-Generated Code](#auto-generated-code)
4. [Usage](#usage)

---

## Introduction

This library is designed for testing OpenRPC endpoints using hierarchical test suites and test cases. It relies on a `build.rs` script that automatically generates the required code to register and run tests.

To add your custom test suites and cases, you need to follow certain conventions to ensure they are properly discovered and included in the build process.

---

## Assumptions and Conventions

### Directory and File Naming

1. **Test Suite Directories**:

   - Each test suite directory **must** start with the prefix `suite_`.
     Example:
     ```
     src/suite_example
     ```
   - Each test suite directory **must** include a `mod.rs` file.

2. **Test Case Files**:
   - Each test case file **must** start with the prefix `test_`.
     Example:
     ```
     src/suite_example/test_my_function.rs
     ```

---

### Test Suite Structure

1. **Test Suite Definition**:

   - Define a `TestSuite...` struct in the `mod.rs` file of the suite directory, where `...` is a unique name.
   - Implement the `SetupableTrait` for the struct to handle any setup logic.

   Example:

   ```rust
   pub struct TestSuiteExample;

   impl SetupableTrait for TestSuiteExample {
       type Input = SetupInput;

       async fn setup(input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
           // Your setup logic here
           Ok(Self)
       }
   }
   ```

2. **Including Generated Code**:
   - At the end of the `mod.rs` file, include the auto-generated code for the suite:
     ```rust
     include!(concat!(env!("OUT_DIR"), "/generated_tests_suite_example.rs"));
     ```

---

### Test Case Structure

1. **Test Case Definition**:

   - Each test case file must define a `TestCase` struct.
   - Implement the `RunnableTrait` for the `TestCase` struct to define the logic for running the test.

   Example:

   ```rust
   pub struct TestCase;

   impl RunnableTrait for TestCase {
       type Input = TestSuiteExample;

       async fn run(input: &Self::Input) -> Result<(), OpenRpcTestGenError> {
           // Your test logic here
           Ok(())
       }
   }
   ```

2. **Execution Logic**:
   - Test cases are executed as part of their parent suite's `run` method. Ensure the `RunnableTrait` is implemented correctly.

---

### Adding a New Suite Feature

1. **Modify `lib.rs` by adding:**
   ```rust
   #[cfg(feature = "new_suite")]
   pub mod new_suite;
   ```
2. **Update `cargo.toml`**

   ```rust
   [features]
   new_suite = []
   ```

## Auto-Generated Code

The `build.rs` script automates the discovery and registration of test suites and cases. It processes directories and generates code to:

- Register all test suites starting with `suite_`.
- Discover and include test cases prefixed with `test_`.
- Generate the `run` implementation for each suite, chaining its setup logic with the execution of its test cases and nested suites.

The script monitors changes in the `src/` directory using:

```rust
   println!("cargo:rerun-if-changed=src");
```

---

## Usage

1. **Create a new test suite**:

   - Add a directory under `src/` with a `suite_` prefix (e.g., `suite_example`).
   - Add a `mod.rs` file to define the test suite:
     - Define a `TestSuite...` struct and implement the `SetupableTrait` for it.
     - Include the auto-generated code at the end of the `mod.rs` file using:
       ```rust
       include!(concat!(env!("OUT_DIR"), "/generated_tests_suite_example.rs"));
       ```

2. **Add test cases**:

   - Inside the suite directory, create files prefixed with `test_` (e.g., `test_case_one.rs`).
   - Each file must define a `TestCase` struct and implement the `RunnableTrait` to provide the logic for the test case.

3. **Rebuild the project**:

   - Run `cargo build` to ensure the generated test code is updated.

4. **Run test suites via a binary**:

   - Create a binary that imports the `openrpc-testgen` crate and runs the desired test suite.
   - Example:

     ```rust
     use args::Args;
     use clap::Parser;
     use openrpc_testgen::{
         suite_openrpc::{SetupInput, TestSuiteOpenRpc},
         RunnableTrait,
     };
     pub mod args;

     #[tokio::main]
     async fn main() {
         tracing_subscriber::fmt()
             .with_max_level(tracing::Level::INFO)
             .init();
         let args = Args::parse();

         let suite_openrpc_input = SetupInput {
             urls: args.urls,
             paymaster_account_address: args.paymaster_account_address,
             paymaster_private_key: args.paymaster_private_key,
             udc_address: args.udc_address,
             executable_account_sierra_path: args.executable_account_sierra_path,
             executable_account_casm_path: args.executable_account_casm_path,
             account_class_hash: args.account_class_hash,
         };

         let _ = TestSuiteOpenRpc::run(&suite_openrpc_input).await;
     }
     ```

5. **Run the binary**:

   - Execute the binary with the required arguments to run the test suite:
     ```bash
     cargo run -- <arguments>
     ```

6. **Iterate and develop**:
   - Modify test suites and cases as needed, and rerun the binary to test the updated logic.

---

## Example Directory Structure

Here’s an example structure:

```
src/
├── lib.rs
├── suite_example
│   ├── mod.rs
│   ├── test_case_one.rs
│   ├── test_case_two.rs
│   └── suite_nested
│       ├── mod.rs
│       └── test_nested_case.rs
```

- `suite_example/mod.rs` defines `TestSuiteExample`.
- `test_case_one.rs` and `test_case_two.rs` define `TestCase` structs.
- `suite_nested` is a nested suite containing its own `TestSuite...` and test cases.

---

## Notes

- Ensure that naming conventions are strictly followed, as the `build.rs` script relies on these patterns.
- Any misnamed or incomplete module will result in compilation errors.
- The `build.rs` script automatically generates all the necessary glue code for running suites and cases.

Feel free to extend the library by following the outlined structure and conventions!
