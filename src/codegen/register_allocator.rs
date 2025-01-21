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

#[path = "./tests/register_allocator_tests.rs"]
#[cfg(test)]
mod register_allocator_tests;

// Temps and registers are represented by %t[1-9][0-9]*
#[derive(Debug)]
struct Dependency {
    used: HashSet<String>,     // Denotes the temps used on this line
    defined: Option<String>,   // Denotes the temp or register defined on this line
    live_out: HashSet<String>, // Denotes live-out temps on this line, deriable from used and defined sets
    is_move: bool, // True iff the instruction is a move instruction, needed for register coalescing
    line: usize,   // Line number within the abstract asseumbly programming
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

fn allocate_registers(k: usize, dependencies: &Vec<Dependency>) -> Output {
    let assignments = Vec::new();
    Output { assignments }
}
