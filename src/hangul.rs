//! ハングル変換モジュール。

use std::collections::HashMap;

/// ハングル変換器。
pub struct HangulConverter {
	/// 初声(子音)マッピング。
	choseong: HashMap<&'static str, u32>,
	/// 中声(母音)マッピング。
	jungseong: HashMap<&'static str, u32>,
	/// 終声(パッチム)マッピング。
	jongseong: HashMap<&'static str, u32>,
}

impl HangulConverter {
	/// 新しい変換器を作成する。
	pub fn new() -> Self {
		Self {
			choseong: Self::build_choseong(),
			jungseong: Self::build_jungseong(),
			jongseong: Self::build_jongseong(),
		}
	}

	/// 初声マッピングを構築する。
	fn build_choseong() -> HashMap<&'static str, u32> {
		[
			("g", 0), ("gg", 1),("kk", 1), ("n", 2), ("d", 3), ("dd", 4), ("tt", 4),
			("r", 5), ("l", 5), ("m", 6), ("b", 7), ("bb", 8), ("pp", 8),
			("s", 9), ("ss", 10), ("j", 12), ("jj", 13),
			("ch", 14), ("k", 15), ("t", 16), ("p", 17), ("h", 18),
		]
			.into_iter()
			.collect()
	}

	/// 中声マッピングを構築する。
	fn build_jungseong() -> HashMap<&'static str, u32> {
		[
			("a", 0), ("ae", 1), ("ya", 2), ("yae", 3), ("eo", 4),
			("e", 5), ("yeo", 6), ("ye", 7), ("o", 8), ("wa", 9),
			("wae", 10), ("oe", 11), ("yo", 12), ("u", 13), ("wo", 14),
			("we", 15), ("wi", 16), ("yu", 17), ("eu", 18), ("ui", 19),
			("i", 20),
		]
			.into_iter()
			.collect()
	}

	/// 終声マッピングを構築する。
	fn build_jongseong() -> HashMap<&'static str, u32> {
		[
			("g", 1), ("gg", 2), ("gs", 3), ("n", 4),
			("nj", 5), ("nh", 6), ("d", 7), ("l", 8), ("lg", 9),
			("lm", 10), ("lb", 11), ("ls", 12), ("lt", 13), ("lp", 14),
			("lh", 15), ("m", 16), ("b", 17), ("bs", 18), ("s", 19),
			("ss", 20), ("ng", 21), ("j", 22), ("ch", 23), ("k", 24),
			("t", 25), ("p", 26), ("h", 27),
		]
			.into_iter()
			.collect()
	}

	/// ローマ字をハングルに変換する。
	pub fn convert(&self, input: &str) -> String {
		let input = input.to_lowercase();
		let mut result = String::new();
		let mut chars = input.chars().peekable();
		let mut current_syllable = String::new();

		while let Some(c) = chars.next() {
			if c == ' ' {
				// スペースの連続をカウント。
				let mut space_count = 1;
				while chars.peek() == Some(&' ') {
					chars.next();
					space_count += 1;
				}

				// 現在の音節を変換。
				if !current_syllable.is_empty() {
					result.push_str(&self.convert_syllable(&current_syllable));
					current_syllable.clear();
				}

				// スペース2つごとに実際のスペースを出力。
				for _ in 0..(space_count / 2) {
					result.push(' ');
				}
			} else {
				current_syllable.push(c);
			}
		}

		// 最後の音節を変換。
		if !current_syllable.is_empty() {
			result.push_str(&self.convert_syllable(&current_syllable));
		}

		result
	}

	/// 単一の音節群を変換する (区切りなし)。
	fn convert_syllable(&self, input: &str) -> String {
		let chars: Vec<char> = input.chars().collect();
		let mut result = String::new();
		let mut pos = 0;

		while pos < chars.len() {
			// 初声を探す。
			match self.find_choseong(&chars, pos) {
				Some((cho_idx, cho_len)) => {
					pos += cho_len;

					// 中声を探す。
					match self.find_jungseong(&chars, pos) {
						Some((jung_idx, jung_len)) => {
							pos += jung_len;

							// 終声を探す(次の音節との境界判定)。
							let jong_idx = self.find_jongseong_with_lookahead(&chars, &mut pos);

							// ハングル文字を合成。
							result.push(self.compose(cho_idx, jung_idx, jong_idx));
						}
						None => {
							// 中声がない場合、子音をそのまま出力。
							for i in 0..cho_len {
								result.push(chars[pos - cho_len + i]);
							}
						}
					}
				}
				None => {
					// 母音から始まる場合(初声 = ㅇ)。
					if let Some((jung_idx, jung_len)) = self.find_jungseong(&chars, pos) {
						pos += jung_len;
						let jong_idx = self.find_jongseong_with_lookahead(&chars, &mut pos);
						// 11 = ㅇ (無音の初声)。
						result.push(self.compose(11, jung_idx, jong_idx));
					} else {
						// マッチしない文字はそのまま。
						result.push(chars[pos]);
						pos += 1;
					}
				}
			}
		}

		result
	}

	/// 初声を検索する(最長一致)。
	fn find_choseong(&self, chars: &[char], pos: usize) -> Option<(u32, usize)> {
		self.find_longest_match(chars, pos, &self.choseong)
	}

	/// 中声を検索する(最長一致)。
	fn find_jungseong(&self, chars: &[char], pos: usize) -> Option<(u32, usize)> {
		self.find_longest_match(chars, pos, &self.jungseong)
	}

	/// 終声を検索する(次の音節を考慮)。
	fn find_jongseong_with_lookahead(&self, chars: &[char], pos: &mut usize) -> u32 {
		for len in (1..=2).rev() {
			if *pos + len > chars.len() {
				continue;
			}

			let substr: String = chars[*pos..*pos + len].iter().collect();
			if let Some(&jong_idx) = self.jongseong.get(substr.as_str()) {
				let next_pos = *pos + len;

				if next_pos >= chars.len() {
					// 文末 → 終声採用。
					*pos = next_pos;
					return jong_idx;
				}

				// 次に子音があるかチェック。
				if let Some((_, cho_len)) = self.find_longest_match(chars, next_pos, &self.choseong) {
					if self.find_jungseong(chars, next_pos + cho_len).is_some() {
						// 子音+母音パターン → 終声採用 (子音が次の初声になる)。
						*pos = next_pos;
						return jong_idx;
					}
					// 子音のみ → 終声採用。
					*pos = next_pos;
					return jong_idx;
				}

				// 次が母音のみ → より短い終声を試す。
				if self.find_jungseong(chars, next_pos).is_some() {
					continue;
				}

				// その他 (マッチしない文字) → 終声採用。
				*pos = next_pos;
				return jong_idx;
			}
		}

		// 終声なし。
		0
	}

	/// マップから最長一致で検索する。
	fn find_longest_match(
		&self,
		chars: &[char],
		pos: usize,
		map: &HashMap<&'static str, u32>,
	) -> Option<(u32, usize)> {
		// 長い方から試す(最長一致)。
		for len in (1..=3).rev() {
			if pos + len > chars.len() {
				continue;
			}

			let substr: String = chars[pos..pos + len].iter().collect();
			if let Some(&idx) = map.get(substr.as_str()) {
				return Some((idx, len));
			}
		}

		None
	}

	/// 初声・中声・終声からハングル文字を合成する。
	fn compose(&self, cho: u32, jung: u32, jong: u32) -> char {
		let code = 0xAC00 + (cho * 21 + jung) * 28 + jong;
		char::from_u32(code).unwrap_or('?')
	}
}

