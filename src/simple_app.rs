use color_eyre::eyre::Result as RepResult;

use crate::{
    app::App,
    binaries::{
        BinaryNode,
        attach_manpaths,
        search_binaries,
    },
    cli::Cli,
    clipboard::clipboard_copy,
};

#[derive(Default)]
pub struct SimpleApp {
    args: Cli
}

impl App for SimpleApp {
    fn with_args(args: Cli) -> Self {
        Self { args }
    }

    fn run(&mut self) -> RepResult<()> {
        let input = self.args.input.as_ref().unwrap();

        let search_result = search_binaries(input);
        let mut result_iter = search_result
            .owned_ordered_iter()
            .peekable();

        if let Some(b) = result_iter.peek()
           && self.args.should_copy_result
        {
            let readable_binary = b.read().unwrap();
            let name = &readable_binary.name;

            let _ = clipboard_copy(name);
            drop(readable_binary);
        }

        let binaries = result_iter
            .take(self.args.result_length)
            .collect::<Vec<BinaryNode>>();

        if self.args.show_descriptions {
            attach_manpaths(&binaries);
        }

        for binary in binaries {
            let readable_binary = binary.read().unwrap();

            print!("{}", readable_binary.name);

            if !self.args.show_descriptions {
                print!("\n");
                continue;
            }

            if let Some(desc) = readable_binary.get_description() {
                println!(":\n{}", desc.value);
            } else {
                println!(",");
            }
        }

        Ok(())
    }
}
