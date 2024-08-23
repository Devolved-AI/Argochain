//! Elaboration phase: lowers EGraph back to sequences of operations
//! in CFG nodes.

use super::cost::{pure_op_cost, Cost};
use super::domtree::DomTreeWithChildren;
use super::Stats;
use crate::dominator_tree::DominatorTree;
use crate::fx::FxHashSet;
use crate::ir::{Block, Function, Inst, Value, ValueDef};
use crate::loop_analysis::{Loop, LoopAnalysis, LoopLevel};
use crate::scoped_hash_map::ScopedHashMap;
use crate::trace;
use crate::unionfind::UnionFind;
use alloc::vec::Vec;
use cranelift_entity::{packed_option::ReservedValue, SecondaryMap};
use smallvec::{smallvec, SmallVec};

pub(crate) struct Elaborator<'a> {
    func: &'a mut Function,
    domtree: &'a DominatorTree,
    domtree_children: &'a DomTreeWithChildren,
    loop_analysis: &'a LoopAnalysis,
    eclasses: &'a mut UnionFind<Value>,
    /// Map from Value that is produced by a pure Inst (and was thus
    /// not in the side-effecting skeleton) to the value produced by
    /// an elaborated inst (placed in the layout) to whose results we
    /// refer in the final code.
    ///
    /// The first time we use some result of an instruction during
    /// elaboration, we can place it and insert an identity map (inst
    /// results to that same inst's results) in this scoped
    /// map. Within that block and its dom-tree children, that mapping
    /// is visible and we can continue to use it. This allows us to
    /// avoid cloning the instruction. However, if we pop that scope
    /// and use it somewhere else as well, we will need to
    /// duplicate. We detect this case by checking, when a value that
    /// we want is not present in this map, whether the producing inst
    /// is already placed in the Layout. If so, we duplicate, and
    /// insert non-identity mappings from the original inst's results
    /// to the cloned inst's results.
    value_to_elaborated_value: ScopedHashMap<Value, ElaboratedValue>,
    /// Map from Value to the best (lowest-cost) Value in its eclass
    /// (tree of union value-nodes).
    value_to_best_value: SecondaryMap<Value, (Cost, Value)>,
    /// Stack of blocks and loops in current elaboration path.
    loop_stack: SmallVec<[LoopStackEntry; 8]>,
    /// The current block into which we are elaborating.
    cur_block: Block,
    /// Values that opt rules have indicated should be rematerialized
    /// in every block they are used (e.g., immediates or other
    /// "cheap-to-compute" ops).
    remat_values: &'a FxHashSet<Value>,
    /// Explicitly-unrolled value elaboration stack.
    elab_stack: Vec<ElabStackEntry>,
    /// Results from the elab stack.
    elab_result_stack: Vec<ElaboratedValue>,
    /// Explicitly-unrolled block elaboration stack.
    block_stack: Vec<BlockStackEntry>,
    /// Stats for various events during egraph processing, to help
    /// with optimization of this infrastructure.
    stats: &'a mut Stats,
}

#[derive(Clone, Copy, Debug)]
struct ElaboratedValue {
    in_block: Block,
    value: Value,
}

#[derive(Clone, Debug)]
struct LoopStackEntry {
    /// The loop identifier.
    lp: Loop,
    /// The hoist point: a block that immediately dominates this
    /// loop. May not be an immediate predecessor, but will be a valid
    /// point to place all loop-invariant ops: they must depend only
    /// on inputs that dominate the loop, so are available at (the end
    /// of) this block.
    hoist_block: Block,
    /// The depth in the scope map.
    scope_depth: u32,
}

#[derive(Clone, Debug)]
enum ElabStackEntry {
    /// Next action is to resolve this value into an elaborated inst
    /// (placed into the layout) that produces the value, and
    /// recursively elaborate the insts that produce its args.
    ///
    /// Any inserted ops should be inserted before `before`, which is
    /// the instruction demanding this value.
    Start { value: Value, before: Inst },
    /// Args have been pushed; waiting for results.
    PendingInst {
        inst: Inst,
        result_idx: usize,
        num_args: usize,
        remat: bool,
        before: Inst,
    },
}

