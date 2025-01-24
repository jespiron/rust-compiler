//! Register allocator.
/*
Each input file has the JSON format of the form
```
//target 8
[
...
{
"Uses": [ "%t9", "%t10" ],
"Defines": [ "%t11" ],
"Live_out": [ "%t11" ],
"Move": false,
"Line": 30,
},
{
"Uses": [ "%t11" ],
"Defines": [ "%eax" ],
"Live_out": [],
"Move": true,
"Line": 31,
},
1
...
]
```

The first line of the input file is a directive //target k, meaning that your compiler should use at
most k registers in register allocation to receive points for this input. The rest of the input file is a
JSON list of JSON objects. The ith object in the list represents the liveness information of the ith
abstract assembly line. For example, the two objects above correspond to the abstract assembly
lines below:
%t11 <-- %t9 * %t10
%eax <--%t11

Here is a breakdown of what the fields in each object means:


[
{ "%t1": "%edx" },
{ "%t2": "%edx" },
{ "%t3": "%eax" },
{},
...
]

The output is also a JSON list of JSON objects. The ith object in the list must correspond to the
ith abstract assembly line. There is at most one temp defined on an abstract assembly line, and the
2
corresponding JSON object in your output should map this defined temp to the register allocated
for this temp. If no temp is defined on an abstract assembly line, the corresponding JSON object
should be an empty JSON object, as shown above.

The reason we don’t ask you to output a single mapping from each temp to the register allocated
for that temp is we wanted to be flexible. If there are multiple definitions of the same temp, some
register allocator implementations might map each definition to a different register. That being
said, the reference compiler used to generate the input files implements SSA, so all temps have a
unique definition within the input files we provide you.
*/

use super::x86_encoding::Register;
use std::collections::HashSet;

/// `Dependency`` represents liveness information of an abstract assembly line.
/// For example, the
/// Temps and registers are represented by %t[1-9][0-9]*
#[derive(Debug)]
struct Dependency {
    /// Denotes the temps used on this line
    used: HashSet<String>,
    /// Denotes the temp or register defined on this line
    defined: Option<String>,
    /// Denotes live-out temps on this line, deriable from used and defined sets
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
    assignments: Vec<Option<Assignment>>,
}

/// Private helper. Allocates
fn _allocate_registers(k: usize, dependencies: &Vec<Dependency>) -> Output {
    let assignments = Vec::new();
    Output { assignments }
}

pub fn allocate_registers(dependencies: &Vec<Dependency>) -> Output {
    Output {
        assignments: vec![],
    }
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
