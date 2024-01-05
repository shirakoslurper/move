When does stackless bytecode come in?

Do we compile directly to stackless bytecode in the prover or do we convert normall bytecode to stackless bytecode? (I'm assuming it's the latter.)

The pipelines come in `move_prover::create_and_process_bytecode()` and that functino is called after the model is built.