#[derive(Clone, Debug)]
enum BlockStackEntry {
    Elaborate { block: Block, idom: Option<Block> },
    Pop,
}

impl<'a> Elaborator<'a> {
    pub(crate) fn new(
        func: &'a mut Function,
        domtree: &'a DominatorTree,
        domtree_children: &'a DomTreeWithChildren,
        loop_analysis: &'a LoopAnalysis,
        remat_values: &'a FxHashSet<Value>,
        eclasses: &'a mut UnionFind<Value>,
        stats: &'a mut Stats,
    ) -> Self {
        let num_values = func.dfg.num_values();
        let mut value_to_best_value =
            SecondaryMap::with_default((Cost::infinity(), Value::reserved_value()));
        value_to_best_value.resize(num_values);
        Self {
            func,
            domtree,
            domtree_children,
            loop_analysis,
            eclasses,
            value_to_elaborated_value: ScopedHashMap::with_capacity(num_values),
            value_to_best_value,
            loop_stack: smallvec![],
            cur_block: Block::reserved_value(),
            remat_values,
            elab_stack: vec![],
            elab_result_stack: vec![],
            block_stack: vec![],
            stats,
        }
    }

    fn start_block(&mut self, idom: Option<Block>, block: Block) {
        trace!(
            "start_block: block {:?} with idom {:?} at loop depth {:?} scope depth {}",
            block,
            idom,
            self.loop_stack.len(),
            self.value_to_elaborated_value.depth()
        );

        // Pop any loop levels we're no longer in.
        while let Some(inner_loop) = self.loop_stack.last() {
            if self.loop_analysis.is_in_loop(block, inner_loop.lp) {
                break;
            }
            self.loop_stack.pop();
        }

        // Note that if the *entry* block is a loop header, we will
        // not make note of the loop here because it will not have an
        // immediate dominator. We must disallow this case because we
        // will skip adding the `LoopStackEntry` here but our
        // `LoopAnalysis` will otherwise still make note of this loop
        // and loop depths will not match.
        if let Some(idom) = idom {
            if let Some(lp) = self.loop_analysis.is_loop_header(block) {
                self.loop_stack.push(LoopStackEntry {
                    lp,
                    // Any code hoisted out of this loop will have code
                    // placed in `idom`, and will have def mappings
                    // inserted in to the scoped hashmap at that block's
                    // level.
                    hoist_block: idom,
                    scope_depth: (self.value_to_elaborated_value.depth() - 1) as u32,
                });
                trace!(
                    " -> loop header, pushing; depth now {}",
                    self.loop_stack.len()
                );
            }
        } else {
            debug_assert!(
                self.loop_analysis.is_loop_header(block).is_none(),
                "Entry block (domtree root) cannot be a loop header!"
            );
        }

        trace!("block {}: loop stack is {:?}", block, self.loop_stack);

        self.cur_block = block;
    }

