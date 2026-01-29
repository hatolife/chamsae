//! ハングル変換CLIツール。

use anyhow::Result;
use clap::Parser;
use hangul_ime::hangul::HangulConverter;

/// コマンドライン引数。
#[derive(Parser, Debug)]
#[command(author, version, about = "ローマ字→ハングル変換")]
struct Args {
	/// 変換するローマ字。
	#[arg(short, long)]
	input: Option<String>,

	/// インタラクティブモード。
	#[arg(short = 'I', long)]
	interactive: bool,
}

fn main() -> Result<()> {
	let args = Args::parse();
	let converter = HangulConverter::new();

	if args.interactive {
		run_interactive(&converter)?;
	} else if let Some(input) = args.input {
		println!("{}", converter.convert(&input));
	} else {
		println!("Usage: hangul_cli -i <input> or hangul_cli -I");
	}

	Ok(())
}

/// インタラクティブモード。
fn run_interactive(converter: &HangulConverter) -> Result<()> {
	use std::io::{self, BufRead, Write};

	println!("ハングル変換 (exitで終了)");
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	loop {
		print!("> ");
		stdout.flush()?;

		let mut line = String::new();
		stdin.lock().read_line(&mut line)?;
		let input = line.trim();

		if input == "exit" {
			break;
		}

		println!("  → {}", converter.convert(input));
	}

	Ok(())
}
