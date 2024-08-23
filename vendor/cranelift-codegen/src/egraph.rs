//! Support for egraphs represented in the DataFlowGraph.

use crate::alias_analysis::{AliasAnalysis, LastStores};
use crate::ctxhash::{CtxEq, CtxHash, CtxHashMap};
use crate::cursor::{Cursor, CursorPosition, FuncCursor};
use crate::dominator_tree::DominatorTree;
use crate::egraph::domtree::DomTreeWithChildren;
use crate::egraph::elaborate::Elaborator;
use crate::fx::FxHashSet;
use crate::inst_predicates::{is_mergeable_for_egraph, is_pure_for_egraph};
use crate::ir::{
    Block, DataFlowGraph, Function, Inst, InstructionData, Type, Value, ValueDef, ValueListPool,
};
use crate::loop_analysis::LoopAnalysis;
use crate::opts::generated_code::ContextIter;
use crate::opts::IsleContext;
use crate::scoped_hash_map::{Entry as ScopedEntry, ScopedHashMap};
use crate::trace;
use crate::unionfind::UnionFind;
use cranelift_entity::packed_option::ReservedValue;
use cranelift_entity::SecondaryMap;
use std::hash::Hasher;

mod cost;
mod domtree;
mod elaborate;

/// Pass over a Function that does the whole aegraph thing.
///
/// - Removes non-skeleton nodes from the Layout.
/// - Performs a GVN-and-rule-application pass over all Values
///   reachable from the skeleton, potentially creating new Union
///   nodes (i.e., an aegraph) so that some values have multiple
///   representations.
/// - Does "extraction" on the aegraph: selects the best value out of
///   the tree-of-Union nodes for each used value.
/// - Does "scoped elaboration" on the aegraph: chooses one or more
///   locations for pure nodes to become instructions again in the
///   layout, as forced by the skeleton.
///
/// At the beginning and end of this pass, the CLIF should be in a
/// state that passes the verifier and, additionally, has no Union
/// nodes. During the pass, Union nodes may exist, and instructions in
/// the layout may refer to results of instructions that are not
/// placed in the layout.
pub struct EgraphPass<'a> {
    /// The function we're operating on.
    func: &'a mut Function,
    /// Dominator tree, used for elaboration pass.
    domtree: &'a DominatorTree,
    /// Alias analysis, used during optimization.
    alias_analysis: &'a mut AliasAnalysis<'a>,
    /// "Domtree with children": like `domtree`, but with an explicit
    /// list of children, rather than just parent pointers.
    domtree_children: DomTreeWithChildren,
    /// Loop analysis results, used for built-in LICM during
    /// elaboration.
    loop_analysis: &'a LoopAnalysis,
    /// Which canonical Values do we want to rematerialize in each
    /// block where they're used?
    ///
    /// (A canonical Value is the *oldest* Value in an eclass,
    /// i.e. tree of union value-nodes).
    remat_values: FxHashSet<Value>,
    /// Stats collected while we run this pass.
    pub(crate) stats: Stats,
    /// Union-find that maps all members of a Union tree (eclass) back
    /// to the *oldest* (lowest-numbered) `Value`.
    eclasses: UnionFind<Value>,
}

/// Context passed through node insertion and optimization.
pub(crate) struct OptimizeCtx<'opt, 'analysis>
where
    'analysis: 'opt,
{
    // Borrowed from EgraphPass:
    pub(crate) func: &'opt mut Function,
    pub(crate) value_to_opt_value: &'opt mut SecondaryMap<Value, Value>,
    pub(crate) gvn_map: &'opt mut CtxHashMap<(Type, InstructionData), Value>,
    pub(crate) effectful_gvn_map: &'opt mut ScopedHashMap<(Type, InstructionData), Value>,
    pub(crate) eclasses: &'opt mut UnionFind<Value>,
    pub(crate) remat_values: &'opt mut FxHashSet<Value>,
    pub(crate) stats: &'opt mut Stats,
    pub(crate) alias_analysis: &'opt mut AliasAnalysis<'analysis>,
    pub(crate) alias_analysis_state: &'opt mut LastStores,
    // Held locally during optimization of one node (recursively):
    pub(crate) rewrite_depth: usize,
    pub(crate) subsume_values: FxHashSet<Value>,
}