    fn compute_best_values(&mut self) {
        let best = &mut self.value_to_best_value;
        for (value, def) in self.func.dfg.values_and_defs() {
            trace!("computing best for value {:?} def {:?}", value, def);
            match def {
                ValueDef::Union(x, y) => {
                    // Pick the best of the two options based on
                    // min-cost. This works because each element of `best`
                    // is a `(cost, value)` tuple; `cost` comes first so
                    // the natural comparison works based on cost, and
                    // breaks ties based on value number.
                    trace!(" -> best of {:?} and {:?}", best[x], best[y]);
                    best[value] = std::cmp::min(best[x], best[y]);
                    trace!(" -> {:?}", best[value]);
                }
                ValueDef::Param(_, _) => {
                    best[value] = (Cost::zero(), value);
                }
                // If the Inst is inserted into the layout (which is,
                // at this point, only the side-effecting skeleton),
                // then it must be computed and thus we give it zero
                // cost.
                ValueDef::Result(inst, _) if self.func.layout.inst_block(inst).is_some() => {
                    best[value] = (Cost::zero(), value);
                }
                ValueDef::Result(inst, _) => {
                    trace!(" -> value {}: result, computing cost", value);
                    let inst_data = &self.func.dfg.insts[inst];
                    let loop_level = self
                        .func
                        .layout
                        .inst_block(inst)
                        .map(|block| self.loop_analysis.loop_level(block))
                        .unwrap_or(LoopLevel::root());
                    // N.B.: at this point we know that the opcode is
                    // pure, so `pure_op_cost`'s precondition is
                    // satisfied.
                    let cost = self.func.dfg.inst_values(inst).fold(
                        pure_op_cost(inst_data.opcode()).at_level(loop_level.level()),
                        |cost, value| cost + best[value].0,
                    );
                    best[value] = (cost, value);
                }
            };
            debug_assert_ne!(best[value].0, Cost::infinity());
            debug_assert_ne!(best[value].1, Value::reserved_value());
            trace!("best for eclass {:?}: {:?}", value, best[value]);
        }
    }

    /// Elaborate use of an eclass, inserting any needed new
    /// instructions before the given inst `before`. Should only be
    /// given values corresponding to results of instructions or
    /// blockparams.
    fn elaborate_eclass_use(&mut self, value: Value, before: Inst) -> ElaboratedValue {
        debug_assert_ne!(value, Value::reserved_value());

        // Kick off the process by requesting this result
        // value.
        self.elab_stack
            .push(ElabStackEntry::Start { value, before });

        // Now run the explicit-stack recursion until we reach
        // the root.
        self.process_elab_stack();
        debug_assert_eq!(self.elab_result_stack.len(), 1);
        self.elab_result_stack.pop().unwrap()
    }

