#[macro_use]
extern crate clap;
extern crate gbasm;

fn main() {
    
    let args = clap::App::new("gbasm")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("A rust based GameBoy Assembler")
        .arg(clap::Arg::with_name("sources")
            .help("Assembly source files")
            .index(1)
            .multiple(true)
        )
        .arg(clap::Arg::with_name("outfile")
            .help("The name of the output rom image (default is \"game.gb\", use \"stdout\" to directly write rom contents to standard out)")
            .short("o")
            .takes_value(true)
        )
        .arg(clap::Arg::with_name("optimize")
            .help("Enable basic instruction optimizations")
            .short("O")
            .long("optimize")
        )
        .arg(clap::Arg::with_name("unsafe")
            .help("Enable unsafe optimizations")
            .long("optimize-unsafe")
            .requires("optimize")
        )
        .arg(clap::Arg::with_name("mapfile")
            .help("Generates a ASCII overview of the mapped ROM space")
            .short("m")
            .takes_value(true)
        )
        .arg(clap::Arg::with_name("symfile")
            .help("Generates a symbol map compatible with debuggers")
            .short("s")
            .takes_value(true)
        )
        .arg(clap::Arg::with_name("jsonfile")
            .help("Generates a JSON data dump of all sections with their data, labels, instructions etc")
            .short("j")
            .takes_value(true)
        )
        .arg(clap::Arg::with_name("silent")
            .help("Surpresses all logging")
            .long("silent")
        )
        .arg(clap::Arg::with_name("unused")
            .help("Report unused labels and variables")
            .long("report-unused")
        )
        .arg(clap::Arg::with_name("verbose")
            .help("Provide increased logging")
            .long("verbose")

        ).get_matches();


    match args.values_of("sources") {

        Some(ref sources) => {

            let mut c = gbasm::Compiler::new(
                args.is_present("silent"),
                args.is_present("verbose")
            );

            // Compile Source Files
            if let Err(message) = c.compile_source_files(sources, !args.is_present("optimize")) {
                use std::io::{Write, stderr};
                writeln!(&mut stderr(), "Compilation error: {}", message).ok();
                std::process::exit(1);
            }

            // Apply optimizations
            if args.is_present("optimize") {
                c.optimize_instructions(args.is_present("unsafe"));
            }

            // Report unused variables
            if args.is_present("unused") {
                println!("{}", c.report_unsused());
            }

            // Generate ROM image
            c.generate_rom_image(args.value_of("outfile").unwrap_or("game.gb"));

            // Generates symbol file
            if let Some(ref symfile) = args.value_of("symfile") {
                c.generate_symbol_file(symfile);
            }
            
            // Generate mapping file
            if let Some(ref mapfile) = args.value_of("mapfile") {
                c.generate_mapping_file(mapfile);
            }
            
            // Generate json file
            if let Some(ref jsonfile) = args.value_of("jsonfile") {
                c.generate_json_file(jsonfile);
            }

        }, 

        None => {
            println!("{}", args.usage());
        }

    }

}