/// For passing to `insert_pure_enode`. Sometimes the enode already
/// exists as an Inst (from the original CLIF), and sometimes we're in
/// the middle of creating it and want to avoid inserting it if
/// possible until we know we need it.
pub(crate) enum NewOrExistingInst {
    New(InstructionData, Type),
    Existing(Inst),
}

impl NewOrExistingInst {
    fn get_inst_key<'a>(&'a self, dfg: &'a DataFlowGraph) -> (Type, InstructionData) {
        match self {
            NewOrExistingInst::New(data, ty) => (*ty, *data),
            NewOrExistingInst::Existing(inst) => {
                let ty = dfg.ctrl_typevar(*inst);
                (ty, dfg.insts[*inst].clone())
            }
        }
    }
}

impl<'opt, 'analysis> OptimizeCtx<'opt, 'analysis>
where
    'analysis: 'opt,
{
    /// Optimization of a single instruction.
    ///
    /// This does a few things:
    /// - Looks up the instruction in the GVN deduplication map. If we
    ///   already have the same instruction somewhere else, with the
    ///   same args, then we can alias the original instruction's
    ///   results and omit this instruction entirely.
    ///   - Note that we do this canonicalization based on the
    ///     instruction with its arguments as *canonical* eclass IDs,
    ///     that is, the oldest (smallest index) `Value` reachable in
    ///     the tree-of-unions (whole eclass). This ensures that we
    ///     properly canonicalize newer nodes that use newer "versions"
    ///     of a value that are still equal to the older versions.
    /// - If the instruction is "new" (not deduplicated), then apply
    ///   optimization rules:
    ///   - All of the mid-end rules written in ISLE.
    ///   - Store-to-load forwarding.
    /// - Update the value-to-opt-value map, and update the eclass
    ///   union-find, if we rewrote the value to different form(s).
    pub(crate) fn insert_pure_enode(&mut self, inst: NewOrExistingInst) -> Value {
        // Create the external context for looking up and updating the
        // GVN map. This is necessary so that instructions themselves
        // do not have to carry all the references or data for a full
        // `Eq` or `Hash` impl.
        let gvn_context = GVNContext {
            union_find: self.eclasses,
            value_lists: &self.func.dfg.value_lists,
        };

        self.stats.pure_inst += 1;
        if let NewOrExistingInst::New(..) = inst {
            self.stats.new_inst += 1;
        }

        // Does this instruction already exist? If so, add entries to
        // the value-map to rewrite uses of its results to the results
        // of the original (existing) instruction. If not, optimize
        // the new instruction.
        if let Some(&orig_result) = self
            .gvn_map
            .get(&inst.get_inst_key(&self.func.dfg), &gvn_context)
        {
            self.stats.pure_inst_deduped += 1;
            if let NewOrExistingInst::Existing(inst) = inst {
                debug_assert_eq!(self.func.dfg.inst_results(inst).len(), 1);
                let result = self.func.dfg.first_result(inst);
                self.value_to_opt_value[result] = orig_result;
                self.eclasses.union(result, orig_result);
                self.stats.union += 1;
                result
            } else {
                orig_result
            }
        } else {
            // Now actually insert the InstructionData and attach
            // result value (exactly one).
            let (inst, result, ty) = match inst {
                NewOrExistingInst::New(data, typevar) => {
                    let inst = self.func.dfg.make_inst(data);
                    // TODO: reuse return value?
                    self.func.dfg.make_inst_results(inst, typevar);
                    let result = self.func.dfg.first_result(inst);
                    // Add to eclass unionfind.
                    self.eclasses.add(result);
                    // New inst. We need to do the analysis of its result.
                    (inst, result, typevar)
                }
                NewOrExistingInst::Existing(inst) => {
                    let result = self.func.dfg.first_result(inst);
                    let ty = self.func.dfg.ctrl_typevar(inst);
                    (inst, result, ty)
                }
            };

            let opt_value = self.optimize_pure_enode(inst);
            let gvn_context = GVNContext {
                union_find: self.eclasses,
                value_lists: &self.func.dfg.value_lists,
            };
            self.gvn_map.insert(
                (ty, self.func.dfg.insts[inst].clone()),
                opt_value,
                &gvn_context,
            );
            self.value_to_opt_value[result] = opt_value;
            opt_value
        }
    }

    /// Optimizes an enode by applying any matching mid-end rewrite
    /// rules (or store-to-load forwarding, which is a special case),
    /// unioning together all possible optimized (or rewritten) forms
    /// of this expression into an eclass and returning the `Value`
    /// that represents that eclass.
    fn optimize_pure_enode(&mut self, inst: Inst) -> Value {
        // A pure node always has exactly one result.
        let orig_value = self.func.dfg.first_result(inst);

        let mut isle_ctx = IsleContext { ctx: self };

        // Limit rewrite depth. When we apply optimization rules, they
        // may create new nodes (values) and those are, recursively,
        // optimized eagerly as soon as they are created. So we may
        // have more than one ISLE invocation on the stack. (This is
        // necessary so that as the toplevel builds the
        // right-hand-side expression bottom-up, it uses the "latest"
        // optimized values for all the constituent parts.) To avoid
        // infinite or problematic recursion, we bound the rewrite
        // depth to a small constant here.
        const REWRITE_LIMIT: usize = 5;
        if isle_ctx.ctx.rewrite_depth > REWRITE_LIMIT {
            isle_ctx.ctx.stats.rewrite_depth_limit += 1;
            return orig_value;
        }
        isle_ctx.ctx.rewrite_depth += 1;

        // Invoke the ISLE toplevel constructor, getting all new
        // values produced as equivalents to this value.
        trace!("Calling into ISLE with original value {}", orig_value);
        isle_ctx.ctx.stats.rewrite_rule_invoked += 1;
        let mut optimized_values =
            crate::opts::generated_code::constructor_simplify(&mut isle_ctx, orig_value);

        // Create a union of all new values with the original (or
        // maybe just one new value marked as "subsuming" the
        // original, if present.)
        let mut union_value = orig_value;
        while let Some(optimized_value) = optimized_values.next(&mut isle_ctx) {
            trace!(
                "Returned from ISLE for {}, got {:?}",
                orig_value,
                optimized_value
            );
            if optimized_value == orig_value {
                trace!(" -> same as orig value; skipping");
                continue;
            }
            if isle_ctx.ctx.subsume_values.contains(&optimized_value) {
                // Merge in the unionfind so canonicalization
                // still works, but take *only* the subsuming
                // value, and break now.
                isle_ctx.ctx.eclasses.union(optimized_value, union_value);
                union_value = optimized_value;
                break;
            }

            let old_union_value = union_value;
            union_value = isle_ctx
                .ctx
                .func
                .dfg
                .union(old_union_value, optimized_value);
            isle_ctx.ctx.stats.union += 1;
            trace!(" -> union: now {}", union_value);
            isle_ctx.ctx.eclasses.add(union_value);
            isle_ctx
                .ctx
                .eclasses
                .union(old_union_value, optimized_value);
            isle_ctx.ctx.eclasses.union(old_union_value, union_value);
        }

        isle_ctx.ctx.rewrite_depth -= 1;

        union_value
    }

    /// Optimize a "skeleton" instruction, possibly removing
    /// it. Returns `true` if the instruction should be removed from
    /// the layout.
    fn optimize_skeleton_inst(&mut self, inst: Inst) -> bool {
        self.stats.skeleton_inst += 1;

        // First, can we try to deduplicate? We need to keep some copy
        // of the instruction around because it's side-effecting, but
        // we may be able to reuse an earlier instance of it.
        if is_mergeable_for_egraph(self.func, inst) {
            let result = self.func.dfg.inst_results(inst)[0];
            trace!(" -> mergeable side-effecting op {}", inst);

            // Does this instruction already exist? If so, add entries to
            // the value-map to rewrite uses of its results to the results
            // of the original (existing) instruction. If not, optimize
            // the new instruction.
            //
            // Note that we use the "effectful GVN map", which is
            // scoped: because effectful ops are not removed from the
            // skeleton (`Layout`), we need to be mindful of whether
            // our current position is dominated by an instance of the
            // instruction. (See #5796 for details.)
            let ty = self.func.dfg.ctrl_typevar(inst);
            match self
                .effectful_gvn_map
                .entry((ty, self.func.dfg.insts[inst].clone()))
            {
                ScopedEntry::Occupied(o) => {
                    let orig_result = *o.get();
                    // Hit in GVN map -- reuse value.
                    self.value_to_opt_value[result] = orig_result;
                    self.eclasses.union(orig_result, result);
                    trace!(" -> merges result {} to {}", result, orig_result);
                    true
                }
                ScopedEntry::Vacant(v) => {
                    // Otherwise, insert it into the value-map.
                    self.value_to_opt_value[result] = result;
                    v.insert(result);
                    trace!(" -> inserts as new (no GVN)");
                    false
                }
            }
        }
        // Otherwise, if a load or store, process it with the alias
        // analysis to see if we can optimize it (rewrite in terms of
        // an earlier load or stored value).
        else if let Some(new_result) =
            self.alias_analysis
                .process_inst(self.func, self.alias_analysis_state, inst)
        {
            self.stats.alias_analysis_removed += 1;
            let result = self.func.dfg.first_result(inst);
            trace!(
                " -> inst {} has result {} replaced with {}",
                inst,
                result,
                new_result
            );
            self.value_to_opt_value[result] = new_result;
            true
        }
        // Otherwise, generic side-effecting op -- always keep it, and
        // set its results to identity-map to original values.
        else {
            // Set all results to identity-map to themselves
            // in the value-to-opt-value map.
            for &result in self.func.dfg.inst_results(inst) {
                self.value_to_opt_value[result] = result;
                self.eclasses.add(result);
            }
            false
        }
    }
}

