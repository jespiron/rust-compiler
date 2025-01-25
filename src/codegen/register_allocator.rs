//! Register allocator.

use regex::Regex;
use std::collections::{HashMap, HashSet};

/// `Dependency`` represents liveness information of an abstract assembly line.
///
/// For a given assembly line,
///     live_out: variables that are live (i.e. needed for future instructions) at the _end_ of the instruction
///         live_out = UNION of live_in(successors)
///     live_in: variables that are live _before_ the instruction is executed
///         live_in = (live_out MINUS defined_vars) UNION used_vars
///     
///
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
///
#[derive(Debug)]
struct Dependency {
    /// Denotes the temps used on this line
    uses: HashSet<String>,
    /// Denotes the temp or register defined on this line
    defines: Option<String>,
    /// Denotes live-out temps on this line, derivable from uses and defines sets
    live_out: HashSet<String>,
    /// Denotes live-in temps on this line, derivable from live_out, uses, and defines
    live_in: HashSet<String>,
    /// True iff the instruction is a move instruction, needed for register coalescing
    is_move: bool,
    /// Line number within the abstract assembly programming
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

/// Assigns temps using at most K registers
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
    // Chordal Graph Algorithm
    // See https://www.cs.cmu.edu/~15411/lectures/02-regalloc.pdf
    let mut graph = create_interference_graph(dependencies);

    // "don't mess with %rsp"
    static COLOR_TO_REGISTER: [&str; 15] = [
        "%eax", "%edx", "%ebx", "%ecx", "%esi", "%edi", "%ebp", "%r8", "%r9", "%r10", "%r11",
        "%r12", "%r13", "%r14", "%r15",
    ];
    assign_colors(&mut graph, k);

    // Construct output
    let mut assignments = Vec::new();
    let mut spillover = HashSet::new();

    for dependency in dependencies.iter() {
        if let Some(temp) = &dependency.defines {
            // Check if the temp has a valid color assigned
            if let Some(color) = graph.node_colors.get(temp) {
                // If the color is present, try to find the corresponding register
                if *color < k {
                    let register = COLOR_TO_REGISTER[*color];
                    assignments.push(Some(Assignment {
                        temp: temp.clone(),
                        register: register.to_string(),
                    }));
                } else {
                    // Handle case where there is no register for the color
                    spillover.insert(temp.clone());
                    assignments.push(None);
                }
            } else {
                // No color found for the temp, spillover
                spillover.insert(temp.clone());
                assignments.push(None);
            }
        } else {
            assignments.push(None);
        }
    }

    Output {
        assignments,
        spillover,
    }
}

/// Interference graph.
///  Nodes: variables and registers
///  An edge exists between two variables if they should be assigned different registers;
///      that is, they have overlapping live ranges and hold *different values*.
///  The fact that they have *different values* is important! If we have a variable-to-variable
///     move, in which they'll have overlapping live ranges, it's actually beneficial to assign
///     the variables to the same register so that the move becomes redundant.
struct InterferenceGraph {
    /// neighbors[v] = neighbors of v
    neighbors: HashMap<String, HashSet<String>>,
    /// node_colors[v] = numerical color of v
    node_colors: HashMap<String, usize>,
}

fn create_interference_graph(dependencies: &Vec<Dependency>) -> InterferenceGraph {
    // The adjacency list of our interference graph
    let mut neighbors: HashMap<String, HashSet<String>> = HashMap::new();

    // Traverse program *backwards* from the last line
    for dep in dependencies.iter().rev() {
        if let Some(temp) = &dep.defines {
            // Case 1: t <- s_1 OP s_2 instruction (some computation stored in t)
            //         Create an edge between t and any t_i that is live after this line,
            //         where t_i != t.
            let mut criteria = HashSet::from([temp]);

            // Case 2: t <- s instruction (move into t)
            //         Create an edge between t and any t_i that is live after this line,
            //         where t_i != t AND t_i != s.
            if dep.is_move {
                assert!(dep.uses.len() == 1);
                criteria.insert(dep.uses.iter().next().unwrap());
            }

            // For each live-in of successors
            for live_temp in dep.live_out.iter() {
                if !criteria.contains(live_temp) {
                    neighbors
                        .entry(temp.clone())
                        .or_insert_with(HashSet::new)
                        .insert(live_temp.clone());
                    neighbors
                        .entry(live_temp.clone())
                        .or_insert_with(HashSet::new)
                        .insert(temp.clone());
                }
            }

            // Create entry for the defined temp, if zero neighbors
            if !neighbors.contains_key(temp) {
                neighbors.insert(temp.clone(), HashSet::new());
            }
        }
    }

    println!("Neighbors: {:?}", neighbors);

    InterferenceGraph {
        neighbors,
        node_colors: HashMap::new(),
    }
}

