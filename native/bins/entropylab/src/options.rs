//! The command line.

use el_bip32::ScriptType;

pub struct Options {
    phrase: String,
    passphrase: String,
    script: ScriptType,
    account: u32,
    frames: Option<u32>,
    headless: bool,
}

impl Options {
    /// The BIP-39 test phrase, used when no `--phrase` is given. It is the
    /// published vector, so the window shows known keys on a first run.
    const DEFAULT_PHRASE: &'static str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    pub fn parse(arguments: impl Iterator<Item = String>) -> Self {
        let mut options = Self::default();
        let mut arguments = arguments;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--phrase" => options.phrase = arguments.next().unwrap_or_default(),
                "--passphrase" => options.passphrase = arguments.next().unwrap_or_default(),
                "--account" => {
                    options.account = arguments
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(0);
                }
                "--script" => {
                    if let Some(name) = arguments.next() {
                        options.script = Self::script_from(&name).unwrap_or(options.script);
                    }
                }
                "--frames" => options.frames = arguments.next().and_then(|v| v.parse().ok()),
                "--print" => options.headless = true,
                _ => {}
            }
        }
        options
    }

    fn script_from(name: &str) -> Option<ScriptType> {
        ScriptType::ALL
            .into_iter()
            .find(|script| script.bip().eq_ignore_ascii_case(name))
    }

    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    pub fn passphrase(&self) -> &str {
        &self.passphrase
    }

    pub const fn script(&self) -> ScriptType {
        self.script
    }

    pub const fn account(&self) -> u32 {
        self.account
    }

    pub const fn headless(&self) -> bool {
        self.headless
    }

    pub fn reached_frame_limit(&self, painted: u32) -> bool {
        self.frames.is_some_and(|limit| painted >= limit)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            phrase: Self::DEFAULT_PHRASE.to_owned(),
            passphrase: String::new(),
            script: ScriptType::P2wpkh,
            account: 0,
            frames: None,
            headless: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Options;
    use el_bip32::ScriptType;

    fn parse(arguments: &[&str]) -> Options {
        Options::parse(arguments.iter().map(|a| (*a).to_owned()))
    }

    #[test]
    fn the_defaults_use_the_published_phrase_and_native_segwit() {
        let options = parse(&[]);
        assert!(options.phrase().starts_with("abandon abandon"));
        assert_eq!(options.script(), ScriptType::P2wpkh);
        assert_eq!(options.account(), 0);
        assert!(!options.headless());
    }

    #[test]
    fn every_bip_name_selects_its_script() {
        for script in ScriptType::ALL {
            let options = parse(&["--script", script.bip()]);
            assert_eq!(options.script(), script);
        }
        assert_eq!(parse(&["--script", "bip84"]).script(), ScriptType::P2wpkh);
    }

    #[test]
    fn an_unknown_script_keeps_the_default() {
        assert_eq!(
            parse(&["--script", "nonsense"]).script(),
            ScriptType::P2wpkh
        );
    }

    #[test]
    fn the_account_index_parses() {
        assert_eq!(parse(&["--account", "7"]).account(), 7);
        assert_eq!(parse(&["--account", "oops"]).account(), 0);
    }

    #[test]
    fn the_frame_limit_stops_the_loop() {
        let options = parse(&["--frames", "3"]);
        assert!(!options.reached_frame_limit(2));
        assert!(options.reached_frame_limit(3));
        assert!(!parse(&[]).reached_frame_limit(10_000));
    }

    #[test]
    fn the_print_flag_selects_the_headless_path() {
        assert!(parse(&["--print"]).headless());
    }

    #[test]
    fn the_phrase_and_passphrase_are_read() {
        let options = parse(&["--phrase", "one two", "--passphrase", "secret"]);
        assert_eq!(options.phrase(), "one two");
        assert_eq!(options.passphrase(), "secret");
    }
}
