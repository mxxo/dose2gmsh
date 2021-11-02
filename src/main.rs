use dose2gmsh::{Cli, DoseBlock, Fmt};
use structopt::StructOpt;

fn main() -> Result<(), std::io::Error> {
    let args = Cli::from_args();
    match args.input_file.extension() {
        Some(ext) if ext == "3ddose" => {},
        _ => eprintln!("warning: input file does not have 3ddose extension"),
    }
    let data = DoseBlock::from_3d_dose(&args.input_file)?;

    let mut output_name = match args.output_file {
        Some(name) => name,
        None => args.input_file.clone(),
    };

    match args.format {
        Fmt::Csv => {
            output_name.set_extension("csv");
            data.write_csv(&output_name)
        },
        Fmt::Msh2 => {
            output_name.set_extension("msh");
            data.write_msh2(&output_name)
        }
        Fmt::Vtk => {
            output_name.set_extension("vtk");
            data.write_vtk(&output_name)
        }
    }
}
