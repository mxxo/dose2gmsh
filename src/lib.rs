//! Convert `3ddose` files to Gmsh `msh` (version 2) files.
//!
//! Get started with `cargo install dose2gmsh`.

use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;

use structopt::StructOpt;

/// Command line input parameters.
#[derive(StructOpt, Debug)]
#[structopt(name = "dose2gmsh", author = "Max Orok <morok@mevex.com>", about = "Convert dosxyznrc 3ddose files to Gmsh msh files")]
pub struct Cli {
    /// The input 3ddose file
    #[structopt(parse(from_os_str), short, long)]
    pub input_file: std::path::PathBuf,
    /// The output file name, defaults to <input_file>.msh
    #[structopt(parse(from_os_str), short, long)]
    pub output_file: Option<std::path::PathBuf>,
}

/// Dose and uncertainty data for a 3D rectilinear hexahedral mesh.
#[derive(Debug, Clone)]
pub struct DoseBlock {
    /// Node coordinates along *x* in `[cm]`.
    pub xs: Vec<f64>,
    /// Node coordinates along *y* in `[cm]`.
    pub ys: Vec<f64>,
    /// Node coordinates along *z* in `[cm]`.
    pub zs: Vec<f64>,
    /// Unnormalized voxel dose per fluence data in `[Gy · cm2]`.
    pub doses: Vec<f64>,
    /// Fractional dose uncertainties.
    pub uncerts: Vec<f64>,
}

// add more output formats here
// enum OutputFormat {
//     Gmsh2,
// }

impl DoseBlock {
    /// Create a new `DoseBlock` by parsing a `3ddose` data file.
    pub fn from_3d_dose(input_file: &std::path::PathBuf) -> Result<DoseBlock, std::io::Error> {
        let dose_input = BufReader::new(File::open(input_file)?);

        let mut lines = dose_input.lines().map(|l| l.unwrap());
        // first line is number of x, y, z voxels
        let (num_x, num_y, num_z) = {
            let voxel_nums = lines.next().expect("voxel numbers");
            let voxel_nums = parse_simple_line::<usize>(voxel_nums, "voxel number", 3);
            (voxel_nums[0], voxel_nums[1], voxel_nums[2])
        };

        // second line is x-coordinates
        let xs = parse_simple_line::<f64>(
            lines.next().expect("x-coordinates"),
            "x-coordinate",
            num_x + 1,
        );

        // third is y-coordinates
        let ys = parse_simple_line::<f64>(
            lines.next().expect("y-coordinates"),
            "y-coordinate",
            num_y + 1,
        );

        // fourth is z-coordinates
        let zs = parse_simple_line::<f64>(
            lines.next().expect("z-coordinates"),
            "z-coordinate",
            num_z + 1,
        );

        let num_voxels = num_x * num_y * num_z;

        // fifth is deposited dose
        let doses = parse_simple_line::<f64>(lines.next().expect("doses"), "dose value", num_voxels);

        // sixth is uncertainty values
        let uncerts = parse_simple_line::<f64>(
            lines.next().expect("uncerts"),
            "uncertainty value",
            num_voxels,
        );

        Ok(DoseBlock {
            xs,
            ys,
            zs,
            doses,
            uncerts,
        })

    }

    /// Number of voxels in the *x*-direction.
    pub fn num_x(&self) -> usize {
        self.xs.len() - 1
    }

    /// Number of voxels in the *y*-direction.
    pub fn num_y(&self) -> usize {
        self.ys.len() - 1
    }

    /// Number of voxels in the *z*-direction.
    pub fn num_z(&self) -> usize {
        self.zs.len() - 1
    }

    /// Total number of mesh voxels.
    pub fn num_voxels(&self) -> usize {
        self.num_x() * self.num_y() * self.num_z()
    }

    /// Total number of mesh nodes.
    pub fn num_nodes(&self) -> usize {
        self.xs.len() * self.ys.len() * self.zs.len()
    }

    /// `[i, j, k]` node list indexing.
    pub fn grid_index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.xs.len() * j + self.xs.len() * self.ys.len() * k
    }

    /// Convert the `3ddose` data to a Gmsh `.msh` file (version 2.2).
    pub fn write_gmsh(&self, output: &std::path::PathBuf) -> Result<(), std::io::Error> {
        use itertools::Itertools;

        let mut filestream = BufWriter::new(File::create(output)?);

        // gmsh header
        writeln!(&mut filestream, "$MeshFormat\n2.2 0 8\n$EndMeshFormat")?;
        // nodes
        write!(&mut filestream, "$Nodes\n{}\n", self.num_nodes())?;
        for (k, z) in self.zs.iter().enumerate() {
            for (j, y) in self.ys.iter().enumerate() {
                for (i, x) in self.xs.iter().enumerate() {
                    // gmsh expects 1-indexing
                    writeln!(
                        &mut filestream,
                        "{} {} {} {}",
                        self.grid_index(i, j, k) + 1,
                        x,
                        y,
                        z
                    )?;
                }
            }
        }
        writeln!(&mut filestream, "$EndNodes")?;

        // todo find exact len
        let mut x_nodes = 1..=self.num_nodes();
        let gmsh_y_index = |x_index| x_index + self.xs.len();
        let gmsh_z_index = |x_index| x_index + self.xs.len() * self.ys.len();

        writeln!(&mut filestream, "$Elements\n{}", self.num_voxels())?;
        for index in 0..self.num_voxels() {
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
            if index != 0 && index % self.num_x() == 0 {
                x_nodes = x_nodes.dropping(1);
            }

            // skip top row of nodes to move to the next x-y block
            if index != 0 && index % (self.num_x() * self.num_y()) == 0 {
                x_nodes = x_nodes.dropping(self.xs.len());
            }

            let xl = x_nodes.next().unwrap(); // 0 node
            let xr = xl + 1; // 1

            let yl = gmsh_y_index(xl); // 3
            let yr = yl + 1; // 2

            let zl = gmsh_z_index(xl); // 4
            let zr = zl + 1; // 5

            let yzl = gmsh_z_index(yl); // 7
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
            )?;
        }
        writeln!(&mut filestream, "$EndElements")?;

        let mut write_elt_data = |name: &str, data: &Vec<f64>| -> Result<(), std::io::Error> {
            writeln!(&mut filestream, "$ElementData")?;
            // one string - the field name
            writeln!(&mut filestream, "1\n{}", name)?;
            // one real value - the time
            writeln!(&mut filestream, "1\n0.0")?;
            // three int tags
            //   timestep 0
            //   1-component (scalar) field
            //   num_elt values
            writeln!(&mut filestream, "3\n0\n1\n{}", data.len())?;
            for (index, val) in data.iter().enumerate() {
                writeln!(&mut filestream, "{} {}", index + 1, val)?;
            }
            writeln!(&mut filestream, "$EndElementData")?;
            Ok(())
        };

        write_elt_data(r#""Dose [Gy·cm2]""#, &self.doses)?;
        write_elt_data(r#""Uncertainty fraction""#, &self.uncerts)
    }
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