impl Default for HangulConverter {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 初声 (子音) テスト ====================

    #[test]
    fn test_choseong_basic() {
        let c = HangulConverter::new();

        // 基本子音。
        assert_eq!(c.convert("ga"), "가");
        assert_eq!(c.convert("na"), "나");
        assert_eq!(c.convert("da"), "다");
        assert_eq!(c.convert("ra"), "라");
        assert_eq!(c.convert("la"), "라");  // l = r
        assert_eq!(c.convert("ma"), "마");
        assert_eq!(c.convert("ba"), "바");
        assert_eq!(c.convert("sa"), "사");
        assert_eq!(c.convert("ja"), "자");
        assert_eq!(c.convert("ha"), "하");
    }

    #[test]
    fn test_choseong_aspirated() {
        let c = HangulConverter::new();

        // 激音 (거센소리)。
        assert_eq!(c.convert("ka"), "카");
        assert_eq!(c.convert("ta"), "타");
        assert_eq!(c.convert("pa"), "파");
        assert_eq!(c.convert("cha"), "차");
    }

    #[test]
    fn test_choseong_tense() {
        let c = HangulConverter::new();

        // 濃音 (된소리)。
        assert_eq!(c.convert("kka"), "까");
        assert_eq!(c.convert("gga"), "까");
        assert_eq!(c.convert("dda"), "따");
        assert_eq!(c.convert("tta"), "따");
        assert_eq!(c.convert("bba"), "빠");
        assert_eq!(c.convert("ppa"), "빠");
        assert_eq!(c.convert("ssa"), "싸");
        assert_eq!(c.convert("jja"), "짜");
    }

    // ==================== 中声 (母音) テスト ====================

    #[test]
    fn test_jungseong_basic() {
        let c = HangulConverter::new();

        // 基本母音。
        assert_eq!(c.convert("a"), "아");
        assert_eq!(c.convert("eo"), "어");
        assert_eq!(c.convert("o"), "오");
        assert_eq!(c.convert("u"), "우");
        assert_eq!(c.convert("eu"), "으");
        assert_eq!(c.convert("i"), "이");
    }