    fn process_elab_stack(&mut self) {
        while let Some(entry) = self.elab_stack.last() {
            match entry {
                &ElabStackEntry::Start { value, before } => {
                    // We always replace the Start entry, so pop it now.
                    self.elab_stack.pop();

                    debug_assert_ne!(value, Value::reserved_value());
                    let value = self.func.dfg.resolve_aliases(value);

                    self.stats.elaborate_visit_node += 1;
                    let canonical_value = self.eclasses.find_and_update(value);
                    debug_assert_ne!(canonical_value, Value::reserved_value());
                    trace!(
                        "elaborate: value {} canonical {} before {}",
                        value,
                        canonical_value,
                        before
                    );

                    // Get the best option; we use `value` (latest
                    // value) here so we have a full view of the
                    // eclass.
                    trace!("looking up best value for {}", value);
                    let (_, best_value) = self.value_to_best_value[value];
                    debug_assert_ne!(best_value, Value::reserved_value());
                    trace!("elaborate: value {} -> best {}", value, best_value);

                    let remat = if let Some(elab_val) =
                        self.value_to_elaborated_value.get(&canonical_value)
                    {
                        // Value is available. Look at the defined
                        // block, and determine whether this node kind
                        // allows rematerialization if the value comes
                        // from another block. If so, ignore the hit
                        // and recompute below.
                        let remat = elab_val.in_block != self.cur_block
                            && self.remat_values.contains(&best_value);
                        if !remat {
                            trace!("elaborate: value {} -> {:?}", value, elab_val);
                            self.stats.elaborate_memoize_hit += 1;
                            self.elab_result_stack.push(*elab_val);
                            continue;
                        }
                        trace!("elaborate: value {} -> remat", canonical_value);
                        self.stats.elaborate_memoize_miss_remat += 1;
                        // The op is pure at this point, so it is always valid to
                        // remove from this map.
                        self.value_to_elaborated_value.remove(&canonical_value);
                        true
                    } else {
                        // Value not available; but still look up
                        // whether it's been flagged for remat because
                        // this affects placement.
                        let remat = self.remat_values.contains(&best_value);
                        trace!(" -> not present in map; remat = {}", remat);
                        remat
                    };
                    self.stats.elaborate_memoize_miss += 1;

                    // Now resolve the value to its definition to see
                    // how we can compute it.
                    let (inst, result_idx) = match self.func.dfg.value_def(best_value) {
                        ValueDef::Result(inst, result_idx) => {
                            trace!(
                                " -> value {} is result {} of {}",
                                best_value,
                                result_idx,
                                inst
                            );
                            (inst, result_idx)
                        }
                        ValueDef::Param(in_block, _) => {
                            // We don't need to do anything to compute
                            // this value; just push its result on the
                            // result stack (blockparams are already
                            // available).
                            trace!(" -> value {} is a blockparam", best_value);
                            self.elab_result_stack.push(ElaboratedValue {
                                in_block,
                                value: best_value,
                            });
                            continue;
                        }
                        ValueDef::Union(_, _) => {
                            panic!("Should never have a Union value as the best value");
                        }
                    };

                    trace!(
                        " -> result {} of inst {:?}",
                        result_idx,
                        self.func.dfg.insts[inst]
                    );

                    // We're going to need to use this instruction
                    // result, placing the instruction into the
                    // layout. First, enqueue all args to be
                    // elaborated. Push state to receive the results
                    // and later elab this inst.
                    let num_args = self.func.dfg.inst_values(inst).count();
                    self.elab_stack.push(ElabStackEntry::PendingInst {
                        inst,
                        result_idx,
                        num_args,
                        remat,
                        before,
                    });

                    // Push args in reverse order so we process the
                    // first arg first.
                    for arg in self.func.dfg.inst_values(inst).rev() {
                        debug_assert_ne!(arg, Value::reserved_value());
                        self.elab_stack
                            .push(ElabStackEntry::Start { value: arg, before });
                    }
                }

                &ElabStackEntry::PendingInst {
                    inst,
                    result_idx,
                    num_args,
                    remat,
                    before,
                } => {
                    self.elab_stack.pop();

                    trace!(
                        "PendingInst: {} result {} args {} remat {} before {}",
                        inst,
                        result_idx,
                        num_args,
                        remat,
                        before
                    );

                    // We should have all args resolved at this
                    // point. Grab them and drain them out, removing
                    // them.
                    let arg_idx = self.elab_result_stack.len() - num_args;
                    let arg_values = &self.elab_result_stack[arg_idx..];

                    // Compute max loop depth.
                    //
                    // Note that if there are no arguments then this instruction
                    // is allowed to get hoisted up one loop. This is not
                    // usually used since no-argument values are things like
                    // constants which are typically rematerialized, but for the
                    // `vconst` instruction 128-bit constants aren't as easily
                    // rematerialized. They're hoisted out of inner loops but
                    // not to the function entry which may run the risk of
                    // placing too much register pressure on the entire
                    // function. This is modeled with the `.saturating_sub(1)`
                    // as the default if there's otherwise no maximum.
                    let loop_hoist_level = arg_values
                        .iter()
                        .map(|&value| {
                            // Find the outermost loop level at which
                            // the value's defining block *is not* a
                            // member. This is the loop-nest level
                            // whose hoist-block we hoist to.
                            let hoist_level = self
                                .loop_stack
                                .iter()
                                .position(|loop_entry| {
                                    !self.loop_analysis.is_in_loop(value.in_block, loop_entry.lp)
                                })
                                .unwrap_or(self.loop_stack.len());
                            trace!(
                                " -> arg: elab_value {:?} hoist level {:?}",
                                value,
                                hoist_level
                            );
                            hoist_level
                        })
                        .max()
                        .unwrap_or(self.loop_stack.len().saturating_sub(1));
                    trace!(
                        " -> loop hoist level: {:?}; cur loop depth: {:?}, loop_stack: {:?}",
                        loop_hoist_level,
                        self.loop_stack.len(),
                        self.loop_stack,
                    );

                    // We know that this is a pure inst, because
                    // non-pure roots have already been placed in the
                    // value-to-elab'd-value map and are never subject
                    // to remat, so they will not reach this stage of
                    // processing.
                    //
                    // We now must determine the location at which we
                    // place the instruction. This is the current
                    // block *unless* we hoist above a loop when all
                    // args are loop-invariant (and this op is pure).
                    let (scope_depth, before, insert_block) =
                        if loop_hoist_level == self.loop_stack.len() || remat {
                            // Depends on some value at the current
                            // loop depth, or remat forces it here:
                            // place it at the current location.
                            (
                                self.value_to_elaborated_value.depth(),
                                before,
                                self.func.layout.inst_block(before).unwrap(),
                            )
                        } else {
                            // Does not depend on any args at current
                            // loop depth: hoist out of loop.
                            self.stats.elaborate_licm_hoist += 1;
                            let data = &self.loop_stack[loop_hoist_level];
                            // `data.hoist_block` should dominate `before`'s block.
                            let before_block = self.func.layout.inst_block(before).unwrap();
                            debug_assert!(self.domtree.dominates(
                                data.hoist_block,
                                before_block,
                                &self.func.layout
                            ));
                            // Determine the instruction at which we
                            // insert in `data.hoist_block`.
                            let before = self.func.layout.last_inst(data.hoist_block).unwrap();
                            (data.scope_depth as usize, before, data.hoist_block)
                        };

                    trace!(
                        " -> decided to place: before {} insert_block {}",
                        before,
                        insert_block
                    );

                    //  Now we need to place `inst` at the computed
                    //  location (just before `before`). Note that
                    //  `inst` may already have been placed somewhere
                    //  else, because a pure node may be elaborated at
                    //  more than one place. In this case, we need to
                    //  duplicate the instruction (and return the
                    //  `Value`s for that duplicated instance
                    //  instead).
                    trace!("need inst {} before {}", inst, before);
                    let inst = if self.func.layout.inst_block(inst).is_some() {
                        // Clone the inst!
                        let new_inst = self.func.dfg.clone_inst(inst);
                        trace!(
                            " -> inst {} already has a location; cloned to {}",
                            inst,
                            new_inst
                        );
                        // Create mappings in the
                        // value-to-elab'd-value map from original
                        // results to cloned results.
                        for (&result, &new_result) in self
                            .func
                            .dfg
                            .inst_results(inst)
                            .iter()
                            .zip(self.func.dfg.inst_results(new_inst).iter())
                        {
                            let elab_value = ElaboratedValue {
                                value: new_result,
                                in_block: insert_block,
                            };
                            let canonical_result = self.eclasses.find_and_update(result);
                            self.value_to_elaborated_value.insert_if_absent_with_depth(
                                canonical_result,
                                elab_value,
                                scope_depth,
                            );

                            self.eclasses.add(new_result);
                            self.eclasses.union(result, new_result);
                            self.value_to_best_value[new_result] = self.value_to_best_value[result];

                            trace!(
                                " -> cloned inst has new result {} for orig {}",
                                new_result,
                                result
                            );
                        }
                        new_inst
                    } else {
                        trace!(" -> no location; using original inst");
                        // Create identity mappings from result values
                        // to themselves in this scope, since we're
                        // using the original inst.
                        for &result in self.func.dfg.inst_results(inst) {
                            let elab_value = ElaboratedValue {
                                value: result,
                                in_block: insert_block,
                            };
                            let canonical_result = self.eclasses.find_and_update(result);
                            self.value_to_elaborated_value.insert_if_absent_with_depth(
                                canonical_result,
                                elab_value,
                                scope_depth,
                            );
                            trace!(" -> inserting identity mapping for {}", result);
                        }
                        inst
                    };
                    // Place the inst just before `before`.
                    self.func.layout.insert_inst(inst, before);

                    // Update the inst's arguments.
                    self.func
                        .dfg
                        .overwrite_inst_values(inst, arg_values.into_iter().map(|ev| ev.value));

                    // Now that we've consumed the arg values, pop
                    // them off the stack.
                    self.elab_result_stack.truncate(arg_idx);

                    // Push the requested result index of the
                    // instruction onto the elab-results stack.
                    self.elab_result_stack.push(ElaboratedValue {
                        in_block: insert_block,
                        value: self.func.dfg.inst_results(inst)[result_idx],
                    });
                }
            }
        }
    }

