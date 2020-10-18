use std::env;
use std::io;
use std::io::Write;
use std::process;

use treebender::rules::Grammar;
use treebender::Err;

fn usage(prog_name: &str) -> String {
  format!(
    r"
Usage: {} FILE [options]

Options:
  -h, --help    Print this message
  -c, --chart   Print the parse chart (defaults to not printing)
  -n, --no-fs   Don't print feature structures (defaults to printing)",
    prog_name
  )
}

fn parse(g: &Grammar, sentence: &str, print_chart: bool, print_fs: bool) -> Result<(), Err> {
  let sentence = sentence.split(' ').collect::<Vec<_>>();

  let chart = g.parse_chart(&sentence);

  if print_chart {
    println!("chart:\n{}\n", chart);
  }

  let trees = g.parse(&sentence);

  println!(
    "Parsed {} tree{}",
    trees.len(),
    if trees.len() == 1 { "" } else { "s" }
  );

  for (t, fs) in trees {
    println!("{}", t);
    if print_fs {
      println!("{}", fs);
    }
    println!();
  }

  Ok(())
}

fn main() -> Result<(), Err> {
  let opts: Vec<String> = env::args().collect();
  let prog_name = opts[0].clone();

  if opts.len() < 2 {
    println!("{}", usage(&prog_name));
    process::exit(1);
  }

  let mut opts = opts.into_iter().skip(1);
  let filename = opts.next().unwrap();

  let mut print_fs = true;      // default to printing feature structures
  let mut print_chart = false;  // default to *not* printing the chart
  for o in opts {
    if o == "-h" || o == "--help" {
      println!("{}", usage(&prog_name));
      process::exit(0);
    } else if o == "-n" || o == "--no-fs" {
      print_fs = false;
    } else if o == "-c" || o == "--chart" {
      print_chart = true;
    }
  }

  let g: Grammar = Grammar::read_from_file(&filename)?;

  let mut input = String::new();
  loop {
    print!("> ");
    io::stdout().flush()?;

    match io::stdin().read_line(&mut input) {
      Ok(_) => {
        if input.is_empty() {
          // ctrl+d
          return Ok(());
        }
        input.make_ascii_lowercase();
        parse(&g, &input.trim(), print_chart, print_fs)?;
        input.clear();
      }
      Err(error) => return Err(error.into()),
    }
  }
}