    #[test]
    fn test_jungseong_compound_ae() {
        let c = HangulConverter::new();

        // 애, 에 系。
        assert_eq!(c.convert("ae"), "애");
        assert_eq!(c.convert("e"), "에");
        assert_eq!(c.convert("yae"), "얘");
        assert_eq!(c.convert("ye"), "예");
    }

    #[test]
    fn test_jungseong_y_glide() {
        let c = HangulConverter::new();

        // ヤ行母音。
        assert_eq!(c.convert("ya"), "야");
        assert_eq!(c.convert("yeo"), "여");
        assert_eq!(c.convert("yo"), "요");
        assert_eq!(c.convert("yu"), "유");
    }

    #[test]
    fn test_jungseong_w_glide() {
        let c = HangulConverter::new();

        // ワ行母音。
        assert_eq!(c.convert("wa"), "와");
        assert_eq!(c.convert("wae"), "왜");
        assert_eq!(c.convert("wo"), "워");
        assert_eq!(c.convert("we"), "웨");
        assert_eq!(c.convert("wi"), "위");
    }

    #[test]
    fn test_jungseong_special() {
        let c = HangulConverter::new();

        // 特殊母音。
        assert_eq!(c.convert("oe"), "외");
        assert_eq!(c.convert("ui"), "의");
    }

    // ==================== 終声 (パッチム) テスト ====================

    #[test]
    fn test_jongseong_basic() {
        let c = HangulConverter::new();

        // 基本終声。
        assert_eq!(c.convert("gag"), "각");
        assert_eq!(c.convert("gan"), "간");
        assert_eq!(c.convert("gad"), "갇");
        assert_eq!(c.convert("gal"), "갈");
        assert_eq!(c.convert("gam"), "감");
        assert_eq!(c.convert("gab"), "갑");
        assert_eq!(c.convert("gas"), "갓");
        assert_eq!(c.convert("gang"), "강");
        assert_eq!(c.convert("gaj"), "갖");
        assert_eq!(c.convert("gak"), "갘");
        assert_eq!(c.convert("gat"), "같");
        assert_eq!(c.convert("gap"), "갚");
        assert_eq!(c.convert("gah"), "갛");
    }

	#[test]
	fn test_jongseong_double() {
		let c = HangulConverter::new();

		// 二重終声。
		assert_eq!(c.convert("gags"), "갃");   // gs = 3
		assert_eq!(c.convert("ganj"), "갅");   // nj = 5
		assert_eq!(c.convert("ganh"), "갆");   // nh = 6
		assert_eq!(c.convert("galg"), "갉");   // lg = 9
		assert_eq!(c.convert("galm"), "갊");   // lm = 10
		assert_eq!(c.convert("galb"), "갋");   // lb = 11
		assert_eq!(c.convert("gals"), "갌");   // ls = 12
		assert_eq!(c.convert("galt"), "갍");   // lt = 13
		assert_eq!(c.convert("galp"), "갎");   // lp = 14
		assert_eq!(c.convert("galh"), "갏");   // lh = 15
		assert_eq!(c.convert("gabs"), "값");   // bs = 18 ← 修正
	}

    #[test]
    fn test_jongseong_tense() {
        let c = HangulConverter::new();

        // 濃音終声。
        assert_eq!(c.convert("gagg"), "갂");
        assert_eq!(c.convert("gass"), "갔");
    }

    // ==================== 音節組み合わせテスト ====================

    #[test]
    fn test_syllable_boundary() {
        let c = HangulConverter::new();

        // 音節境界の処理。
        assert_eq!(c.convert("han gug"), "한국");
        assert_eq!(c.convert("han gug eo"), "한국어");
        assert_eq!(c.convert("gug eo"), "국어");
    }

    #[test]
    fn test_words_common() {
        let c = HangulConverter::new();

        // よく使う単語。
        assert_eq!(c.convert("an nyeong"), "안녕");
        assert_eq!(c.convert("gam sa hab ni da"), "감사합니다");
        assert_eq!(c.convert("sa rang"), "사랑");
        assert_eq!(c.convert("chin gu"), "친구");
        assert_eq!(c.convert("hag gyo"), "학교");
        assert_eq!(c.convert("seon saeng nim"), "선생님");
    }

	#[test]
	fn test_words_with_tense() {
		let c = HangulConverter::new();

		// 濃音を含む単語。
		assert_eq!(c.convert("a bba"), "아빠");
		// 엄마 = eom + ma (ㅓ+ㅁ の終声m)
		assert_eq!(c.convert("eom ma"), "엄마");
		assert_eq!(c.convert("bbang"), "빵");
	}

    // ==================== スペース処理テスト ====================