fn assign_colors(graph: &mut InterferenceGraph, k: usize) {
    // Pre-color the registers %eax and %edx with 0 and 1 respectively
    assert!(k >= 2);
    if graph.neighbors.contains_key("%eax") {
        graph.node_colors.insert("%eax".to_string(), 0);
    }

    if graph.neighbors.contains_key("%edx") {
        graph.node_colors.insert("%edx".to_string(), 1);
    }

    // Color the rest with greedy approach
    for temp in graph.neighbors.keys() {
        // Skip if already colored, especially for %eax and %edx
        if graph.node_colors.contains_key(temp) {
            continue;
        }

        // Check the colors of neighboring nodes
        let mut used_colors = HashSet::new();
        if let Some(neighbors) = graph.neighbors.get(temp) {
            for neighbor in neighbors {
                if let Some(color) = graph.node_colors.get(neighbor) {
                    used_colors.insert(*color);
                }
            }
        }

        // Find the first color that is not used by neighbors
        // This smells like a Leetcode problem but I don't feel like writing the O(1) space solution
        // Assign the smallest available color
        for color in 0..k {
            if !used_colors.contains(&color) {
                graph.node_colors.insert(temp.clone(), color);
                break;
            }
        }

        // Spillover, no colors available for this temp
        // Designate k as the "spillover" color
        if !graph.node_colors.contains_key(temp) {
            graph.node_colors.insert(temp.clone(), k);
        }
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
        let mut defined_registers: HashMap<String, String> = HashMap::new();

        for (i, dependency) in input.dependencies.iter().enumerate() {
            // Ensure all defined temps are assigned
            if let Some(temp) = &dependency.defines {
                if let Some(assignment) = &output.assignments[i] {
                    assert!(
                        assignment.temp == *temp,
                        "Assignment mismatch at line {}",
                        i
                    );

                    let assigned_register = &assignment.register;

                    // Check for register conflicts
                    for live_temp in &dependency.live_out {
                        if let Some(live_register) = defined_registers.get(live_temp) {
                            if live_register == assigned_register {
                                eprintln!(
                                    "Conflict: Register {} is used by both {} and {} at line {}",
                                    assigned_register, live_temp, temp, i
                                );
                                return false;
                            }
                        }
                    }

                    // Update defined registers
                    defined_registers.insert(temp.clone(), assigned_register.clone());
                } else {
                    // Temp is not assigned a register
                    eprintln!(
                        "Temp {} defined at line {} is not assigned a register",
                        temp, i
                    );
                    return false;
                }
            }

            // Remove temps that are no longer live
            defined_registers.retain(|temp, _| dependency.live_out.contains(temp));

            // Ensure no more than K registers are used
            if defined_registers.len() > input.k {
                eprintln!(
                    "Exceeded register limit ({} registers) at line {}",
                    input.k, i
                );
                return false;
            }
        }

        true
    }

    // Macro for generating tests
    macro_rules! register_allocator_test {
        (
            $test_name:ident,
            $k:expr,
            $dependencies:expr
        ) => {
            #[test]
            fn $test_name() {
                let test_case = TestCase {
                    k: $k,
                    dependencies: $dependencies,
                };

                let output = _allocate_registers(test_case.k, &test_case.dependencies);

                assert!(
                    validate_output(&test_case, &output),
                    "Output failed validation"
                );
            }
        };
    }

    fn compute_liveness(dependencies: &mut Vec<Dependency>) {
        // Initialize `live_out` and `live_in` sets for all lines
        let mut live_out = vec![HashSet::new(); dependencies.len()];
        let mut live_in = vec![HashSet::new(); dependencies.len()];

        let mut has_changed = true;
        while has_changed {
            has_changed = false;

            // Iterate in reverse (backward pass through the assembly lines)
            for i in (0..dependencies.len()).rev() {
                let dep = &dependencies[i];

                // Compute `live_in`: used_vars ∪ (live_out - defined_vars)
                let mut current_live_in = dep.uses.clone();
                for temp in &live_out[i] {
                    if dep.defines.as_ref() != Some(temp) {
                        current_live_in.insert(temp.clone());
                    }
                }

                // Compute `live_out`: union of live_in from all successors
                let mut current_live_out = HashSet::new();
                if i + 1 < dependencies.len() {
                    current_live_out = live_in[i + 1].clone();
                }

                // Check if either `live_in` or `live_out` changed
                if live_in[i] != current_live_in || live_out[i] != current_live_out {
                    has_changed = true;
                    live_in[i] = current_live_in;
                    live_out[i] = current_live_out;
                }
            }
        }

        // Update the dependencies with computed liveness information
        for (i, dep) in dependencies.iter_mut().enumerate() {
            dep.live_in = live_in[i].clone();
            dep.live_out = live_out[i].clone();
        }
    }

    fn parse_dependencies(input: &str) -> Vec<Dependency> {
        let line_regex = Regex::new(r"L(\d+):\s*(\S+)\s*<-\s*(.*)").unwrap();
        let arithmetic_regex = Regex::new(r"(\S+)\s*([+\-*/])\s*(\S+)").unwrap();

        let mut raw_dependencies: Vec<Dependency> = input
            .lines()
            .filter_map(|line| {
                line_regex.captures(line).map(|captures| {
                    let line_number: usize = captures[1].parse().unwrap();
                    let defines = Some(captures[2].to_string());
                    let value = captures[3].trim();

                    let (uses, is_move) =
                        if let Some(arith_captures) = arithmetic_regex.captures(value) {
                            let left = arith_captures[1].to_string();
                            let right = arith_captures[3].to_string();

                            let mut uses = HashSet::new();
                            if !left.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                                uses.insert(left);
                            }
                            if !right.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                                uses.insert(right);
                            }

                            (uses, false)
                        } else {
                            // Simple move or constant assignment
                            let uses = if !value.is_empty() {
                                [value.to_string()].iter().cloned().collect()
                            } else {
                                HashSet::new()
                            };
                            (uses, true)
                        };

                    Dependency {
                        uses: uses.clone(),
                        defines,
                        live_out: HashSet::new(), // Placeholder
                        live_in: HashSet::new(),  // Placeholder
                        is_move,
                        line: line_number,
                    }
                })
            })
            .collect();

        // Compute liveness
        compute_liveness(&mut raw_dependencies);
        raw_dependencies
    }

    // Interference graph:
    //
    //      x1 - x2 - x3 - x4   x5  %eax
    //
    register_allocator_test!(
        simple_linear_interference,
        4,
        parse_dependencies(
            r#"
            L1: x1 <- 1
            L2: x2 <- 1
            L3: x3 <- x2 + x1
            L4: x4 <- x3 + x2
            L5: x5 <- x4 + x3
            L6: %eax <- x5
            "#
        )
    );

    // Interference graph:
    //
    //      a - b - c   %eax
    //       \     /
    //        bb- d
    register_allocator_test!(
        chordal_graph_temp_b_reuse,
        8,
        parse_dependencies(
            r#"
            L1: a <- 0
            L2: b <- 1
            L3: c <- a + b
            L4: d <- b + c
            L5: a <- c + d
            L6: bb <- 7
            L7: d <- a + bb
            L8: %eax <- bb + d
            "#
        )
    );

    // Interference graph:
    //
    //      a - b - c - d   %eax
    //      aa - bb - dd
    //
    register_allocator_test!(
        range_split_with_temp_reuse,
        8,
        parse_dependencies(
            r#"
            L1: a <- 0
            L2: b <- 1
            L3: c <- a + b
            L4: d <- b + c
            L5: aa <- c + d
            L6: bb <- 7
            L7: dd <- aa + bb
            L8: %eax <- bb + dd
            "#
        )
    );

    register_allocator_test!(
        disconnected_graph_allocation,
        8,
        parse_dependencies(
            r#"
            L1: a <- 0
            L2: b <- 1
            L3: c <- a + b
            L4: x <- 2
            L5: y <- 3
            L6: z <- x + y
            L7: %eax <- c + z
            "#
        )
    );

    register_allocator_test!(
        high_pressure_register_allocation,
        5,
        parse_dependencies(
            r#"
            L1: a <- 0
            L2: b <- 1
            L3: c <- a + b
            L4: d <- b + c
            L5: e <- c + d
            L6: f <- d + e
            L7: g <- e + f
            L8: h <- f + g
            L9: %eax <- g + h
            "#
        )
    );

    register_allocator_test!(
        move_coalescing_scenario,
        8,
        parse_dependencies(
            r#"
            L1: a <- 0
            L2: b <- a
            L3: c <- b + 1
            L4: d <- b + c
            L5: e <- c + d
            L6: f <- d + e
            L7: %eax <- f
            "#
        )
    );

    register_allocator_test!(
        spillover_limited_registers,
        5,
        parse_dependencies(
            r#"
            L1: a <- 0
            L2: b <- 1
            L3: c <- a + b
            L4: d <- b + c
            L5: e <- c + d
            L6: f <- d + e
            L7: g <- e + f
            L8: h <- f + g
            L9: %eax <- h
            "#
        )
    );

    register_allocator_test!(
        triangular_interference,
        8,
        parse_dependencies(
            r#"
            L1: a <- 0
            L2: b <- 1
            L3: c <- a + b
            L4: d <- b + c
            L5: e <- a + d
            L6: %eax <- e + c
            "#
        )
    );
}
