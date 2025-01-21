use super::{allocate_registers, Assignment, Dependency, Output};
use std::collections::HashSet;

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
        if let Some(temp) = &dependency.defined {
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
#[macro_export]
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
            let output = allocate_registers(test_case.k, &test_case.dependencies);

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

// Generate tests from macro
#[cfg(test)]
mod tests {
    use super::*;

    register_allocator_test!(
        test_case_simple,
        8,
        vec![
            Dependency {
                used: HashSet::from(["%t9".to_string(), "%t10".to_string()]),
                defined: Some("%t11".to_string()),
                live_out: HashSet::from(["%t11".to_string()]),
                is_move: false,
                line: 30,
            },
            Dependency {
                used: HashSet::from(["%t11".to_string()]),
                defined: Some("%eax".to_string()),
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
            ]
        }
    );

    register_allocator_test!(
        test_case_spilling,
        1,
        vec![
            Dependency {
                used: HashSet::from(["%t1".to_string()]),
                defined: Some("%t2".to_string()),
                live_out: HashSet::from(["%t2".to_string()]),
                is_move: false,
                line: 10,
            },
            Dependency {
                used: HashSet::from(["%t2".to_string()]),
                defined: Some("%eax".to_string()),
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
            ]
        }
    );
}