impl<'a> EgraphPass<'a> {
    /// Create a new EgraphPass.
    pub fn new(
        func: &'a mut Function,
        domtree: &'a DominatorTree,
        loop_analysis: &'a LoopAnalysis,
        alias_analysis: &'a mut AliasAnalysis<'a>,
    ) -> Self {
        let num_values = func.dfg.num_values();
        let domtree_children = DomTreeWithChildren::new(func, domtree);
        Self {
            func,
            domtree,
            domtree_children,
            loop_analysis,
            alias_analysis,
            stats: Stats::default(),
            eclasses: UnionFind::with_capacity(num_values),
            remat_values: FxHashSet::default(),
        }
    }

    /// Run the process.
    pub fn run(&mut self) {
        self.remove_pure_and_optimize();

        trace!("egraph built:\n{}\n", self.func.display());
        if cfg!(feature = "trace-log") {
            for (value, def) in self.func.dfg.values_and_defs() {
                trace!(" -> {} = {:?}", value, def);
                match def {
                    ValueDef::Result(i, 0) => {
                        trace!("  -> {} = {:?}", i, self.func.dfg.insts[i]);
                    }
                    _ => {}
                }
            }
        }
        trace!("stats: {:?}", self.stats);
        self.elaborate();
    }

    /// Remove pure nodes from the `Layout` of the function, ensuring
    /// that only the "side-effect skeleton" remains, and also
    /// optimize the pure nodes. This is the first step of
    /// egraph-based processing and turns the pure CFG-based CLIF into
    /// a CFG skeleton with a sea of (optimized) nodes tying it
    /// together.
    ///
    /// As we walk through the code, we eagerly apply optimization
    /// rules; at any given point we have a "latest version" of an
    /// eclass of possible representations for a `Value` in the
    /// original program, which is itself a `Value` at the root of a
    /// union-tree. We keep a map from the original values to these
    /// optimized values. When we encounter any instruction (pure or
    /// side-effecting skeleton) we rewrite its arguments to capture
    /// the "latest" optimized forms of these values. (We need to do
    /// this as part of this pass, and not later using a finished map,
    /// because the eclass can continue to be updated and we need to
    /// only refer to its subset that exists at this stage, to
    /// maintain acyclicity.)
    fn remove_pure_and_optimize(&mut self) {
        let mut cursor = FuncCursor::new(self.func);
        let mut value_to_opt_value: SecondaryMap<Value, Value> =
            SecondaryMap::with_default(Value::reserved_value());
        // Map from instruction to value for hash-consing of pure ops
        // into the egraph. This can be a standard (non-scoped)
        // hashmap because pure ops have no location: they are
        // "outside of" control flow.
        //
        // Note also that we keep the controlling typevar (the `Type`
        // in the tuple below) because it may disambiguate
        // instructions that are identical except for type.
        let mut gvn_map: CtxHashMap<(Type, InstructionData), Value> =
            CtxHashMap::with_capacity(cursor.func.dfg.num_values());
        // Map from instruction to value for GVN'ing of effectful but
        // idempotent ops, which remain in the side-effecting
        // skeleton. This needs to be scoped because we cannot
        // deduplicate one instruction to another that is in a
        // non-dominating block.
        //
        // Note that we can use a ScopedHashMap here without the
        // "context" (as needed by CtxHashMap) because in practice the
        // ops we want to GVN have all their args inline. Equality on
        // the InstructionData itself is conservative: two insts whose
        // struct contents compare shallowly equal are definitely
        // identical, but identical insts in a deep-equality sense may
        // not compare shallowly equal, due to list indirection. This
        // is fine for GVN, because it is still sound to skip any
        // given GVN opportunity (and keep the original instructions).
        //
        // As above, we keep the controlling typevar here as part of
        // the key: effectful instructions may (as for pure
        // instructions) be differentiated only on the type.
        let mut effectful_gvn_map: ScopedHashMap<(Type, InstructionData), Value> =
            ScopedHashMap::new();

        // In domtree preorder, visit blocks. (TODO: factor out an
        // iterator from this and elaborator.)
        let root = self.domtree_children.root();
        enum StackEntry {
            Visit(Block),
            Pop,
        }
        let mut block_stack = vec![StackEntry::Visit(root)];
        while let Some(entry) = block_stack.pop() {
            match entry {
                StackEntry::Visit(block) => {
                    // We popped this block; push children
                    // immediately, then process this block.
                    block_stack.push(StackEntry::Pop);
                    block_stack
                        .extend(self.domtree_children.children(block).map(StackEntry::Visit));
                    effectful_gvn_map.increment_depth();

                    trace!("Processing block {}", block);
                    cursor.set_position(CursorPosition::Before(block));

                    let mut alias_analysis_state = self.alias_analysis.block_starting_state(block);

                    for &param in cursor.func.dfg.block_params(block) {
                        trace!("creating initial singleton eclass for blockparam {}", param);
                        self.eclasses.add(param);
                        value_to_opt_value[param] = param;
                    }
                    while let Some(inst) = cursor.next_inst() {
                        trace!("Processing inst {}", inst);

                        // While we're passing over all insts, create initial
                        // singleton eclasses for all result and blockparam
                        // values.  Also do initial analysis of all inst
                        // results.
                        for &result in cursor.func.dfg.inst_results(inst) {
                            trace!("creating initial singleton eclass for {}", result);
                            self.eclasses.add(result);
                        }

                        // Rewrite args of *all* instructions using the
                        // value-to-opt-value map.
                        cursor.func.dfg.resolve_aliases_in_arguments(inst);
                        cursor.func.dfg.map_inst_values(inst, |_, arg| {
                            let new_value = value_to_opt_value[arg];
                            trace!("rewriting arg {} of inst {} to {}", arg, inst, new_value);
                            debug_assert_ne!(new_value, Value::reserved_value());
                            new_value
                        });

                        // Build a context for optimization, with borrows of
                        // state. We can't invoke a method on `self` because
                        // we've borrowed `self.func` mutably (as
                        // `cursor.func`) so we pull apart the pieces instead
                        // here.
                        let mut ctx = OptimizeCtx {
                            func: cursor.func,
                            value_to_opt_value: &mut value_to_opt_value,
                            gvn_map: &mut gvn_map,
                            effectful_gvn_map: &mut effectful_gvn_map,
                            eclasses: &mut self.eclasses,
                            rewrite_depth: 0,
                            subsume_values: FxHashSet::default(),
                            remat_values: &mut self.remat_values,
                            stats: &mut self.stats,
                            alias_analysis: self.alias_analysis,
                            alias_analysis_state: &mut alias_analysis_state,
                        };

                        if is_pure_for_egraph(ctx.func, inst) {
                            // Insert into GVN map and optimize any new nodes
                            // inserted (recursively performing this work for
                            // any nodes the optimization rules produce).
                            let inst = NewOrExistingInst::Existing(inst);
                            ctx.insert_pure_enode(inst);
                            // We've now rewritten all uses, or will when we
                            // see them, and the instruction exists as a pure
                            // enode in the eclass, so we can remove it.
                            cursor.remove_inst_and_step_back();
                        } else {
                            if ctx.optimize_skeleton_inst(inst) {
                                cursor.remove_inst_and_step_back();
                            }
                        }
                    }
                }
                StackEntry::Pop => {
                    effectful_gvn_map.decrement_depth();
                }
            }
        }
    }

