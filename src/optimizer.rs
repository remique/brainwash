use std::{borrow::Borrow, marker::PhantomData};

use inkwell::{
    passes::{PassManager, PassManagerSubType},
    values::FunctionValue,
};

pub struct OptWrapper<T> {
    pub pass_manager: PassManager<T>,
    sub_type: PhantomData<T>,
}

impl<T: PassManagerSubType> OptWrapper<T> {
    pub fn new<I: Borrow<T::Input>>(input: I) -> Self {
        OptWrapper {
            pass_manager: PassManager::create(input),
            sub_type: PhantomData,
        }
    }
}

impl OptWrapper<FunctionValue<'_>> {
    pub fn optimize(&self) {
        self.run_passes();
        self.initialize();
    }

    pub fn run_passes(&self) {
        self.pass_manager.add_verifier_pass();
        self.pass_manager.add_early_cse_pass();
        self.pass_manager.add_cfg_simplification_pass();
        self.pass_manager.add_scalar_repl_aggregates_pass();
        self.pass_manager.add_instruction_combining_pass();
        self.pass_manager.add_loop_vectorize_pass();
        self.pass_manager.add_slp_vectorize_pass();
        self.pass_manager.add_verifier_pass();
    }

    pub fn initialize(&self) {
        self.pass_manager.initialize();
    }
}
