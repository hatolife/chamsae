//! ハングル変換CLIツール。

use anyhow::Result;
use clap::Parser;
use chamsae::config::Config;
use chamsae::hangul::HangulConverter;

/// コマンドライン引数。
#[derive(Parser, Debug)]
#[command(author, about = "ローマ字→ハングル変換", disable_version_flag = true)]
struct Args {
	/// 変換するローマ字。
	#[arg(short, long)]
	input: Option<String>,

	/// インタラクティブモード。
	#[arg(short = 'I', long)]
	interactive: bool,

	/// 設定ファイルのテンプレートをカレントディレクトリに生成。
	#[arg(short = 't', long = "template")]
	template: bool,

	/// バージョン表示。
	#[arg(short = 'v', long = "version")]
	version: bool,
}

fn main() -> Result<()> {
	let args = Args::parse();

	if args.version {
		println!("chamsae {}", env!("CARGO_PKG_VERSION"));
		return Ok(());
	} else if args.template {
		generate_template()?;
	} else if args.interactive {
		let converter = HangulConverter::new();
		run_interactive(&converter)?;
	} else if let Some(input) = args.input {
		let converter = HangulConverter::new();
		println!("{}", converter.convert(&input));
	} else {
		let converter = HangulConverter::new();
		run_stdin(&converter)?;
	}

	Ok(())
}

/// 標準入力から読み込んで変換する。
fn run_stdin(converter: &HangulConverter) -> Result<()> {
	use std::io::{self, BufRead};

	let stdin = io::stdin();
	for line in stdin.lock().lines() {
		let line = line?;
		println!("{}", converter.convert(&line));
	}

	Ok(())
}

/// 設定ファイルのテンプレートをカレントディレクトリに生成する。
fn generate_template() -> Result<()> {
	let path = std::env::current_dir()?.join("chamsae.json");
	if path.exists() {
		println!("chamsae.json は既に存在します: {}", path.display());
		return Ok(());
	}
	Config::load(&std::env::current_dir()?);
	println!("chamsae.json を生成しました: {}", path.display());
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
