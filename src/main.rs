mod engine;
mod helper;

use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        eprintln!("usage: {} regex file", args[0]);
        let err: Box<dyn Error> = "invalid arguments".into();
        return Err(err);
    } else {
        match_file(&args[1], &args[2])?;
    }

    Ok(())
}

/// ファイルをオープンし、行ごとにマッチングを行う。
///
/// マッチングはそれぞれの行頭から1文字ずつずらして行い、
/// いずれかにマッチした場合に、その行がマッチしたもとみなす。
///
/// たとえば、abcdという文字列があった場合、以下の順にマッチが行われ、
/// このいずれかにマッチした場合、与えられた正規表現にマッチする行と判定する。
///
/// - abcd
/// - bcd
/// - cd
/// - d
fn match_file(expr: &str, file: &str) -> Result<(), Box<dyn Error>> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    engine::print(expr)?;
    println!();

    for line in reader.lines() {
        let line = line?;
        for (i, _) in line.char_indices() {
            if engine::do_matching(expr, &line[i..], true)? {
                println!("{line}");
                break;
            }
        }
    }

    Ok(())
}

// 単体テスト。プライベート関数もテスト可能
#[cfg(test)]
mod tests {
    use crate::{
        engine::do_matching,
        helper::{safe_add, SafeAdd},
        engine::print,
    };

    #[test]
    fn test_safe_add() {
        let n: usize = 10;
        assert_eq!(Some(30), n.safe_add(&20));

        let n: usize = !0; // 2^64 - 1 (64 bits CPU)
        assert_eq!(None, n.safe_add(&1));

        let mut n: usize = 10;
        assert!(safe_add(&mut n, &20, || ()).is_ok());

        let mut n: usize = !0;
        assert!(safe_add(&mut n, &1, || ()).is_err());
    }

    #[test]
    fn test_matching() {
        // パースエラー
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());

        // パース成功、マッチ成功
        assert!(do_matching("abc|def", "def", true).unwrap());
        assert!(do_matching("(abc)*", "abcabc", true).unwrap());
        assert!(do_matching("(ab|cd)+", "abcdcd", true).unwrap());
        assert!(do_matching("abc?", "ab", true).unwrap());

        // パース成功、マッチ失敗
        assert!(!do_matching("abc|def", "efa", true).unwrap());
        assert!(!do_matching("(ab|cd)+", "", true).unwrap());
        assert!(!do_matching("abc?", "acb", true).unwrap());

        // 自分で書いたテスト
        // ?演算子を使用。任意の文字が0回または1回出現する
        assert!(do_matching(".?.", "abc", true).unwrap());
        assert!(do_matching(".?.", "ac", true).unwrap());

        // *演算子を使用。任意の文字が0回以上出現する
        assert!(do_matching("a.*b", "acb", true).unwrap());
        assert!(do_matching("a.*b", "ab", true).unwrap());

        // +演算子を使用。任意の文字が1回以上出現する
        assert!(do_matching("a.+b", "acb", true).unwrap());
        assert!(!do_matching("a.+b", "ab", true).unwrap());  // 中に何か文字がなければならない

        // ?と*を組み合わせて使用
        assert!(do_matching(".?.*a", "ba", true).unwrap());
        assert!(do_matching(".?.*a", "a", true).unwrap());

        // ?と+を組み合わせて使用
        assert!(do_matching(".?.+a", "ba", true).unwrap());
        assert!(!do_matching(".?.+a", "a", true).unwrap());  // 中に何か文字がなければならない

        // *と+を組み合わせて使用
        assert!(do_matching("a.*.+b", "acccb", true).unwrap());
        assert!(do_matching("a.*.+b", "accb", true).unwrap());
        assert!(!do_matching("a.*.+b", "ab", true).unwrap());  // 中に何か文字がなければならない

        // ?、*、+を全て組み合わせて使用
        assert!(do_matching("a?.*.+b", "acb", true).unwrap());
        assert!(do_matching("a?.*.+b", "accb", true).unwrap());
        assert!(!do_matching("a?.*.+b", "b", true).unwrap());  // 'a'または何か文字がなければならない

    }
    #[test]
    fn test_print(){
        assert!(print("abc").is_ok());
        assert!(print("abc|def").is_ok());
        assert!(print("(abc)*").is_ok());
        assert!(print("(ab|cd)+").is_ok());
        assert!(print("abc?").is_ok());
        assert!(print("e.s.*").is_ok());
    }
}