    #[test]
    fn test_space_delimiter() {
        let c = HangulConverter::new();

        // 単一スペース = 区切り。
        assert_eq!(c.convert("na neun"), "나는");
        assert_eq!(c.convert("neo neun"), "너는");
    }

    #[test]
    fn test_space_literal() {
        let c = HangulConverter::new();

        // ダブルスペース = 実際のスペース。
        assert_eq!(c.convert("an nyeong  ha se yo"), "안녕 하세요");
        assert_eq!(c.convert("na neun  hag saeng  ib ni da"), "나는 학생 입니다");
    }

	#[test]
	fn test_space_multiple() {
		let c = HangulConverter::new();

		// 複数スペース。
		// "b" は子音のみなのでそのまま出力。
		assert_eq!(c.convert("a   beu"), "아 브");   // 3 = 1スペース + 区切り
		assert_eq!(c.convert("a    beu"), "아  브"); // 4 = 2スペース
	}

    // ==================== エッジケーステスト ====================

    #[test]
    fn test_edge_empty() {
        let c = HangulConverter::new();

        // 空文字列。
        assert_eq!(c.convert(""), "");
    }

	#[test]
	fn test_edge_passthrough() {
		let c = HangulConverter::new();

		// 変換できない文字はそのまま。
		assert_eq!(c.convert("123"), "123");
		assert_eq!(c.convert("!@#"), "!@#");
		// f, c, x, z などはハングルにマッピングなし。
		assert_eq!(c.convert("f"), "f");
		assert_eq!(c.convert("x"), "x");
		assert_eq!(c.convert("z"), "z");
	}

	#[test]
	fn test_edge_mixed() {
		let c = HangulConverter::new();

		// 混合入力。
		assert_eq!(c.convert("han gug 123"), "한국123");
		// test: te(테) + s(終声) + t(残り) = 텟t
		assert_eq!(c.convert("test ga na da"), "텟t가나다");
	}

    #[test]
    fn test_edge_case_sensitivity() {
        let c = HangulConverter::new();

        // 大文字小文字。
        assert_eq!(c.convert("GA"), "가");
        assert_eq!(c.convert("HAN GUG"), "한국");
    }

	// ==================== 連音化テスト ====================

	#[test]
	fn test_liaison() {
		let c = HangulConverter::new();

		// 終声 + 母音 → 終声が次の初声に移動しない (区切りで明示)
		// 국어: gug + eo (区切りあり)
		assert_eq!(c.convert("gug eo"), "국어");
		
		// 区切りなしの場合の動作確認
		assert_eq!(c.convert("gugeo"), "구거");  // 曖昧性により
		
		// 連音化が必要なケース
		assert_eq!(c.convert("dog ib"), "독입");  // 독립の一部
		assert_eq!(c.convert("hab ni da"), "합니다");
	}

	// ==================== 複合パターンテスト ====================

	#[test]
	fn test_complex_combinations() {
		let c = HangulConverter::new();

		// 濃音 + 複合母音
		assert_eq!(c.convert("ssya"), "쌰");
		assert_eq!(c.convert("bbyeo"), "뼈");
		
		// 激音 + 複合母音
		assert_eq!(c.convert("chwa"), "촤");
		assert_eq!(c.convert("kwe"), "퀘");
		
		// 複合母音 + 二重終声
		assert_eq!(c.convert("walg"), "왉");
		assert_eq!(c.convert("oelm"), "욂");
	}

	#[test]
	fn test_continuous_syllables() {
		let c = HangulConverter::new();

		// 3音節以上の連続
		assert_eq!(c.convert("ga na da"), "가나다");
		assert_eq!(c.convert("a ya eo yeo"), "아야어여");
		assert_eq!(c.convert("gag nan dam"), "각난담");
	}

	// ==================== 境界ケーステスト ====================

	#[test]
	fn test_boundary_vowel_sequences() {
		let c = HangulConverter::new();

		// 母音のみの連続 (各音節は ㅇ+母音)
		assert_eq!(c.convert("a i u"), "아이우");
		assert_eq!(c.convert("o eu i"), "오으이");
	}

	#[test]
	fn test_boundary_consonant_only() {
		let c = HangulConverter::new();

		// 子音のみ (母音がないのでそのまま出力)
		assert_eq!(c.convert("g"), "g");
		assert_eq!(c.convert("ng"), "ng");
		assert_eq!(c.convert("gg"), "gg");
		assert_eq!(c.convert("kk"), "kk");
		assert_eq!(c.convert("jj"), "jj");
		assert_eq!(c.convert("bb"), "bb");
		assert_eq!(c.convert("pp"), "pp");
		assert_eq!(c.convert("dd"), "dd");
		assert_eq!(c.convert("tt"), "tt");
		assert_eq!(c.convert("ss"), "ss");
	}