    /// Scoped elaboration: compute a final ordering of op computation
    /// for each block and update the given Func body. After this
    /// runs, the function body is back into the state where every
    /// Inst with an used result is placed in the layout (possibly
    /// duplicated, if our code-motion logic decides this is the best
    /// option).
    ///
    /// This works in concert with the domtree. We do a preorder
    /// traversal of the domtree, tracking a scoped map from Id to
    /// (new) Value. The map's scopes correspond to levels in the
    /// domtree.
    ///
    /// At each block, we iterate forward over the side-effecting
    /// eclasses, and recursively generate their arg eclasses, then
    /// emit the ops themselves.
    ///
    /// To use an eclass in a given block, we first look it up in the
    /// scoped map, and get the Value if already present. If not, we
    /// need to generate it. We emit the extracted enode for this
    /// eclass after recursively generating its args. Eclasses are
    /// thus computed "as late as possible", but then memoized into
    /// the Id-to-Value map and available to all dominated blocks and
    /// for the rest of this block. (This subsumes GVN.)
    fn elaborate(&mut self) {
        let mut elaborator = Elaborator::new(
            self.func,
            self.domtree,
            &self.domtree_children,
            self.loop_analysis,
            &mut self.remat_values,
            &mut self.eclasses,
            &mut self.stats,
        );
        elaborator.elaborate();

        self.check_post_egraph();
    }