    fn elaborate_block(&mut self, elab_values: &mut Vec<Value>, idom: Option<Block>, block: Block) {
        trace!("elaborate_block: block {}", block);
        self.start_block(idom, block);

        // Iterate over the side-effecting skeleton using the linked
        // list in Layout. We will insert instructions that are
        // elaborated *before* `inst`, so we can always use its
        // next-link to continue the iteration.
        let mut next_inst = self.func.layout.first_inst(block);
        let mut first_branch = None;
        while let Some(inst) = next_inst {
            trace!(
                "elaborating inst {} with results {:?}",
                inst,
                self.func.dfg.inst_results(inst)
            );
            // Record the first branch we see in the block; all
            // elaboration for args of *any* branch must be inserted
            // before the *first* branch, because the branch group
            // must remain contiguous at the end of the block.
            if self.func.dfg.insts[inst].opcode().is_branch() && first_branch == None {
                first_branch = Some(inst);
            }

            // Determine where elaboration inserts insts.
            let before = first_branch.unwrap_or(inst);
            trace!(" -> inserting before {}", before);

            elab_values.extend(self.func.dfg.inst_values(inst));
            for arg in elab_values.iter_mut() {
                trace!(" -> arg {}", *arg);
                // Elaborate the arg, placing any newly-inserted insts
                // before `before`. Get the updated value, which may
                // be different than the original.
                let new_arg = self.elaborate_eclass_use(*arg, before);
                trace!("   -> rewrote arg to {:?}", new_arg);
                *arg = new_arg.value;
            }
            self.func
                .dfg
                .overwrite_inst_values(inst, elab_values.drain(..));

            // We need to put the results of this instruction in the
            // map now.
            for &result in self.func.dfg.inst_results(inst) {
                trace!(" -> result {}", result);
                let canonical_result = self.eclasses.find_and_update(result);
                self.value_to_elaborated_value.insert_if_absent(
                    canonical_result,
                    ElaboratedValue {
                        in_block: block,
                        value: result,
                    },
                );
            }

            next_inst = self.func.layout.next_inst(inst);
        }
    }