	#[test]
	fn test_boundary_repeated_chars() {
		let c = HangulConverter::new();

		// 同じ文字の連続
		assert_eq!(c.convert("aaa"), "아아아");  // 要確認
		assert_eq!(c.convert("gaga"), "가가");
	}

	// ==================== 実用単語テスト ====================

	#[test]
	fn test_practical_words() {
		let c = HangulConverter::new();

		// 挨拶
		assert_eq!(c.convert("an nyeong ha se yo"), "안녕하세요");
		assert_eq!(c.convert("an nyeong hi ga se yo"), "안녕히가세요");
		assert_eq!(c.convert("an nyeong hi gye se yo"), "안녕히계세요");
		
		// 数字
		assert_eq!(c.convert("il i sam sa o"), "일이삼사오");
		assert_eq!(c.convert("yug chil pal gu sib"), "육칠팔구십");
		
		// 曜日
		assert_eq!(c.convert("wol hwa su mog geum to il"), "월화수목금토일");
	}

	#[test]
	fn test_practical_phrases() {
		let c = HangulConverter::new();

		// フレーズ (ダブルスペースで単語区切り)
		assert_eq!(c.convert("jeo neun  il bon  sa ram  ib ni da"), "저는 일본 사람 입니다");
		assert_eq!(c.convert("man na seo  ban gab seub ni da"), "만나서 반갑습니다");
	}

	// ==================== 長文テスト ====================

	#[test]
	fn test_long_input() {
		let c = HangulConverter::new();

		// パフォーマンス確認用 (正確性より完走するか)
		let input = "ga na da la ma ba sa a ja cha ka ta pa ha";
		let output = c.convert(input);
		assert_eq!(output.chars().count(), 14);  // 14文字のハングル
	}

	// ==================== 入力バリエーション ====================

	#[test]
	fn test_input_variations() {
		let c = HangulConverter::new();

		// 大文字小文字混在
		assert_eq!(c.convert("HaN GuG"), "한국");
		assert_eq!(c.convert("ANNYEONG"), "안녕");
		
		// 先頭・末尾のスペース
		assert_eq!(c.convert(" ga"), "가");
		assert_eq!(c.convert("ga "), "가");
		assert_eq!(c.convert("  ga"), " 가");  // 2スペース = 1スペース出力
	}

	// ==================== 回帰テスト ====================

	#[test]
	fn test_regression_known_issues() {
		let c = HangulConverter::new();

		// 過去に問題があったケース
		assert_eq!(c.convert("han gug eo"), "한국어");  // 音節境界
		assert_eq!(c.convert("gam sa hab ni da"), "감사합니다");  // 終声b
		assert_eq!(c.convert("eom ma"), "엄마");  // 終声m + 初声m
	}

	// ==================== 完全網羅テスト ====================

	#[test]
	fn test_choseong_complete() {
		let c = HangulConverter::new();

		// 全19種類の初声 (母音 a で統一)
		assert_eq!(c.convert("ga"), "가");    // 0: ㄱ
		assert_eq!(c.convert("gga"), "까");   // 1: ㄲ
		assert_eq!(c.convert("kka"), "까");   // 1: ㄲ
		assert_eq!(c.convert("na"), "나");    // 2: ㄴ
		assert_eq!(c.convert("da"), "다");    // 3: ㄷ
		assert_eq!(c.convert("dda"), "따");   // 4: ㄸ
		assert_eq!(c.convert("tta"), "따");   // 4: ㄸ
		assert_eq!(c.convert("ra"), "라");    // 5: ㄹ
		assert_eq!(c.convert("ma"), "마");    // 6: ㅁ
		assert_eq!(c.convert("ba"), "바");    // 7: ㅂ
		assert_eq!(c.convert("bba"), "빠");   // 8: ㅃ
		assert_eq!(c.convert("ppa"), "빠");   // 8: ㅃ
		assert_eq!(c.convert("sa"), "사");    // 9: ㅅ
		assert_eq!(c.convert("ssa"), "싸");   // 10: ㅆ
		assert_eq!(c.convert("a"), "아");     // 11: ㅇ (母音始まり)
		assert_eq!(c.convert("ja"), "자");    // 12: ㅈ
		assert_eq!(c.convert("jja"), "짜");   // 13: ㅉ
		assert_eq!(c.convert("cha"), "차");   // 14: ㅊ
		assert_eq!(c.convert("ka"), "카");    // 15: ㅋ
		assert_eq!(c.convert("ta"), "타");    // 16: ㅌ
		assert_eq!(c.convert("pa"), "파");    // 17: ㅍ
		assert_eq!(c.convert("ha"), "하");    // 18: ㅎ
	}

