use dose2gmsh::{parse_3ddose, Cli};
use structopt::StructOpt;

fn main() -> Result<(), std::io::Error> {
    let args = Cli::from_args();

    let dose_data = parse_3ddose(&args.input_file)?;

    let output = if let Some(output_file) = args.output_file {
        output_file
    } else {
        let mut output_file = args.input_file.clone();
        output_file.set_extension("msh");
        output_file
    };

    dose_data.write_gmsh(&output)
}
