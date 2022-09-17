use std::env;
use std::io;
use std::io::Write;
use std::process;

use treebender::rules::Grammar;
use treebender::Err;

fn usage(prog_name: &str) -> String {
  format!(
    r"Usage: {} FILE [options]

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

struct Args {
  filename: String,
  print_fs: bool,
  print_chart: bool,
}

impl Args {
  fn make_error_message(msg: &str, prog_name: impl AsRef<str>) -> String {
    format!("argument error: {}.\n\n{}", msg, usage(prog_name.as_ref()))
  }

  fn parse(v: Vec<String>) -> Result<Self, String> {
    if v.is_empty() {
      return Err(Self::make_error_message(
        "bad argument vector",
        "treebender",
      ));
    }

    let args_len = v.len();
    let mut iter = v.into_iter();
    let prog_name = iter.next().unwrap();

    if args_len < 2 {
      return Err(Self::make_error_message("not enough arguments", prog_name));
    }

    let mut filename: Option<String> = None;
    let mut print_fs = true; // default to printing feature structures
    let mut print_chart = false; // default to *not* printing the chart

    for o in iter {
      if o == "-h" || o == "--help" {
        println!("{}", usage(&prog_name));
        process::exit(0);
      } else if o == "-n" || o == "--no-fs" {
        print_fs = false;
      } else if o == "-c" || o == "--chart" {
        print_chart = true;
      } else if filename.is_none() {
        filename = Some(o);
      } else {
        return Err(Self::make_error_message("invalid arguments", prog_name));
      }
    }

    if let Some(filename) = filename {
      Ok(Self {
        filename,
        print_fs,
        print_chart,
      })
    } else {
      Err(Self::make_error_message("missing filename", prog_name))
    }
  }
}

fn main() -> Result<(), Err> {
  let opts = match Args::parse(env::args().collect()) {
    Ok(opts) => opts,
    Err(msg) => {
      eprintln!("{}", msg);
      process::exit(255);
    }
  };

  let g: Grammar = Grammar::read_from_file(&opts.filename)?;

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
        parse(&g, input.trim(), opts.print_chart, opts.print_fs)?;
        input.clear();
      }
      Err(error) => return Err(error.into()),
    }
  }
}
