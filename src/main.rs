use dose2gmsh::{Cli, DoseBlock};
use structopt::StructOpt;

fn main() -> Result<(), std::io::Error> {
    let args = Cli::from_args();

    let data = DoseBlock::from_3d_dose(&args.input_file)?;

    let output = if let Some(output_file) = args.output_file {
        output_file
    } else {
        let mut output_file = args.input_file.clone();
        output_file.set_extension("msh");
        output_file
    };

    data.write_gmsh(&output)
}
