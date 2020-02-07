use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(parse(from_os_str), short, long)]
    input_file: std::path::PathBuf,
    #[structopt(parse(from_os_str), short, long)]
    output_file: Option<std::path::PathBuf>,
}

fn parse_simple_line<T>(line: String, title: &'static str, expect_len: usize) -> Vec<T>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: Debug,
{
    let entries: Vec<T> = line
        .trim()
        .split_whitespace()
        .map(|num| num.parse::<T>().expect(title))
        .collect();
    assert!(entries.len() == expect_len);
    entries
}

fn main() -> Result<(), std::io::Error> {
    use itertools::Itertools;

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
    let (num_x, num_y, num_z) = {
        let voxel_nums = lines_iter.next().expect("voxel numbers");
        let voxel_nums = parse_simple_line::<usize>(voxel_nums, "voxel number", 3);
        (voxel_nums[0], voxel_nums[1], voxel_nums[2])
    };

    // second line is x-coordinates
    let xs = parse_simple_line::<f64>(
        lines_iter.next().expect("x-coordinates"),
        "x-coordinate",
        num_x + 1,
    );

    // third is y-coordinates
    let ys = parse_simple_line::<f64>(
        lines_iter.next().expect("y-coordinates"),
        "y-coordinate",
        num_y + 1,
    );

    let zs = parse_simple_line::<f64>(
        lines_iter.next().expect("z-coordinates"),
        "z-coordinate",
        num_z + 1,
    );

    let num_voxels = num_x * num_y * num_z;
    let grid_index = |i, j, k| i + xs.len() * j + xs.len() * ys.len() * k;

    dbg!(grid_index(0, 0, 1));
    dbg!(grid_index(1, 0, 1));
    dbg!(grid_index(0, 1, 0));

    // fifth is deposited dose
    let doses = parse_simple_line::<f64>(
        lines_iter.next().expect("doses"),
        "dose value",
        num_voxels
    );

    // sixth is uncertainty values
    let uncerts = parse_simple_line::<f64>(
        lines_iter.next().expect("uncerts"),
        "uncertainty value",
        num_voxels,
    );

    // output
    let mut filestream = BufWriter::new(File::create(output).unwrap());
    let num_nodes = xs.len() * ys.len() * zs.len();

    // gmsh header
    writeln!(&mut filestream, "$MeshFormat\n2.2 0 8\n$EndMeshFormat")?;
    // nodes
    write!(
        &mut filestream,
        "$Nodes\n{}\n",
        num_nodes,
    )?;
    for (k, z) in zs.iter().enumerate() {
        for (j, y) in ys.iter().enumerate() {
            for (i, x) in xs.iter().enumerate() {
                // gmsh expects 1-indexing
                writeln!(
                    &mut filestream,
                    "{} {} {} {}",
                    grid_index(i, j, k) + 1,
                    x,
                    y,
                    z
                )?;
            }
        }
    }
    writeln!(&mut filestream, "$EndNodes")?;

    // todo find exact len
    let mut x_nodes = 1..=num_nodes;
    let y_index = |x_index| x_index + xs.len();
    let z_index = |x_index| x_index + xs.len() * ys.len();

    dbg!(y_index(1));
    dbg!(z_index(1));

    let num_voxels = num_x * num_y * num_z;

    writeln!(&mut filestream, "$Elements\n{}", num_voxels)?;
    for index in 0..num_voxels {
        // we order nodes following the gmsh numbering
        // source: http://gmsh.info/doc/texinfo/gmsh.html#Low-order-elements
        //               v
        //        3----------2
        //        |\     ^   |\
        //        | \    |   | \
        //        |  \   |   |  \
        //        |   7------+---6
        //        |   |  +-- |-- | -> u
        //        0---+---\--1   |
        //         \  |    \  \  |
        //          \ |     \  \ |
        //           \|      w  \|
        //            4----------5
        //

        // skip rightmost node to start a new row
        if index != 0 && index % num_x == 0 {
            x_nodes = x_nodes.dropping(1);
        }

        // skip top row of nodes to move to the next x-y block
        if index != 0 && index % (num_x * num_y) == 0 {
            x_nodes = x_nodes.dropping(xs.len());
        }

        let xl = x_nodes.next().unwrap(); // 0 node
        let xr = xl + 1; // 1

        let yl = y_index(xl); // 3
        let yr = yl + 1; // 2

        let zl = z_index(xl); // 4
        let zr = zl + 1; // 5

        let yzl = z_index(yl); // 7
        let yzr = yzl + 1; // 6

        writeln!(
            &mut filestream,
            // 5 is the gmsh magic number for a hexahedron
            // 2 0 0 doesn't matter for us -- see element type section of
            // gmsh doc for more: http://gmsh.info/doc/texinfo/gmsh.html#MSH-file-format
            "{} 5 2 0 0 {} {} {} {} {} {} {} {}",
            index + 1,
            xl,  // 0
            xr,  // 1
            yr,  // 2
            yl,  // 3
            zl,  // 4
            zr,  // 5
            yzr, // 6
            yzl, // 7
        )
        .unwrap();

    }
    writeln!(&mut filestream, "$EndElements").unwrap();

    let mut write_elt_data = |name: &str, data: Vec<f64>| {
        writeln!(&mut filestream, "$ElementData").unwrap();
        // one string - the field name
        writeln!(&mut filestream, "1\n{}", name).unwrap();
        // one real value - the time
        writeln!(&mut filestream, "1\n0.0").unwrap();
        // three int tags
        //   timestep 0
        //   1-component (scalar) field
        //   num_elt values
        writeln!(&mut filestream, "3\n0\n1\n{}", data.len()).unwrap();
        for (index, val) in data.iter().enumerate() {
            writeln!(&mut filestream, "{} {}", index + 1, val).unwrap();
        }
        writeln!(&mut filestream, "$EndElementData").unwrap();
    };

    write_elt_data(r#""Dose""#, doses);
    write_elt_data(r#""Uncertainty""#, uncerts);

    println!("{:?}", num_voxels);
    println!("{:?}", xs[0]);
    // println!("{:?}", &args);
    // println!("{:?}", output);
    Ok(())
}