    #[cfg(debug_assertions)]
    fn check_post_egraph(&self) {
        // Verify that no union nodes are reachable from inst args,
        // and that all inst args' defining instructions are in the
        // layout.
        for block in self.func.layout.blocks() {
            for inst in self.func.layout.block_insts(block) {
                self.func
                    .dfg
                    .inst_values(inst)
                    .for_each(|arg| match self.func.dfg.value_def(arg) {
                        ValueDef::Result(i, _) => {
                            debug_assert!(self.func.layout.inst_block(i).is_some());
                        }
                        ValueDef::Union(..) => {
                            panic!("egraph union node {} still reachable at {}!", arg, inst);
                        }
                        _ => {}
                    })
            }
        }
    }

    #[cfg(not(debug_assertions))]
    fn check_post_egraph(&self) {}
}

/// Implementation of external-context equality and hashing on
/// InstructionData. This allows us to deduplicate instructions given
/// some context that lets us see its value lists and the mapping from
/// any value to "canonical value" (in an eclass).
struct GVNContext<'a> {
    value_lists: &'a ValueListPool,
    union_find: &'a UnionFind<Value>,
}

impl<'a> CtxEq<(Type, InstructionData), (Type, InstructionData)> for GVNContext<'a> {
    fn ctx_eq(
        &self,
        (a_ty, a_inst): &(Type, InstructionData),
        (b_ty, b_inst): &(Type, InstructionData),
    ) -> bool {
        a_ty == b_ty
            && a_inst.eq(b_inst, self.value_lists, |value| {
                self.union_find.find(value)
            })
    }
}

