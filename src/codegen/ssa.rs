use crate::codegen::context::{AbstractAssemblyInstruction, AsmLabel, Context};
use std::collections::HashMap;

pub struct SSABuilder {
    cfg: ControlFlowGraph,
    /// Track current block for SSABuilder
    current_block: BasicBlock,
}

pub struct ControlFlowGraph {
    blocks: HashMap<BasicBlockId, BasicBlock>,
    edges: HashMap<BasicBlockId, (Option<BasicBlockId>, Option<BasicBlockId>)>,
}

#[derive(Debug)]
pub struct BasicBlock {
    id: BasicBlockId,
    label: Option<AsmLabel>,
    instructions: Vec<AbstractAssemblyInstruction>,
}

#[derive(Debug)]
pub struct BasicBlockId(usize);

impl SSABuilder {
    pub fn new() -> Self {}

    pub fn convert_to_ssa(context: &Context) -> Context {
        // 1. Build CFG from context.instructions
        // 2. Compute dominance frontiers
        // 3. Insert phi nodes at dominance frontiers
        // 4. Rename variables
    }

    fn compute_dominance_frontiers(&mut self) {}

    fn insert_phi_nodes(&mut self, context: &mut Context) {}

    fn rename_variables(&mut self, context: &mut Context) {}
}