	#[test]
	fn test_jungseong_complete() {
		let c = HangulConverter::new();

		// 全21種類の中声 (初声ㅇ)
		assert_eq!(c.convert("a"), "아");     // 0
		assert_eq!(c.convert("ae"), "애");    // 1
		assert_eq!(c.convert("ya"), "야");    // 2
		assert_eq!(c.convert("yae"), "얘");   // 3
		assert_eq!(c.convert("eo"), "어");    // 4
		assert_eq!(c.convert("e"), "에");     // 5
		assert_eq!(c.convert("yeo"), "여");   // 6
		assert_eq!(c.convert("ye"), "예");    // 7
		assert_eq!(c.convert("o"), "오");     // 8
		assert_eq!(c.convert("wa"), "와");    // 9
		assert_eq!(c.convert("wae"), "왜");   // 10
		assert_eq!(c.convert("oe"), "외");    // 11
		assert_eq!(c.convert("yo"), "요");    // 12
		assert_eq!(c.convert("u"), "우");     // 13
		assert_eq!(c.convert("wo"), "워");    // 14
		assert_eq!(c.convert("we"), "웨");    // 15
		assert_eq!(c.convert("wi"), "위");    // 16
		assert_eq!(c.convert("yu"), "유");    // 17
		assert_eq!(c.convert("eu"), "으");    // 18
		assert_eq!(c.convert("ui"), "의");    // 19
		assert_eq!(c.convert("i"), "이");     // 20
	}

	#[test]
	fn test_jongseong_complete() {
		let c = HangulConverter::new();

		// 全27種類の終声 (0=なし を除く)
		assert_eq!(c.convert("gag"), "각");   // g = 1
		assert_eq!(c.convert("gagg"), "갂");  // gg = 2
		assert_eq!(c.convert("gags"), "갃");  // gs = 3
		assert_eq!(c.convert("gan"), "간");   // n = 4
		assert_eq!(c.convert("ganj"), "갅");  // nj = 5
		assert_eq!(c.convert("ganh"), "갆");  // nh = 6
		assert_eq!(c.convert("gad"), "갇");   // d = 7
		assert_eq!(c.convert("gal"), "갈");   // l = 8
		assert_eq!(c.convert("galg"), "갉");  // lg = 9
		assert_eq!(c.convert("galm"), "갊");  // lm = 10
		assert_eq!(c.convert("galb"), "갋");  // lb = 11
		assert_eq!(c.convert("gals"), "갌");  // ls = 12
		assert_eq!(c.convert("galt"), "갍");  // lt = 13
		assert_eq!(c.convert("galp"), "갎");  // lp = 14
		assert_eq!(c.convert("galh"), "갏");  // lh = 15
		assert_eq!(c.convert("gam"), "감");   // m = 16
		assert_eq!(c.convert("gab"), "갑");   // b = 17
		assert_eq!(c.convert("gabs"), "값");  // bs = 18
		assert_eq!(c.convert("gas"), "갓");   // s = 19
		assert_eq!(c.convert("gass"), "갔");  // ss = 20
		assert_eq!(c.convert("gang"), "강");  // ng = 21
		assert_eq!(c.convert("gaj"), "갖");   // j = 22
		assert_eq!(c.convert("gach"), "갗");  // ch = 23
		assert_eq!(c.convert("gak"), "갘");   // k = 24
		assert_eq!(c.convert("gat"), "같");   // t = 25
		assert_eq!(c.convert("gap"), "갚");   // p = 26
		assert_eq!(c.convert("gah"), "갛");   // h = 27
	}

	// ==================== 実用単語テスト ====================

	#[test]
	fn test_common_korean_words() {
		let c = HangulConverter::new();

		// 基本単語
		assert_eq!(c.convert("mul"), "물");           // 水
		assert_eq!(c.convert("bul"), "불");           // 火
		assert_eq!(c.convert("bab"), "밥");           // ご飯
		assert_eq!(c.convert("jib"), "집");           // 家
		assert_eq!(c.convert("chaeg"), "책");         // 本
		assert_eq!(c.convert("hag gyo"), "학교");     // 学校
		assert_eq!(c.convert("seon saeng"), "선생");  // 先生
		assert_eq!(c.convert("hag saeng"), "학생");   // 学生

		// 動詞語幹
		assert_eq!(c.convert("ga da"), "가다");       // 行く
		assert_eq!(c.convert("o da"), "오다");        // 来る
		assert_eq!(c.convert("meog da"), "먹다");     // 食べる
		assert_eq!(c.convert("ma si da"), "마시다");  // 飲む
		assert_eq!(c.convert("bo da"), "보다");       // 見る
		assert_eq!(c.convert("deud da"), "듣다");     // 聞く

		// 形容詞
		assert_eq!(c.convert("joh da"), "좋다");       // 良い
		assert_eq!(c.convert("na bbeu da"), "나쁘다"); // 悪い
		assert_eq!(c.convert("keu da"), "크다");       // 大きい
		assert_eq!(c.convert("jag da"), "작다");       // 小さい
	}

