use std::process::ExitCode;

fn main() -> ExitCode {
    cat_context::cli::complete();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();

    match runtime {
        Ok(runtime) => ExitCode::from(runtime.block_on(cat_context::run())),
        Err(error) => {
            eprintln!("не поднялся рантайм: {error}");
            ExitCode::FAILURE
        }
    }
}
