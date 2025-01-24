//! Register allocator.

use std::collections::HashSet;

/// `Dependency`` represents liveness information of an abstract assembly line.
/// For example, the abstract assembly lines
///     %t11 <-- %t9 * %t10
///     %eax <--%t11
/// would correspond to the following:
///     Dependency {
///         uses: [ "%t9", "%t10" ],
///         defines: [ "%t11" ],
///         live_out: [ "%t11" ],
///         move: false,
///         line: 30,
///     },
///     Dependency {
///         uses: [ "%t11" ],
///         defines: [ "%eax" ],
///         live_out: [],
///         is_move: true,
///         line: 31,
///     }
#[derive(Debug)]
struct Dependency {
    /// Denotes the temps used on this line
    uses: HashSet<String>,
    /// Denotes the temp or register defined on this line
    defines: Option<String>,
    /// Denotes live-out temps on this line, derivable from used and defined sets
    live_out: HashSet<String>,
    /// True iff the instruction is a move instruction, needed for register coalescing
    is_move: bool,
    /// Line number within the abstract asseumbly programming
    line: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct Assignment {
    temp: String,
    register: String,
}

#[derive(Debug, PartialEq)]
struct Output {
    /// Register assignment for the temp that was defined on the line, if any
    assignments: Vec<Option<Assignment>>,
    /// Temps that were not assigned a register
    spillover: HashSet<String>,
}

/// Private helper. Assigns temps using at most K registers
/// Outputs one assignment per assembly line, or None if no temp is defined on that line.
/// assignments: [
///     Some({ temp: "%t1", register: "%edx" }),
///     Some({ temp: "%t2", register: "%edx" }),
///     Some({ temp: "%t3", register: "%eax" }),
///     None,
///     ...
/// ]
///
/// If it does not find an assignment that uses at most K registers, it will assign as many
/// temps as possible to K registers. The remaining temps will be spilled over to the stack.
/// Spillover temps are collected in the spillover field.
///
fn _allocate_registers(k: usize, dependencies: &Vec<Dependency>) -> Output {
    let assignments = Vec::new();
    let spillover = HashSet::new();
    Output {
        assignments,
        spillover,
    }
}

/// Assigns temps to the 15 general-purpose registers.
/// Precondition: `dependencies` already hardcodes usage of the %eax and %edx registers
///  for assembly lines that use the `ret` and `idiv` instructions. To explain, %eax and $edx
/// are special for these instructions, as %eax holds the return value, while %edx
/// holds the remainder when division is done.
pub fn allocate_registers(dependencies: &Vec<Dependency>) -> Output {
    // First, look for an assignment that uses all 15 general-purpose registers
    let mut output = _allocate_registers(15, dependencies);

    // If spillover exists, then we *reserve one* register for moving temps to and from the stack.
    // Hence, we look for an assignment that uses 14 general-purpose registers.
    if !output.spillover.is_empty() {
        output = _allocate_registers(14, dependencies);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct TestCase {
        k: usize,
        dependencies: Vec<Dependency>,
    }

    /*
    Correctness: The verifier checks three conditions:
    1. That temps defined on each line in the input file are allocated to some register in the output.
    2. That no conflicts occur, i.e., temps that interfere should not assigned to the same register.
    3. No more than K registers are used
    */
    fn validate_output(input: &TestCase, output: &Output) -> bool {
        let mut defined_registers: HashSet<String> = HashSet::new();

        for (i, dependency) in input.dependencies.iter().enumerate() {
            // Make sure all defined temps are assigned
            if let Some(temp) = &dependency.defines {
                if let Some(assignment) = &output.assignments[i] {
                    assert!(&assignment.temp == temp);

                    // Register conflict detected
                    let assigned_register = &assignment.register;
                    if defined_registers.contains(assigned_register) {
                        return false;
                    }

                    defined_registers.insert(assigned_register.clone());
                } else {
                    // Temp not assigned to any register
                    return false;
                }
            }

            // Ensure no more than K registers are used
            if defined_registers.len() > input.k {
                return false;
            }

            // Update live-out info
            // TODO: read slides on live-out
        }

        true
    }

    // Macro for generating tests
    macro_rules! register_allocator_test {
        (
            $test_name:ident,
            $k:expr,
            $dependencies:expr,
            $expected_output:expr
        ) => {
            #[test]
            fn $test_name() {
                let test_case = TestCase {
                    k: $k,
                    dependencies: $dependencies,
                };

                let expected_output = $expected_output;
                let output = _allocate_registers(test_case.k, &test_case.dependencies);

                assert!(
                    validate_output(&test_case, &output),
                    "Output failed validation"
                );

                assert_eq!(
                    output, expected_output,
                    "Output did not match expected output"
                );
            }
        };
    }

    register_allocator_test!(
        test_case_simple,
        8,
        vec![
            Dependency {
                uses: HashSet::from(["%t9".to_string(), "%t10".to_string()]),
                defines: Some("%t11".to_string()),
                live_out: HashSet::from(["%t11".to_string()]),
                is_move: false,
                line: 30,
            },
            Dependency {
                uses: HashSet::from(["%t11".to_string()]),
                defines: Some("%eax".to_string()),
                live_out: HashSet::new(),
                is_move: true,
                line: 31,
            },
        ],
        Output {
            assignments: vec![
                Some(Assignment {
                    temp: "%t11".to_string(),
                    register: "r0".to_string()
                }),
                Some(Assignment {
                    temp: "%eax".to_string(),
                    register: "r1".to_string()
                }),
            ],
            spillover: HashSet::from([])
        }
    );

    register_allocator_test!(
        test_case_spilling,
        1,
        vec![
            Dependency {
                uses: HashSet::from(["%t1".to_string()]),
                defines: Some("%t2".to_string()),
                live_out: HashSet::from(["%t2".to_string()]),
                is_move: false,
                line: 10,
            },
            Dependency {
                uses: HashSet::from(["%t2".to_string()]),
                defines: Some("%eax".to_string()),
                live_out: HashSet::new(),
                is_move: true,
                line: 11,
            },
        ],
        Output {
            assignments: vec![
                Some(Assignment {
                    temp: "%t2".to_string(),
                    register: "r0".to_string()
                }),
                Some(Assignment {
                    temp: "%eax".to_string(),
                    register: "r0".to_string()
                }),
            ],
            spillover: HashSet::from([])
        }
    );
}