	#[test]
	fn test_common_phrases() {
		let c = HangulConverter::new();

		// よく使うフレーズ
		assert_eq!(c.convert("ne"), "네");                       // はい
		assert_eq!(c.convert("a ni yo"), "아니요");              // いいえ
		assert_eq!(c.convert("gwaen chanh a yo"), "괜찮아요");   // 大丈夫です
		assert_eq!(c.convert("mo reu gess eo yo"), "모르겠어요"); // わかりません
		assert_eq!(c.convert("al gess seub ni da"), "알겠습니다"); // わかりました
	}	

	// ==================== 複雑な単語テスト ====================

	#[test]
	fn test_complex_words_double_jongseong() {
		let c = HangulConverter::new();

		// 二重終声を含む実際の単語
		assert_eq!(c.convert("dalg"), "닭");           // 鶏 (ㄹㄱ)
		assert_eq!(c.convert("heulg"), "흙");          // 土 (ㄹㄱ)
		assert_eq!(c.convert("salm"), "삶");           // 人生 (ㄹㅁ)
		assert_eq!(c.convert("jeolm da"), "젊다");     // 若い (ㄹㅁ)
		assert_eq!(c.convert("ilg da"), "읽다");       // 読む (ㄹㄱ)
		assert_eq!(c.convert("balb da"), "밟다");      // 踏む (ㄹㅂ)
		assert_eq!(c.convert("eobs da"), "없다");      // ない (ㅂㅅ)
		assert_eq!(c.convert("gabs"), "값");           // 値段 (ㅂㅅ)
		assert_eq!(c.convert("sags"), "삯");           // 賃金 (ㄹㄱ)
		assert_eq!(c.convert("neolb da"), "넓다");     // 広い (ㄹㅂ)
	}

	#[test]
	fn test_complex_words_tense_consonants() {
		let c = HangulConverter::new();

		// 濃音を含む単語
		assert_eq!(c.convert("eo ggae"), "어깨");       // 肩
		assert_eq!(c.convert("o bba"), "오빠");        // 兄 (女性から)
		assert_eq!(c.convert("eo ddeoh ge"), "어떻게"); // どうやって
		assert_eq!(c.convert("a jig"), "아직");        // まだ
		assert_eq!(c.convert("eo jjae"), "어째");      // なぜ
		assert_eq!(c.convert("ggo ma"), "꼬마");       // ちびっ子
		assert_eq!(c.convert("ssa u da"), "싸우다");   // 戦う
		assert_eq!(c.convert("na bbeu da"), "나쁘다"); // 悪い
		assert_eq!(c.convert("ye bbeu da"), "예쁘다"); // きれい
		assert_eq!(c.convert("ba bbeu da"), "바쁘다"); // 忙しい
	}

	#[test]
	fn test_complex_words_aspirated_consonants() {
		let c = HangulConverter::new();

		// 激音を含む単語
		assert_eq!(c.convert("chim dae"), "침대");     // ベッド
		assert_eq!(c.convert("chin gu"), "친구");      // 友達
		assert_eq!(c.convert("keo pi"), "커피");       // コーヒー
		assert_eq!(c.convert("ta da"), "타다");        // 乗る
		assert_eq!(c.convert("pa ran saeg"), "파란색"); // 青色
		assert_eq!(c.convert("keu da"), "크다");       // 大きい
		assert_eq!(c.convert("pyeon ji"), "편지");     // 手紙
		assert_eq!(c.convert("chug gu"), "축구");      // サッカー
		assert_eq!(c.convert("pyo"), "표");            // チケット
		assert_eq!(c.convert("hyang gi"), "향기");     // 香り
	}

	#[test]
	fn test_complex_words_compound_vowels() {
		let c = HangulConverter::new();

		// 複合母音を含む単語
		assert_eq!(c.convert("goe mul"), "괴물");      // 怪物
		assert_eq!(c.convert("hoe sa"), "회사");       // 会社
		assert_eq!(c.convert("ui sa"), "의사");        // 医者
		assert_eq!(c.convert("gwi"), "귀");            // 耳
		assert_eq!(c.convert("gwa il"), "과일");       // 果物
		assert_eq!(c.convert("wae"), "왜");            // なぜ
		assert_eq!(c.convert("wol yo il"), "월요일");  // 月曜日
		assert_eq!(c.convert("swi da"), "쉬다");       // 休む
		assert_eq!(c.convert("dwi"), "뒤");            // 後ろ
		assert_eq!(c.convert("bwa"), "봐");            // 見て (봐)
	}

