use crate::{
    evm_circuit::{
        execution::ExecutionGadget,
        step::ExecutionState,
        table::CallContextFieldTag,
        util::{
            common_gadget::SameContextGadget,
            constraint_builder::{ConstraintBuilder, StepStateTransition, Transition::Delta},
            Cell, Word,
        },
        witness::{Block, Call, ExecStep, Transaction},
    },
    util::Expr,
};
use eth_types::Field;
use eth_types::ToLittleEndian;
use halo2_proofs::{circuit::Region, plonk::Error};

#[derive(Clone, Debug)]
pub(crate) struct CallValueGadget<F> {
    same_context: SameContextGadget<F>,
    // Value in rw_table->stack_op and call_context->call_value are both RLC
    // encoded, so no need to decode.
    call_value: Cell<F>,
}

impl<F: Field> ExecutionGadget<F> for CallValueGadget<F> {
    const NAME: &'static str = "CALLVALUE";

    const EXECUTION_STATE: ExecutionState = ExecutionState::CALLVALUE;

    fn configure(cb: &mut ConstraintBuilder<F>) -> Self {
        let call_value = cb.query_cell();

        // Lookup rw_table -> call_context with call value
        cb.call_context_lookup(
            false.expr(),
            None, // cb.curr.state.call_id
            CallContextFieldTag::Value,
            call_value.expr(),
        );

        // Push the value to the stack
        cb.stack_push(call_value.expr());

        // State transition
        let opcode = cb.query_cell();
        let step_state_transition = StepStateTransition {
            rw_counter: Delta(2.expr()),
            program_counter: Delta(1.expr()),
            stack_pointer: Delta((-1).expr()),
            ..Default::default()
        };
        let same_context = SameContextGadget::construct(cb, opcode, step_state_transition, None);

        Self {
            same_context,
            call_value,
        }
    }

    fn assign_exec_step(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        block: &Block<F>,
        _: &Transaction,
        _: &Call,
        step: &ExecStep,
    ) -> Result<(), Error> {
        self.same_context.assign_exec_step(region, offset, step)?;

        let call_value = block.rws[step.rw_indices[1]].stack_value();

        self.call_value.assign(
            region,
            offset,
            Some(Word::random_linear_combine(
                call_value.to_le_bytes(),
                block.randomness,
            )),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_util::run_test_circuits;
    use eth_types::bytecode;

    fn test_ok() {
        let bytecode = bytecode! {
            #[start]
            CALLVALUE
            STOP
        };
        assert_eq!(run_test_circuits(bytecode), Ok(()));
    }
    #[test]
    fn callvalue_gadget_test() {
        test_ok();
    }
}