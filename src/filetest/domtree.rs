
//! Test command for verifying dominator trees.
//!
//! The `test domtree` test command looks for annotations on instructions like this:
//!
//!     jump ebb3 ; dominates: ebb3
//!
//! This annotation means that the jump instruction is expected to be the immediate dominator of
//! `ebb3`.
//!
//! We verify that the dominator tree annotations are complete and correct.
//!

use std::collections::HashMap;
use std::borrow::{Borrow, Cow};
use cretonne::ir::Function;
use cretonne::ir::entities::AnyEntity;
use cretonne::cfg::ControlFlowGraph;
use cretonne::dominator_tree::DominatorTree;
use cton_reader::TestCommand;
use filetest::subtest::{SubTest, Context, Result};
use utils::match_directive;

struct TestDomtree;

pub fn subtest(parsed: &TestCommand) -> Result<Box<SubTest>> {
    assert_eq!(parsed.command, "domtree");
    if !parsed.options.is_empty() {
        Err(format!("No options allowed on {}", parsed))
    } else {
        Ok(Box::new(TestDomtree))
    }
}

impl SubTest for TestDomtree {
    fn name(&self) -> Cow<str> {
        Cow::from("domtree")
    }

    // Extract our own dominator tree from
    fn run(&self, func: Cow<Function>, context: &Context) -> Result<()> {
        let func = func.borrow();
        let cfg = ControlFlowGraph::new(func);
        let domtree = DominatorTree::new(&cfg);

        // Build an expected domtree from the source annotations.
        let mut expected = HashMap::new();
        for comment in &context.details.comments {
            if let Some(tail) = match_directive(comment.text, "dominates:") {
                let inst = match comment.entity {
                    AnyEntity::Inst(inst) => inst,
                    _ => {
                        return Err(format!("annotation on non-inst {}: {}",
                                           comment.entity,
                                           comment.text))
                    }
                };
                for src_ebb in tail.split_whitespace() {
                    let ebb = match context.details.map.lookup_str(src_ebb) {
                        Some(AnyEntity::Ebb(ebb)) => ebb,
                        _ => return Err(format!("expected EBB: {}", src_ebb)),
                    };

                    // Annotations say that `inst` is the idom of `ebb`.
                    if expected.insert(ebb, inst).is_some() {
                        return Err(format!("multiple dominators for {}", src_ebb));
                    }

                    // Compare to computed domtree.
                    match domtree.idom(ebb) {
                        Some((_, got_inst)) if got_inst != inst => {
                            return Err(format!("mismatching idoms for {}:\n\
                                                want: {}, got: {}",
                                               src_ebb,
                                               inst,
                                               got_inst));
                        }
                        None => {
                            return Err(format!("mismatching idoms for {}:\n\
                                                want: {}, got: unreachable",
                                               src_ebb,
                                               inst));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Now we know that everything in `expected` is consistent with `domtree`.
        // All other EBB's should be either unreachable or the entry block.
        for ebb in func.layout.ebbs().skip(1).filter(|ebb| !expected.contains_key(&ebb)) {
            if let Some((_, got_inst)) = domtree.idom(ebb) {
                return Err(format!("mismatching idoms for renumbered {}:\n\
                                    want: unrechable, got: {}",
                                   ebb,
                                   got_inst));
            }
        }

        Ok(())
    }
}