	#[test]
	fn test_complex_words_long() {
		let c = HangulConverter::new();

		// 長い単語
		assert_eq!(c.convert("dae han min gug"), "대한민국");             // 大韓民国
		assert_eq!(c.convert("gyeong bog gung"), "경복궁");               // 景福宮
		assert_eq!(c.convert("in cheon gug je gong hang"), "인천국제공항"); // 仁川国際空港
		assert_eq!(c.convert("dae tong ryeong"), "대통령");               // 大統領
		assert_eq!(c.convert("chung cheong nam do"), "충청남도");         // 忠清南道
		assert_eq!(c.convert("bu san gwang yeog si"), "부산광역시");      // 釜山広域市
	}

	#[test]
	fn test_complex_words_foreign_loanwords() {
		let c = HangulConverter::new();

		// 外来語
		assert_eq!(c.convert("keo pi"), "커피");           // コーヒー
		assert_eq!(c.convert("keom pyu teo"), "컴퓨터");   // コンピューター
		assert_eq!(c.convert("tel le bi jeon"), "텔레비전"); // テレビ
		assert_eq!(c.convert("in teo nes"), "인터넷");     // インターネット
		assert_eq!(c.convert("seu ma teu pon"), "스마트폰"); // スマートフォン
		assert_eq!(c.convert("ho tel"), "호텔");           // ホテル
		assert_eq!(c.convert("taeg si"), "택시");          // タクシー
		assert_eq!(c.convert("beo seu"), "버스");          // バス
		assert_eq!(c.convert("pi a no"), "피아노");        // ピアノ
		assert_eq!(c.convert("ka me ra"), "카메라");       // カメラ
	}

	#[test]
	fn test_complex_words_conjugation() {
		let c = HangulConverter::new();

		// 動詞活用形
		assert_eq!(c.convert("meog eoss da"), "먹었다");       // 食べた
		assert_eq!(c.convert("ga go iss da"), "가고있다");     // 行っている
		assert_eq!(c.convert("ha go sip da"), "하고싶다");     // したい
		assert_eq!(c.convert("bol su iss da"), "볼수있다");    // 見ることができる
		assert_eq!(c.convert("hae ya ha da"), "해야하다");     // しなければならない
		assert_eq!(c.convert("meog eul geos i da"), "먹을것이다"); // 食べるだろう
		
		// 敬語形
		assert_eq!(c.convert("deu syeoss seub ni da"), "드셨습니다"); // 召し上がりました
		assert_eq!(c.convert("gye sib ni da"), "계십니다");   // いらっしゃいます
		assert_eq!(c.convert("ju se yo"), "주세요");          // ください
	}

	#[test]
	fn test_complex_words_particles() {
		let c = HangulConverter::new();

		// 助詞付き
		assert_eq!(c.convert("hag gyo e seo"), "학교에서");   // 学校で
		assert_eq!(c.convert("jib eu ro"), "집으로");         // 家へ
		assert_eq!(c.convert("chin gu wa"), "친구와");        // 友達と
		assert_eq!(c.convert("chaeg eul"), "책을");           // 本を
		assert_eq!(c.convert("mul i"), "물이");               // 水が
		assert_eq!(c.convert("sa ram eun"), "사람은");        // 人は
		assert_eq!(c.convert("na ra ui"), "나라의");          // 国の
	}

	#[test]
	fn test_complex_words_difficult_pronunciation() {
		let c = HangulConverter::new();

		// 発音が難しい単語
		assert_eq!(c.convert("chwi eob"), "취업");           // 就職
		assert_eq!(c.convert("gwan gwang"), "관광");          // 観光
		assert_eq!(c.convert("gyeol hon"), "결혼");           // 結婚
		assert_eq!(c.convert("cheol hag"), "철학");           // 哲学
		assert_eq!(c.convert("sim ri hag"), "심리학");        // 心理学
		assert_eq!(c.convert("gyeong je hag"), "경제학");     // 経済学
		assert_eq!(c.convert("jeong chi"), "정치");           // 政治
		assert_eq!(c.convert("mun hwa"), "문화");             // 文化
		assert_eq!(c.convert("yeog sa"), "역사");             // 歴史
		assert_eq!(c.convert("gwa hag"), "과학");             // 科学
	}

	#[test]
	fn test_complex_sentences() {
		let c = HangulConverter::new();

		// 短い文 (ダブルスペースで単語区切り)
		assert_eq!(
			c.convert("na neun  hag saeng  ib ni da"),
			"나는 학생 입니다"
		);  // 私は学生です
		
		assert_eq!(
			c.convert("o neul  nal ssi ga  joh seub ni da"),
			"오늘 날씨가 좋습니다"
		);  // 今日天気が良いです
		
		assert_eq!(
			c.convert("han gug eo reul  gong bu  ha go  iss seub ni da"),
			"한국어를 공부 하고 있습니다"
		);  // 韓国語を勉強しています
		
		assert_eq!(
			c.convert("man na seo  ban gab seub ni da"),
			"만나서 반갑습니다"
		);  // お会いできて嬉しいです
	}	
}