impl<'a> CtxHash<(Type, InstructionData)> for GVNContext<'a> {
    fn ctx_hash<H: Hasher>(&self, state: &mut H, (ty, inst): &(Type, InstructionData)) {
        std::hash::Hash::hash(&ty, state);
        inst.hash(state, self.value_lists, |value| self.union_find.find(value));
    }
}

/// Statistics collected during egraph-based processing.
#[derive(Clone, Debug, Default)]
pub(crate) struct Stats {
    pub(crate) pure_inst: u64,
    pub(crate) pure_inst_deduped: u64,
    pub(crate) skeleton_inst: u64,
    pub(crate) alias_analysis_removed: u64,
    pub(crate) new_inst: u64,
    pub(crate) union: u64,
    pub(crate) subsume: u64,
    pub(crate) remat: u64,
    pub(crate) rewrite_rule_invoked: u64,
    pub(crate) rewrite_depth_limit: u64,
    pub(crate) elaborate_visit_node: u64,
    pub(crate) elaborate_memoize_hit: u64,
    pub(crate) elaborate_memoize_miss: u64,
    pub(crate) elaborate_memoize_miss_remat: u64,
    pub(crate) elaborate_licm_hoist: u64,
    pub(crate) elaborate_func: u64,
    pub(crate) elaborate_func_pre_insts: u64,
    pub(crate) elaborate_func_post_insts: u64,
}