    fn elaborate_domtree(&mut self, domtree: &DomTreeWithChildren) {
        let root = domtree.root();
        self.block_stack.push(BlockStackEntry::Elaborate {
            block: root,
            idom: None,
        });

        // A temporary workspace for elaborate_block, allocated here to maximize the use of the
        // allocation.
        let mut elab_values = Vec::new();

        while let Some(top) = self.block_stack.pop() {
            match top {
                BlockStackEntry::Elaborate { block, idom } => {
                    self.block_stack.push(BlockStackEntry::Pop);
                    self.value_to_elaborated_value.increment_depth();

                    self.elaborate_block(&mut elab_values, idom, block);

                    // Push children. We are doing a preorder
                    // traversal so we do this after processing this
                    // block above.
                    let block_stack_end = self.block_stack.len();
                    for child in domtree.children(block) {
                        self.block_stack.push(BlockStackEntry::Elaborate {
                            block: child,
                            idom: Some(block),
                        });
                    }
                    // Reverse what we just pushed so we elaborate in
                    // original block order. (The domtree iter is a
                    // single-ended iter over a singly-linked list so
                    // we can't `.rev()` above.)
                    self.block_stack[block_stack_end..].reverse();
                }
                BlockStackEntry::Pop => {
                    self.value_to_elaborated_value.decrement_depth();
                }
            }
        }
    }

    pub(crate) fn elaborate(&mut self) {
        self.stats.elaborate_func += 1;
        self.stats.elaborate_func_pre_insts += self.func.dfg.num_insts() as u64;
        self.compute_best_values();
        self.elaborate_domtree(&self.domtree_children);
        self.stats.elaborate_func_post_insts += self.func.dfg.num_insts() as u64;
    }
}
