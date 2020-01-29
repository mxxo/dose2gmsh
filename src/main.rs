use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(parse(from_os_str), short, long)]
    input_file: std::path::PathBuf,
    #[structopt(parse(from_os_str), short, long)]
    output_file: Option<std::path::PathBuf>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::from_args();
    let dose_input =
        BufReader::new(File::open(&args.input_file).expect("couldn't open input file"));

    let output = if let Some(output_file) = &args.output_file {
        output_file.clone()
    } else {
        let mut output_file = args.input_file.clone();
        output_file.set_extension("msh");
        output_file
    };

    let mut lines_iter = dose_input.lines().map(|l| l.unwrap());
    // first line is number of x, y, z voxels
    let (x_num, y_num, z_num) = {
        let voxel_nums = lines_iter.next().expect("voxel numbers");
        let voxel_nums: Vec<usize> = voxel_nums
            .trim()
            .split_whitespace()
            .map(|num| num.parse::<usize>().expect("voxel number"))
            .collect();
        assert!(voxel_nums.len() == 3);
        (voxel_nums[0], voxel_nums[1], voxel_nums[2])
    };

    // second line is x-coordinates
    let xs: Vec<f64> = {
        let xs = lines_iter.next().expect("x-coordinates");
        xs.trim()
            .split_whitespace()
            .map(|num| num.parse::<f64>().expect("x-coordinate"))
            .collect()
    };
    // x_num is number of cells, xs is number of cells + 1
    assert!(xs.len() == x_num + 1);

    // third line is y-coordinates
    let ys: Vec<f64> = {
        let ys = lines_iter.next().expect("y-coordinates");
        ys.trim()
            .split_whitespace()
            .map(|num| num.parse::<f64>().expect("y-coordinate"))
            .collect()
    };
    assert!(ys.len() == y_num + 1);

    // fourth is z-coordinates
    let zs: Vec<f64> = {
        let zs = lines_iter.next().expect("z-coordinates");
        zs.trim()
            .split_whitespace()
            .map(|num| num.parse::<f64>().expect("z-coordinate"))
            .collect()
    };
    assert!(zs.len() == z_num + 1);

    let num_voxels = x_num * y_num * z_num;
    let grid_index = |i, j, k| i + xs.len() * j + xs.len() * ys.len() * k;

    dbg!(grid_index(0, 0, 1));
    dbg!(grid_index(1, 0, 1));
    dbg!(grid_index(0, 1, 0));

    // fifth is deposited dose
    let doses: Vec<f64> = {
        let doses = lines_iter.next().expect("doses");
        doses
            .trim()
            .split_whitespace()
            .map(|num| num.parse::<f64>().expect("dose value"))
            .collect()
    };
    assert!(doses.len() == num_voxels);

    // sixth is uncertainties
    let uncerts: Vec<f64> = {
        let uncerts = lines_iter.next().expect("uncerts");
        uncerts
            .trim()
            .split_whitespace()
            .map(|num| num.parse::<f64>().expect("uncertainty value"))
            .collect()
    };
    assert!(uncerts.len() == doses.len());

    // output
    let mut filestream = BufWriter::new(File::create(output).unwrap());

    // gmsh header
    writeln!(&mut filestream, "$MeshFormat\n2.2 0 8\n$EndMeshFormat")?;
    // nodes
    write!(&mut filestream, "$Nodes\n{}\n", xs.len() * ys.len() * zs.len())?;
    for (k, z) in zs.iter().enumerate() {
        for (j, y) in ys.iter().enumerate() {
            for (i, x) in xs.iter().enumerate() {
                // gmsh expects 1-indexing
                writeln!(
                    &mut filestream,
                    "{} {} {} {}",
                    grid_index(i, j, k) + 1, x, y, z)?;
            }
        }
    }
    writeln!(&mut filestream, "$EndNodes")?;
    // elts
    // let indexes = |index| ((
    //
    //                        ));

    writeln!(&mut filestream, "$Elements\n{}", num_voxels)?;
        for index in 0..num_voxels {
            // 5 is a gmsh magic number for a hexahedron
            // 2 0 0 doesn't matter for us -- see gmsh doc for more
            // x_index =
            // x_coords = i
            let x_index = index + 1;
            let y_index = index + 1 + xs.len();
            let z_index = index + 1 + xs.len() * ys.len();
            writeln!(
                &mut filestream,
                "{} 5 2 0 0 {} {} {} {} {} {} {} {}",
                index + 1,
                x_index + 1,
                x_index,
                y_index,
                y_index + 1,
                z_index + 1,
                z_index,
                z_index + xs.len(),
                z_index + xs.len() + 1,
            )
            .unwrap();
        }
        writeln!(&mut filestream, "$EndElements").unwrap();



    println!("{:?}", num_voxels);
    println!("{:?}", xs[0]);
    // println!("{:?}", &args);
    // println!("{:?}", output);
    Ok(())
}
